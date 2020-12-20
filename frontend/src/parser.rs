use super::expr::{BinaryNode, BinaryOp, ExprTree, UnaryNode, UnaryOp, ValueNode};
/*
Grammar

Cell ::= Formula | Rational Number | Text
Formula ::= “=“ Expr
Expr ::= Term ('+' Term | '-' Term)*
Term ::= Factor ('*' Factor | '/' Factor)*
Factor ::= ['-'] (Value | '(' Expr ')')
Value ::= Function | Coordinate | Rational Number
Function ::= FnId '(' Range ')' (TODO: implement)
Range ::= Coordinate ':' Coordinate (TODO: implement)
Coordinate ::= Letters Natural Number
Letters ::= Letter+
Natural Number ::= Digit+
Rational Number ::= [-] Digit+ ['.' Digit+] (TODO: check if adding [-] breaks Factor)
Digit ::= [0-9]
Letter ::= [a-z][A-Z]

TODO:
- Improve error handling while parsing. Ideally, we would get "unexpected token in line x col y, found: w expected z"
  - This includes passing ExprResult::Error instead of a standard rust error.
- Test for parsers
*/
pub type ParseResult<'a, Output> = Result<(Output, &'a str), &'static str>;

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

pub fn cell(input: &str) -> ParseResult<ExprTree> {
  // Cell ::= Formula | Number | Text
  let (tree, input) = if input.starts_with('=') {
    formula(input)?
  } else {
    // Order matters, we only want to treat something as text if it's not a number.
    either(num, text).parse(input)?
  };
  if !input.is_empty() {
    Err("Expected input to be empty")
  } else {
    Ok((tree, input))
  }
}

fn num(input: &str) -> ParseResult<ExprTree> {
  empty_or_err(map(rational_number, |n| ExprTree::Leaf(ValueNode::Num(n)))).parse(input)
}

fn text(input: &str) -> ParseResult<ExprTree> {
  let multiple_chars = zero_or_more(any_char);
  empty_or_err(map(multiple_chars, |c| {
    ExprTree::Leaf(ValueNode::Text(c.into_iter().collect()))
  }))
  .parse(input)
}

fn formula(input: &str) -> ParseResult<ExprTree> {
  // Formula ::= “=“ Expr
  let (_, input) = literal("=").parse(input)?;
  empty_or_err(expr).parse(input)
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
  let num_val = map(rational_number, ValueNode::Num);
  let num_or_coord = either(num_val, coord);
  let (val, input) = num_or_coord.parse(input)?;
  Ok((ExprTree::Leaf(val), input))
}

fn coord(input: &str) -> ParseResult<ValueNode> {
  // Coordinate ::= Letters Number
  let (ltrs, input) = letters(input)?;
  let col = letters_to_col(&ltrs);
  // TODO(adelavega): We should have a float, and int parser, and use int here.
  let (num, input) = natural_number(input)?;
  // Convert to 0-based index before returning
  let row = (num as usize) - 1;
  Ok((ValueNode::Coord(row, col), input))
}

fn letters(input: &str) -> ParseResult<String> {
  // Letters ::= Letter+
  let (letters_vec, input) =
    one_or_more(predicate(any_char, |c| c.is_ascii_alphabetic())).parse(input)?;
  let ltrs = letters_vec.into_iter().collect::<String>();
  Ok((ltrs, input))
}

fn letters_to_col(letters: &str) -> usize {
  let start = b'a';
  let mut col = 0;
  let base: usize = 26;
  for (i, c) in letters.to_lowercase().bytes().rev().enumerate() {
    let num = (c - start + 1) as usize;
    col += num * base.pow(i as u32);
  }
  // 0-indexed result
  col - 1
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

fn rational_number(input: &str) -> ParseResult<f64> {
  // Rational Number := [-] Digit+ [. Digit+]
  let (negate_opt, input) = optional(literal("-")).parse(input)?;
  let neg_coefficient = if negate_opt.is_some() { -1. } else { 1. };
  let (first_num, input) = digits(input)?;
  let (dot_opt, input) = optional(literal(".")).parse(input)?;
  let (full_num, input) = if dot_opt.is_some() {
    let (second_num, input) = digits(input)?;
    (format!("{}.{}", first_num, second_num), input)
  } else {
    (first_num, input)
  };
  let rational_num = full_num.parse::<f64>().unwrap() * neg_coefficient;
  Ok((rational_num, input))
}

fn natural_number(input: &str) -> ParseResult<f64> {
  // Number ::= Digit+
  let (num, input) = digits(input)?;
  Ok((num.parse().unwrap(), input))
}

fn digits(input: &str) -> ParseResult<String> {
  let (num_vec, input) = one_or_more(predicate(any_char, |c| c.is_numeric())).parse(input)?;
  Ok((num_vec.into_iter().collect::<String>(), input))
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

fn empty_or_err<'a, A>(parser: impl Parser<'a, A>) -> impl Parser<'a, A> {
  move |input: &'a str| {
    let (res, input) = parser.parse(input)?;
    if input.is_empty() {
      Ok((res, ""))
    } else {
      Err("expected input to be empty")
    }
  }
}

fn optional<'a, A>(parser: impl Parser<'a, A>) -> impl Parser<'a, Option<A>> {
  move |input: &'a str| {
    if let Ok((a, input)) = parser.parse(input) {
      Ok((Some(a), input))
    } else {
      Ok((None, input))
    }
  }
}

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
      // TODO: improve error message, this masks the actual error.
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
  mod parsers {
    use super::super::*;
    #[test]
    fn letters_to_col_smoketest() {
      assert_eq!(letters_to_col("A"), 0);
      assert_eq!(letters_to_col("z"), 25);
      assert_eq!(letters_to_col("Aa"), 26);
      assert_eq!(letters_to_col("ba"), 52);
    }
  }
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

    #[test]
    fn optional_smoketest() {
      let p = optional(literal("a"));
      assert_eq!(p.parse("abc"), Ok((Some(()), "bc")));
      assert_eq!(p.parse("bc"), Ok((None, "bc")));
    }

    #[test]
    fn empty_or_err_smoketest() {
      let p = empty_or_err(literal("a"));
      assert_eq!(p.parse("a"), Ok(((), "")));
      assert!(p.parse("ab").is_err());
    }
  }
}
