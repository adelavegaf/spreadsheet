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

fn formula(input: &str) -> ParseResult<&str> {
    let (_, input) = equal(input)?;
    // let (_, input) = parse_expression(input)?;
    todo!()
}

fn equal(input: &str) -> ParseResult<()> {
    literal("=").parse(input)
}

fn number(input: &str) -> ParseResult<f64> {
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
    fn parse_equal_ok() {
        let (_, rest) = equal("=a").unwrap();
        assert_eq!(rest, "a");
    }

    #[test]
    fn parse_equal_err() {
        assert!(equal("a=").is_err());
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
}
