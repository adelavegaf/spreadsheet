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
*/

#[derive(Clone)]
enum ExprTree {
    Empty,
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

#[derive(Clone)]
enum Value {
    Num(f64),
    Ref(usize, usize),
}

struct Spreadsheet {
    grid: Vec<Vec<Cell>>,
}

#[derive(Clone)]
struct Cell {
    raw: String,
    expr: ExprTree,
    out: f64,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            raw: "".to_string(),
            expr: ExprTree::Empty,
            out: 0.,
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

        let (expr, _) = cell(raw)?;
        let out = eval(self, &expr);
        let c = &mut self.grid[row][col];
        c.raw = raw.to_string();
        c.expr = expr;
        c.out = out;
        Ok(c)
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
}

fn eval(ss: &Spreadsheet, tree: &ExprTree) -> f64 {
    match tree {
        ExprTree::Val(val) => match val {
            Value::Num(n) => *n,
            Value::Ref(i, j) => ss.grid[*i][*j].out,
        },
        ExprTree::Unary(u) => u.op.apply(eval(ss, &u.child)),
        ExprTree::Binary(b) => b.op.apply(eval(ss, &b.left), eval(ss, &b.right)),
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
    fn set_formula_with_numbers() {
        let mut ss = Spreadsheet::new();
        let c1 = ss.set(0, 0, "=1+2*10-2").unwrap();
        assert_eq!(c1.out, 19.);

        let c2 = ss.set(0, 1, "=1+-(1+2*10)").unwrap();
        assert_eq!(c2.out, -20.);
    }

    #[test]
    fn set_formula_with_ref() {
        let mut ss = Spreadsheet::new();
        ss.set(0, 0, "1").unwrap();
        ss.set(0, 1, "2").unwrap();
        ss.set(1, 0, "3").unwrap();
        ss.set(1, 1, "4").unwrap();
        let c = ss.set(2, 2, "=[0,0]+[0,1]+[1,0]+[1,1]").unwrap();
        assert_eq!(c.out, 10.);
    }
}
