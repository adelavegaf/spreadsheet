/*
Formula Grammar

Cell ::= Number | Formula
Formula ::= “=“ Expr

https://stackoverflow.com/questions/9785553/how-does-a-simple-calculator-with-parentheses-work
Expr ::= Term ('+' Term | '-' Term)*
Term ::= Factor ('*' Factor | '/' Factor)*
Factor ::= ['-'] (Value | '(' Expr ')')
Value ::= Function | Reference | Number
Formula ::= FnId '(' Range ')'
Range ::= Reference '->' Reference
Reference ::= '[' Number ',' Number ']'
Number ::= Digit+
*/

enum ExprTree {
    Empty,
    Value(f64),
    Unary(Box<UnaryNode>),
    Binary(Box<BinaryNode>),
}

struct UnaryNode {
    op: UnaryOp,
    child: ExprTree,
}

struct BinaryNode {
    op: BinaryOp,
    left: ExprTree,
    right: ExprTree,
}

struct Spreadsheet {
    grid: [[Cell; 100]; 100],
}

struct Cell {
    raw: String,
    out: f64,
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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
    fn set(&mut self, row: usize, col: usize, raw: String) -> Result<(), &str> {
        if self.grid.len() <= row || self.grid[0].len() <= col {
            return Err("out of bounds");
        }
        let cell = &mut self.grid[row][col];
        cell.raw = raw;
        Ok(())
    }

    // TODO: Lets move parsing code into the spreadsheet implementation to have
    // access to mut self. This way we can set references as we are parsing.
}

type ParseResult<'a, Output> = Result<(Output, &'a str), &'a str>;

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

fn formula(input: &str) -> ParseResult<ExprTree> {
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
    let (num, input) = number(input)?;
    Ok((ExprTree::Value(num), input))
}

fn reference(input: &str) -> ParseResult<f64> {
    // Reference ::= '[' Number ',' Number ']'
    let (_, input) = literal("[").parse(input)?;
    let (row, input) = number(input)?;
    let (_, input) = literal(",").parse(input)?;
    let (col, input) = number(input)?;
    let (_, input) = literal("]").parse(input)?;
    // At this point we should grab the contents of the cell at row, col and
    let (val, _) = formula("=100")?;
    todo!();
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

fn eval(tree: ExprTree) -> f64 {
    match tree {
        ExprTree::Value(val) => val,
        ExprTree::Unary(u) => u.op.apply(eval(u.child)),
        ExprTree::Binary(b) => b.op.apply(eval(b.left), eval(b.right)),
        ExprTree::Empty => panic!("Found empty tree node"),
    }
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
    fn formula_with_numbers() {
        let (expr, _) = formula("=1+2*10-2").unwrap();
        assert_eq!(eval(expr), 19.);
        let (expr, _) = formula("=1+-(1+2*10)").unwrap();
        assert_eq!(eval(expr), -20.);
    }
}
