/*
Grammar

Cell ::= Number | Formula
Formula ::= “=“ Expr
Expr ::= Term ('+' Term | '-' Term)*
Term ::= Factor ('*' Factor | '/' Factor)*
Factor ::= ['-'] (Value | '(' Expr ')')
Value ::= Function | Coordinate | Number
Function ::= FnId '(' Range ')'
Range ::= Coordinate '->' Coordinate
Coordinate ::= '[' Number ',' Number ']'
Number ::= Digit+

TODO:
- Improve error handling while parsing. Ideally, we would get "unexpected token in line x col y, found: w expected z"
- Test for parsers
*/
type ParseResult<'a, Output> = Result<(Output, &'a str), &'static str>;

trait Parser<'a, T> {
  fn parse(&self, input: &'a str) -> ParseResult<'a, T>;
}

impl<'a, F, T> Parser<'a, T> for F
where
  F: Fn(&'a str) -> ParseResult<'a, T>,
{
  fn parse(&self, input: &'a str) -> ParseResult<'a, T> {
    self(input)
  }
}

#[derive(Clone)]
pub enum ExprTree {
  Empty,
  Leaf(ValueNode),
  Unary(Box<UnaryNode>),
  Binary(Box<BinaryNode>),
}

impl Default for ExprTree {
  fn default() -> Self {
    ExprTree::Empty
  }
}

#[derive(Clone, Copy)]
pub enum ValueNode {
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
  pub fn apply(&self, val: f64) -> f64 {
    match self {
      UnaryOp::Not => -1. * val,
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
  pub fn apply(&self, val1: f64, val2: f64) -> f64 {
    match self {
      BinaryOp::Sum => val1 + val2,
      BinaryOp::Sub => val1 - val2,
      BinaryOp::Mul => val1 * val2,
      BinaryOp::Div => val1 / val2,
    }
  }
}

// Spreadsheet Parsers

pub fn cell(input: &str) -> ParseResult<ExprTree> {
  // Cell ::= Number | Formula
  let num_node = map(number, |n| ExprTree::Leaf(ValueNode::Num(n)));
  let (tree, input) = either(num_node, formula).parse(input)?;
  if !input.is_empty() {
    Err("Expected input to be empty")
  } else {
    Ok((tree, input))
  }
}

fn formula(input: &str) -> ParseResult<ExprTree> {
  // Formula ::= “=“ Expr
  let (_, input) = literal("=").parse(input)?;
  let (res, input) = expr(input)?;
  if input.is_empty() {
    Ok((res, input))
  } else {
    Err("Expected input to be empty")
  }
}

fn expr(input: &str) -> ParseResult<ExprTree> {
  // Expr ::= Term ('+' Term | '-' Term)*
  let (first_term, input) = term(input)?;
  let sum = map(literal("+"), |_| BinaryOp::Sum);
  let sub = map(literal("-"), |_| BinaryOp::Sub);
  let sum_term = pair(sum, term);
  let sub_term = pair(sub, term);
  let (others, input) = zero_or_more(either(sum_term, sub_term)).parse(input)?;
  Ok((reduce_trees(first_term, others), input))
}

fn term(input: &str) -> ParseResult<ExprTree> {
  // Term ::= Factor ('*' Factor | '/' Factor)*
  let (first_factor, input) = factor(input)?;
  let mul = map(literal("*"), |_| BinaryOp::Mul);
  let div = map(literal("/"), |_| BinaryOp::Div);
  let mul_fact = pair(mul, factor);
  let div_fact = pair(div, factor);
  let (others, input) = zero_or_more(either(mul_fact, div_fact)).parse(input)?;
  Ok((reduce_trees(first_factor, others), input))
}

fn factor(mut input: &str) -> ParseResult<ExprTree> {
  // Factor ::= ['-'] (Number | '(' Expr ')')
  let mut negate = false;
  if let Ok((_, next_input)) = literal("-").parse(input) {
    negate = true;
    input = next_input;
  }
  let paren_expr = right(literal("("), left(expr, literal(")")));
  let val_or_expr = either(value, paren_expr);
  let (child, input) = val_or_expr.parse(input)?;

  if negate {
    let node = UnaryNode {
      op: UnaryOp::Not,
      child,
    };
    let tree = ExprTree::Unary(Box::new(node));
    Ok((tree, input))
  } else {
    Ok((child, input))
  }
}

fn value(input: &str) -> ParseResult<ExprTree> {
  // Value ::= Coord | Function | Number
  let num_val = map(number, ValueNode::Num);
  let num_or_coord = either(num_val, coord);
  let (val, input) = num_or_coord.parse(input)?;
  Ok((ExprTree::Leaf(val), input))
}

fn coord(input: &str) -> ParseResult<ValueNode> {
  // Coordinate ::= '[' Number ',' Number ']'
  let (_, input) = literal("[").parse(input)?;
  // TODO(adelavega): We should have a float, and int parser, and use int here.
  let (col, input) = number(input)?;
  let (_, input) = literal(",").parse(input)?;
  let (row, input) = number(input)?;
  let (_, input) = literal("]").parse(input)?;
  Ok((ValueNode::Coord(row as usize, col as usize), input))
}

fn reduce_trees(first: ExprTree, others: Vec<(BinaryOp, ExprTree)>) -> ExprTree {
  if others.is_empty() {
    return first;
  }
  let mut others_iter = others.into_iter().rev();

  let (op, right) = others_iter.next().unwrap();
  let mut node = BinaryNode {
    op,
    left: ExprTree::Empty,
    right,
  };
  for (op, left) in others_iter {
    node.left = left;
    node = BinaryNode {
      op,
      left: ExprTree::Empty,
      right: ExprTree::Binary(Box::new(node)),
    };
  }
  node.left = first;
  ExprTree::Binary(Box::new(node))
}

fn number(input: &str) -> ParseResult<f64> {
  // Number ::= Digit+
  let (num_vec, input) = one_or_more(predicate(any_char, |c| c.is_numeric())).parse(input)?;
  let num = num_vec.into_iter().collect::<String>().parse().unwrap();
  Ok((num, input))
}

// Generic Parsers

fn any_char(input: &str) -> ParseResult<char> {
  if let Some(c) = input.chars().next() {
    Ok((c, &input[c.len_utf8()..]))
  } else {
    Err("End of string")
  }
}

// Combinators

fn right<'a, A, B>(parser1: impl Parser<'a, A>, parser2: impl Parser<'a, B>) -> impl Parser<'a, B> {
  move |input: &'a str| {
    let (_, input) = parser1.parse(input)?;
    let (b, input) = parser2.parse(input)?;
    Ok((b, input))
  }
}

fn left<'a, A, B>(parser1: impl Parser<'a, A>, parser2: impl Parser<'a, B>) -> impl Parser<'a, A> {
  move |input: &'a str| {
    let (a, input) = parser1.parse(input)?;
    let (_, input) = parser2.parse(input)?;
    Ok((a, input))
  }
}

fn either<'a, A>(parser1: impl Parser<'a, A>, parser2: impl Parser<'a, A>) -> impl Parser<'a, A> {
  move |input: &'a str| {
    if let Ok(res) = parser1.parse(input) {
      Ok(res)
    } else if let Ok(res) = parser2.parse(input) {
      Ok(res)
    } else {
      Err("Parsers unable to parse input")
    }
  }
}

fn one_or_more<'a, A>(parser: impl Parser<'a, A>) -> impl Parser<'a, Vec<A>> {
  move |mut input: &'a str| {
    let mut results = vec![];
    let (res, next_input) = parser.parse(input)?;
    results.push(res);
    input = next_input;
    while let Ok((res, next_input)) = parser.parse(input) {
      results.push(res);
      input = next_input;
    }
    Ok((results, input))
  }
}

