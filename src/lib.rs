/*
Formula Grammar

Cell ::= Number | Formula
Formula ::= “=“ Expr

https://stackoverflow.com/questions/9785553/how-does-a-simple-calculator-with-parentheses-work
Expr ::= Term ('+' Term | '-' Term)*
Term ::= Factor ('*' Factor | '/' Factor)*
Factor ::= ['-'] (Value | '(' Expr ')')
Value ::= Function | Reference | Number
Function ::= FnId '(' Range ')'
Range ::= Reference '->' Reference
Reference ::= '[' Number ',' Number ']'
Number ::= Digit+

TODO:
- Improve error handling while parsing. Ideally, we would get "unexpected token in line x col y, found: w expected z"
- Read how to lay out Enums that use Structs as values. Seems weird to share the name.
- Rename Ref to something else since there's a ref primitive in rust. CellRef?
- Test for parsers
- Test for combinators
- Make eval only calculate numbers, have separate method to return outbound refs.
*/
use std::collections::HashSet;
use std::mem;

#[derive(Clone)]
enum ExprTree {
    Empty,
    // TODO: read about enum and structs and how to properly combine both
    // Is it ok to have an enum value have the same name as the struct it refers
    // to?
    Val(Value),
    Unary(Box<UnaryNode>),
    Binary(Box<BinaryNode>),
}

#[derive(Clone)]
struct UnaryNode {
    op: UnaryOp,
    child: ExprTree,
}

#[derive(Clone)]
struct BinaryNode {
    op: BinaryOp,
    left: ExprTree,
    right: ExprTree,
}

#[derive(Clone, Copy)]
enum Value {
    Num(f64),
    // TODO: read about enum and structs and how to properly combine both
    // Is it ok to have an enum value have the same name as the struct it refers
    // to?
    Ref(usize, usize),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Ref {
    row: usize,
    col: usize,
}

struct Spreadsheet {
    grid: Vec<Vec<Cell>>,
}

#[derive(Clone)]
struct Cell {
    raw: String,
    expr: ExprTree,
    out: f64,
    outbound_refs: HashSet<Ref>,
    inbound_refs: HashSet<Ref>,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            raw: "".to_string(),
            expr: ExprTree::Empty,
            out: 0.,
            outbound_refs: HashSet::new(),
            inbound_refs: HashSet::new(),
        }
    }
}

impl Cell {
    fn new() -> Cell {
        Default::default()
    }
}

#[derive(Debug, Clone, PartialEq)]
enum UnaryOp {
    Not,
}

