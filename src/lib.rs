/*
Grammar

<valid cell> ::= <number>
                 | <formula>

<formula> ::= “=“ <expression>

<expression> ::= <value>
                 | <value> <operator> <expression>

<value> ::= <number>
            | <signed number>
            | <cell>
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
}

impl Operator {
    fn eval(&self, val1: f64, val2: f64) -> f64 {
        match self {
            Sum => val1 + val2,
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

fn eval_formula(input: &str) -> Result<f64, &str> {
    let (expr, _) = formula(input)?;
    let (first_val, rest_expr) = expr;
    if rest_expr.is_empty() {
        return Ok(first_val);
    }

    let mut expr_iter = rest_expr.into_iter().rev();
    let (last_op, last_val) = expr_iter.next().unwrap();
    let mut op = last_op;
    let mut res = last_val;
    for (next_op, next_val) in expr_iter {
        res = op.eval(next_val, res);
        op = next_op;
    }
    res = op.eval(first_val, res);
    Ok(res)
}

// Spreadsheet Parsers

fn formula(input: &str) -> ParseResult<(f64, Vec<(Operator, f64)>)> {
    let (_, input) = equal(input)?;
    let (res, input) = expression(input)?;
    if input.is_empty() {
        Ok((res, input))
    } else {
        Err("Expected input to be empty")
    }
}

fn equal(input: &str) -> ParseResult<()> {
    literal("=").parse(input)
}

fn expression(input: &str) -> ParseResult<(f64, Vec<(Operator, f64)>)> {
    let (first_val, input) = number(input)?;
    let (other_vals, input) = operator_and_value(input)?;
    Ok(((first_val, other_vals), input))
}

fn operator_and_value(input: &str) -> ParseResult<Vec<(Operator, f64)>> {
    zero_or_more(pair(operator, number)).parse(input)
}

fn number(input: &str) -> ParseResult<f64> {
    let (num_vec, input) = one_or_more(predicate(any_char, |c| c.is_numeric())).parse(input)?;
    let num = num_vec.into_iter().collect::<String>().parse().unwrap();
    Ok((num, input))
}

fn operator(input: &str) -> ParseResult<Operator> {
    if let Ok((_, input)) = literal("+").parse(input) {
        Ok((Operator::Sum, input))
    } else {
        Err("unknown operator")
    }
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
    fn eval_formula_with_numbers() {
        assert_eq!(eval_formula("=1+1+1"), Ok(3.));
        assert_eq!(eval_formula("=1+15+1"), Ok(17.));
        assert_eq!(eval_formula("=4"), Ok(4.));
    }

    #[test]
    fn parse_formula() {
        let (resp, input) = formula("=123").unwrap();
        assert_eq!(resp, (123.0, vec![]));
        assert_eq!(input, "");

        assert!(expression("=2+").is_err());
    }
    #[test]
    fn parse_equal() {
        let (_, rest) = equal("=a").unwrap();
        assert_eq!(rest, "a");
        assert!(equal("a=").is_err());
    }

    #[test]
    fn parse_expression() {
        let (resp, input) = expression("123").unwrap();
        assert_eq!(resp, (123.0, vec![]));
        assert_eq!(input, "");

        let (resp, input) = expression("1+2+3").unwrap();
        assert_eq!(
            resp,
            (1.0, vec![(Operator::Sum, 2.0), (Operator::Sum, 3.0)])
        );
        assert_eq!(input, "");

        let (resp, input) = expression("2+").unwrap();
        assert_eq!(resp, (2.0, vec![]));
        assert_eq!(input, "+");

        assert!(expression("+2+3").is_err());
    }

    #[test]
    fn parse_operator_and_value_number() {
        assert_eq!(
            operator_and_value("+123 "),
            Ok((vec![(Operator::Sum, 123.0)], " "))
        );
        assert_eq!(operator_and_value("123+"), Ok((vec![], "123+")));
    }

    #[test]
    fn parse_number_ok() {
        let (num, rest) = number("123 what").unwrap();
        assert_eq!(num, 123.0);
        assert_eq!(rest, " what");
    }

    #[test]
    fn parse_number_err() {
        assert!(number(" 123").is_err());
    }

    #[test]
    fn parse_sum_operator() {
        assert_eq!(operator("+ "), Ok((Operator::Sum, " ")));
        assert!(operator(" +").is_err());
    }
}