fn zero_or_more<'a, A>(parser: impl Parser<'a, A>) -> impl Parser<'a, Vec<A>> {
  move |mut input: &'a str| {
    let mut results = vec![];
    while let Ok((res, next_input)) = parser.parse(input) {
      results.push(res);
      input = next_input;
    }
    Ok((results, input))
  }
}

fn map<'a, A, F, B>(parser: impl Parser<'a, A>, f: F) -> impl Parser<'a, B>
where
  F: Fn(A) -> B,
{
  move |input| {
    let (resp, input) = parser.parse(input)?;
    Ok((f(resp), input))
  }
}

fn predicate<'a, A>(
  parser: impl Parser<'a, A>,
  pred_fn: impl Fn(&A) -> bool,
) -> impl Parser<'a, A> {
  move |input: &'a str| {
    let (res, input) = parser.parse(input)?;
    if pred_fn(&res) {
      Ok((res, input))
    } else {
      Err("Predicate failed")
    }
  }
}

fn pair<'a, A, B>(
  parser1: impl Parser<'a, A>,
  parser2: impl Parser<'a, B>,
) -> impl Parser<'a, (A, B)> {
  move |input: &'a str| {
    let (resp1, input) = parser1.parse(input)?;
    let (resp2, input) = parser2.parse(input)?;
    Ok(((resp1, resp2), input))
  }
}

