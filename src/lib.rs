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
Reference ::= '(' Number ',' Number ')'
Number ::= Digit+
*/

struct Spreadsheet {
    grid: [[Cell; 100]; 100],
}

struct Cell {
    raw: String,
    out: f64,
}

#[derive(Debug, PartialEq)]
enum Operator {
    Sum,
    Sub,
    Mul,
    Div,
}

impl Operator {
    fn eval(&self, val1: f64, val2: f64) -> f64 {
        match self {
            Operator::Sum => val1 + val2,
            Operator::Sub => val1 - val2,
            Operator::Mul => val1 * val2,
            Operator::Div => val1 / val2,
        }
    }
}

impl Spreadsheet {
    fn set(&mut self, row: usize, col: usize, raw_val: &str) {
        todo!()
    }
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

fn formula(input: &str) -> ParseResult<f64> {
    let (_, input) = literal("=").parse(input)?;
    let (res, input) = expr(input)?;
    if input.is_empty() {
        Ok((res, input))
    } else {
        Err("Expected input to be empty")
    }
}

fn expr(input: &str) -> ParseResult<f64> {
    // Expr ::= Term ('+' Term | '-' Term)*
    let (first_term, input) = term(input)?;
    let sum = map(literal("+"), |_| Operator::Sum);
    let sub = map(literal("-"), |_| Operator::Sub);
    let sum_term = pair(sum, term);
    let sub_term = pair(sub, term);
    let (others, input) = zero_or_more(either(sum_term, sub_term)).parse(input)?;
    let e = eval(first_term, others);
    Ok((e, input))
}

fn term(input: &str) -> ParseResult<f64> {
    // Term ::= Factor ('*' Factor | '/' Factor)*
    let (first_factor, input) = factor(input)?;
    let mul = map(literal("*"), |_| Operator::Mul);
    let div = map(literal("/"), |_| Operator::Div);
    let mul_fact = pair(mul, factor);
    let div_fact = pair(div, factor);
    let (others, input) = zero_or_more(either(mul_fact, div_fact)).parse(input)?;
    let t = eval(first_factor, others);
    Ok((t, input))
}

fn factor(mut input: &str) -> ParseResult<f64> {
    // Factor ::= ['-'] (Number | '(' Expr ')')
    let mut coefficient = 1.;
    if let Ok((_, next_input)) = literal("-").parse(input) {
        coefficient = -1.;
        input = next_input;
    }
    let paren_expr = right(literal("("), left(expr, literal(")")));
    let val_or_expr = either(value, paren_expr);
    let (num, input) = val_or_expr.parse(input)?;
    Ok((coefficient * num, input))
}

fn value(input: &str) -> ParseResult<f64> {
    // Value ::= Reference | Function | Number
    either(reference, number).parse(input)
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
    Ok((val, input))
}

fn eval(first_val: f64, others: Vec<(Operator, f64)>) -> f64 {
    if others.is_empty() {
        return first_val;
    }

    let mut others_iter = others.into_iter().rev();
    let (last_op, last_val) = others_iter.next().unwrap();
    let mut op = last_op;
    let mut res = last_val;
    for (next_op, next_val) in others_iter {
        res = op.eval(next_val, res);
        op = next_op;
    }
    op.eval(first_val, res)
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
        assert_eq!(formula("=1+2*10-2"), Ok((19., "")));
        assert_eq!(formula("=1+-(1+2*10)"), Ok((-20., "")));
        assert_eq!(formula("=123+[1,1]"), Ok((223., "")));
    }

    fn formula_with_err() {
        assert!(formula("4").is_err());
    }
}