impl UnaryOp {
    fn apply(&self, val: f64) -> f64 {
        match self {
            UnaryOp::Not => -1. * val,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum BinaryOp {
    Sum,
    Sub,
    Mul,
    Div,
}

impl BinaryOp {
    fn apply(&self, val1: f64, val2: f64) -> f64 {
        match self {
            BinaryOp::Sum => val1 + val2,
            BinaryOp::Sub => val1 - val2,
            BinaryOp::Mul => val1 * val2,
            BinaryOp::Div => val1 / val2,
        }
    }
}

impl Spreadsheet {
    pub fn new() -> Spreadsheet {
        // TODO(adelavega): Does derive clone do a deep copy of the box values?
        Spreadsheet {
            grid: vec![vec![Cell::new(); 100]; 100],
        }
    }
    pub fn set(&mut self, row: usize, col: usize, raw: &str) -> Result<&Cell, &str> {
        if self.grid[0].len() <= col {
            self.resize_cols(col * 2);
        }
        if self.grid.len() <= row {
            self.resize_rows(row * 2);
        }

        let cur_ref = Ref { row, col };

        // Replace old cell with a placeholder to deal with expired inbound references
        let old_cell = mem::replace(&mut self.grid[row][col], Cell::new());
        self.remove_inbound(cur_ref, &old_cell.outbound_refs);

        // Create new cell
        let (expr, _) = cell(raw)?;
        let (out, outbound_refs) = eval(self, &expr);
        let inbound_refs = old_cell.inbound_refs.clone();
        let new_cell = Cell {
            raw: raw.to_string(),
            expr,
            out,
            outbound_refs,
            inbound_refs,
        };

        // Replace placeholder with new cell and add new inbound references
        self.add_inbound(cur_ref, &new_cell.outbound_refs);
        self.grid[row][col] = new_cell;

        if self.has_cycle(cur_ref) {
            self.grid[row][col] = old_cell;
            return Err("This cell introduces a cycle!");
        }

        // Our references form a DAG, we can toposort it to have the correct
        // order we should re-eval our dependencies.
        let eval_order = self.toposort_inbound(cur_ref);
        for r in eval_order {
            let (new_out, _) = eval(self, &self.grid[r.row][r.col].expr);
            self.grid[r.row][r.col].out = new_out;
        }

        Ok(&self.grid[row][col])
    }

    fn resize_rows(&mut self, new_len: usize) {
        self.grid
            .resize(new_len, vec![Cell::new(); self.grid[0].len()]);
    }

    fn resize_cols(&mut self, new_len: usize) {
        for row in &mut self.grid {
            row.resize(new_len, Cell::new());
        }
    }

    fn remove_inbound(&mut self, r: Ref, targets: &HashSet<Ref>) {
        for t in targets.iter() {
            self.grid[t.row][t.col].inbound_refs.remove(&r);
        }
    }

    fn add_inbound(&mut self, r: Ref, targets: &HashSet<Ref>) {
        for t in targets.iter() {
            self.grid[t.row][t.col].inbound_refs.insert(r);
        }
    }

    fn has_cycle(&self, start: Ref) -> bool {
        self.recursive_has_cycle(start, &mut HashSet::new())
    }

    fn recursive_has_cycle(&self, start: Ref, visited: &mut HashSet<Ref>) -> bool {
        if !visited.insert(start) {
            return true;
        }
        for r in self.grid[start.row][start.col].outbound_refs.iter() {
            if self.recursive_has_cycle(*r, visited) {
                return true;
            }
        }
        false
    }

    fn toposort_inbound(&self, start: Ref) -> Vec<Ref> {
        let mut sorted = vec![];
        self.recursive_toposort_inbound(start, &mut sorted);
        sorted
    }

    fn recursive_toposort_inbound(&self, start: Ref, result: &mut Vec<Ref>) {
        result.push(start);
        for r in self.grid[start.row][start.col].inbound_refs.iter() {
            self.recursive_toposort_inbound(*r, result);
        }
    }
}

// TODO(adelavega): not a fan of returning both things here
// Make eval only calculate a number, and bite the performance cost
// and find references in separate method.
fn eval(ss: &Spreadsheet, tree: &ExprTree) -> (f64, HashSet<Ref>) {
    let mut refs = HashSet::new();
    let out = recursive_eval(ss, tree, &mut refs);
    (out, refs)
}

fn recursive_eval(ss: &Spreadsheet, tree: &ExprTree, refs: &mut HashSet<Ref>) -> f64 {
    match tree {
        ExprTree::Val(val) => match val {
            Value::Num(n) => *n,
            Value::Ref(row, col) => {
                refs.insert(Ref {
                    row: *row,
                    col: *col,
                });
                // TODO: this might be out of bounds. Better to call a method to get?
                ss.grid[*row][*col].out
            }
        },
        ExprTree::Unary(u) => u.op.apply(recursive_eval(ss, &u.child, refs)),
        ExprTree::Binary(b) => b.op.apply(
            recursive_eval(ss, &b.left, refs),
            recursive_eval(ss, &b.right, refs),
        ),
        ExprTree::Empty => panic!("Found empty tree node"),
    }
}

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

// Spreadsheet Parsers

fn cell(input: &str) -> ParseResult<ExprTree> {
    // Cell ::= Number | Formula
    let num_node = map(number, |n| ExprTree::Val(Value::Num(n)));
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
    // Value ::= Reference | Function | Number
    let num_val = map(number, Value::Num);
    let num_or_ref = either(num_val, reference);
    let (val, input) = num_or_ref.parse(input)?;
    Ok((ExprTree::Val(val), input))
}

fn reference(input: &str) -> ParseResult<Value> {
    // Reference ::= '[' Number ',' Number ']'
    let (_, input) = literal("[").parse(input)?;
    // TODO(adelavega): We should a float, and int parser, and use int here.
    let (row, input) = number(input)?;
    let (_, input) = literal(",").parse(input)?;
    let (col, input) = number(input)?;
    let (_, input) = literal("]").parse(input)?;
    Ok((Value::Ref(row as usize, col as usize), input))
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
    use super::*;

    #[test]
    fn set_evals_formula_with_numbers() {
        let mut ss = Spreadsheet::new();
        let c1 = ss.set(0, 0, "=1+2*10-2").unwrap();
        assert_eq!(c1.out, 19.);

        let c2 = ss.set(0, 1, "=1+-(1+2*10)").unwrap();
        assert_eq!(c2.out, -20.);
    }

    #[test]
    fn set_evals_formula_with_ref() {
        let mut ss = Spreadsheet::new();
        ss.set(0, 0, "1").unwrap();
        ss.set(0, 1, "2").unwrap();
        ss.set(1, 0, "3").unwrap();
        ss.set(1, 1, "4").unwrap();
        let c = ss.set(2, 2, "=[0,0]+[0,1]+[1,0]+[1,1]").unwrap();
        assert_eq!(c.out, 10.);
    }

    #[test]
    fn set_detects_ref_cycle() {
        let mut ss = Spreadsheet::new();
        ss.set(0, 0, "=[0,1]").unwrap();
        ss.set(0, 1, "=[1,0]").unwrap();
        assert!(ss.set(1, 0, "=[0,0]").is_err())
    }

    #[test]
    fn set_evals_all_inbound() {
        let mut ss = Spreadsheet::new();
        ss.set(0, 0, "10").unwrap();
        ss.set(0, 1, "=[0,0]*2").unwrap();
        ss.set(1, 0, "=[0,0]*3").unwrap();
        ss.set(1, 1, "=[1,0]*4").unwrap();

        ss.set(0, 0, "1").unwrap();
        assert_eq!(ss.grid[0][0].out, 1.);
        assert_eq!(ss.grid[0][1].out, 2.);
        assert_eq!(ss.grid[1][0].out, 3.);
        assert_eq!(ss.grid[1][1].out, 12.);
    }

    #[test]
    fn set_updates_inbound_outbound_refs() {
        let mut ss = Spreadsheet::new();
        ss.set(0, 0, "=[0,1]").unwrap();
        ss.set(1, 1, "=[0,0]").unwrap();

        ss.set(0, 0, "1").unwrap();
        assert_eq!(
            ss.grid[0][0].inbound_refs,
            vec![Ref { row: 1, col: 1 }].into_iter().collect()
        );
        assert_eq!(ss.grid[0][0].outbound_refs, HashSet::new());

        assert_eq!(ss.grid[0][1].inbound_refs, HashSet::new());
        assert_eq!(ss.grid[0][1].outbound_refs, HashSet::new());

        assert_eq!(ss.grid[1][1].inbound_refs, HashSet::new());
        assert_eq!(
            ss.grid[1][1].outbound_refs,
            vec![Ref { row: 0, col: 0 }].into_iter().collect()
        );
    }
}