fn literal<'a>(pattern: &'static str) -> impl Parser<'a, ()> {
  move |input: &'a str| {
    if input.starts_with(pattern) {
      Ok(((), &input[pattern.len()..]))
    } else {
      Err("no match")
    }
  }
}

#[cfg(test)]
mod tests {
  mod combinators {
    use super::super::*;
    #[test]
    fn literal_smoketest() {
      let p = literal("a");
      assert_eq!(p.parse("alpha"), Ok(((), "lpha")));
      assert!(p.parse("this wont match").is_err());
    }

    #[test]
    fn pair_smoketest() {
      let p = pair(literal("a"), literal("l"));

      assert_eq!(p.parse("alpha"), Ok((((), ()), "pha")));
      assert!(p.parse("a not followed by l").is_err());
    }

    #[test]
    fn predicate_smoketest() {
      let p = predicate(any_char, |c| *c == 'a');

      assert_eq!(p.parse("alpha"), Ok(('a', "lpha")));
      assert!(p.parse("beta").is_err());
    }

    #[test]
    fn map_smoketest() {
      let p = map(any_char, |c| c.is_numeric());

      assert_eq!(p.parse("alpha"), Ok((false, "lpha")));
      assert_eq!(p.parse("123"), Ok((true, "23")));
    }

    #[test]
    fn zero_or_more_smoketest() {
      let p = zero_or_more(literal("a"));

      assert_eq!(p.parse("bcd"), Ok((vec![], "bcd")));
      assert_eq!(p.parse("aaabcd"), Ok((vec![(), (), ()], "bcd")));
    }

    #[test]
    fn one_or_more_smoketest() {
      let p = one_or_more(literal("a"));

      assert!(p.parse("bcd").is_err());
      assert_eq!(p.parse("aaabcd"), Ok((vec![(), (), ()], "bcd")));
    }

    #[test]
    fn either_smoketest() {
      let p = either(literal("a"), literal("b"));

      assert!(p.parse("cd").is_err());
      assert_eq!(p.parse("abcd"), Ok(((), "bcd")));
      assert_eq!(p.parse("bcd"), Ok(((), "cd")));
    }

    #[test]
    fn left_smoketest() {
      let p = left(any_char, literal("b"));

      assert!(p.parse("bcd").is_err());
      assert_eq!(p.parse("abc"), Ok(('a', "c")));
    }

    #[test]
    fn right_smoketest() {
      let p = right(any_char, literal("b"));

      assert!(p.parse("bcd").is_err());
      assert_eq!(p.parse("abc"), Ok(((), "c")));
    }
  }
}