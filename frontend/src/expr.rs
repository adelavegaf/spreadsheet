use super::parser::cell;
use super::Spreadsheet;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::ops;

#[derive(Clone)]
pub enum ExprTree {
  Empty,
  Error(String),
  Leaf(ValueNode),
  Unary(Box<UnaryNode>),
  Binary(Box<BinaryNode>),
}

impl ExprTree {
  pub fn new(input: &str) -> ExprTree {
    match cell(input) {
      Ok((expr, _)) => expr,
      Err(e) => ExprTree::Error(e.to_string()),
    }
  }

  pub fn eval(&self, ss: &Spreadsheet) -> ExprResult {
    match self {
      ExprTree::Empty => ExprResult::Text("".to_string()),
      ExprTree::Error(e) => ExprResult::Error(e.clone()),
      ExprTree::Leaf(ValueNode::Num(n)) => ExprResult::Num(*n),
      ExprTree::Leaf(ValueNode::Coord(row, col)) => ss.get(*row, *col).out.clone(),
      ExprTree::Leaf(ValueNode::Text(t)) => ExprResult::Text(t.clone()),
      ExprTree::Unary(u) => u.op.apply(u.child.eval(ss)),
      ExprTree::Binary(b) => b.op.apply(b.left.eval(ss), b.right.eval(ss)),
    }
  }

  pub fn fill_outbound(&self, ss: &Spreadsheet, outbound: &mut HashSet<usize>) {
    match self {
      ExprTree::Empty => (),
      ExprTree::Error(_) => (),
      ExprTree::Leaf(ValueNode::Text(_)) => (),
      ExprTree::Leaf(ValueNode::Num(_)) => (),
      ExprTree::Leaf(ValueNode::Coord(row, col)) => {
        outbound.insert(ss.get_index(*row, *col));
      }
      ExprTree::Unary(u) => u.child.fill_outbound(ss, outbound),
      ExprTree::Binary(b) => {
        b.left.fill_outbound(ss, outbound);
        b.right.fill_outbound(ss, outbound);
      }
    }
  }
}

impl Default for ExprTree {
  fn default() -> Self {
    ExprTree::Empty
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ExprResult {
  Num(f64),
  Text(String),
  Error(String),
}

impl PartialEq for ExprResult {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (ExprResult::Num(n1), ExprResult::Num(n2)) => n1 == n2,
      (ExprResult::Text(t1), ExprResult::Text(t2)) => t1 == t2,
      (ExprResult::Error(e1), ExprResult::Error(e2)) => e1 == e2,
      _ => false,
    }
  }
}

impl ops::Add<ExprResult> for ExprResult {
  type Output = ExprResult;

  fn add(self, rhs: ExprResult) -> Self::Output {
    match (&self, &rhs) {
      (ExprResult::Num(n1), ExprResult::Num(n2)) => ExprResult::Num(n1 + n2),
      (ExprResult::Text(t1), ExprResult::Text(t2)) => ExprResult::Text(format!("{}{}", t1, t2)),
      _ => ExprResult::Error(format!("can't add {:?} with {:?}", self, rhs)),
    }
  }
}

impl ops::Sub<ExprResult> for ExprResult {
  type Output = ExprResult;

  fn sub(self, rhs: ExprResult) -> Self::Output {
    match (&self, &rhs) {
      (ExprResult::Num(n1), ExprResult::Num(n2)) => ExprResult::Num(n1 - n2),
      _ => ExprResult::Error(format!("can't sub {:?} with {:?}", self, rhs)),
    }
  }
}

impl ops::Mul<ExprResult> for ExprResult {
  type Output = ExprResult;

  fn mul(self, rhs: ExprResult) -> Self::Output {
    match (&self, &rhs) {
      (ExprResult::Num(n1), ExprResult::Num(n2)) => ExprResult::Num(n1 * n2),
      _ => ExprResult::Error(format!("can't mul {:?} with {:?}", self, rhs)),
    }
  }
}

impl ops::Div<ExprResult> for ExprResult {
  type Output = ExprResult;

  fn div(self, rhs: ExprResult) -> Self::Output {
    match (&self, &rhs) {
      (ExprResult::Num(n1), ExprResult::Num(n2)) => ExprResult::Num(n1 / n2),
      _ => ExprResult::Error(format!("can't div {:?} with {:?}", self, rhs)),
    }
  }
}

#[derive(Clone)]
pub enum ValueNode {
  Text(String),
  Num(f64),
  Coord(usize, usize),
}

#[derive(Clone)]
pub struct UnaryNode {
  pub op: UnaryOp,
  pub child: ExprTree,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
  Not,
}

impl UnaryOp {
  pub fn apply(&self, val: ExprResult) -> ExprResult {
    match self {
      UnaryOp::Not => ExprResult::Num(-1.) * val,
    }
  }
}

#[derive(Clone)]
pub struct BinaryNode {
  pub op: BinaryOp,
  pub left: ExprTree,
  pub right: ExprTree,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
  Sum,
  Sub,
  Mul,
  Div,
}

impl BinaryOp {
  pub fn apply(&self, val1: ExprResult, val2: ExprResult) -> ExprResult {
    match self {
      BinaryOp::Sum => val1 + val2,
      BinaryOp::Sub => val1 - val2,
      BinaryOp::Mul => val1 * val2,
      BinaryOp::Div => val1 / val2,
    }
  }
}
