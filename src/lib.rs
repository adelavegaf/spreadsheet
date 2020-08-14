struct Spreadsheet {
    grid: Vec<Vec<Value>>,
}
#[derive(Debug, PartialEq)]
enum Value {
    Num(f64),
    Text(String),
    Formula(String),
}

impl Spreadsheet {
    fn set(&mut self, row: usize, col: usize, raw_val: String) {
        self.grid[row][col] = parse_val(&raw_val);
    }

    fn eval(&self, row: usize, col: usize) -> String {
        match &self.grid[row][col] {
            Value::Num(n) => n.to_string(),
            Value::Text(s) => s.to_string(),
            Value::Formula(_f) => "parse_formula".to_string(),
        }
    }
}

fn parse_val(input: &str) -> Value {
    // TODO: if we are going for performance, all the String conversions are going
    // to cost us.
    match input {
        i if i.starts_with('=') => Value::Formula(i['='.len_utf8()..].to_string()),
        i if i.parse::<f64>().is_ok() => Value::Num(i.parse().unwrap()),
        _ => Value::Text(input.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_val_returns_formula() {
        assert_eq!(parse_val("=1+1"), Value::Formula("1+1".to_string()));
    }

    #[test]
    fn parse_val_returns_num() {
        assert_eq!(parse_val("1"), Value::Num(1.0));
        assert_eq!(parse_val("1.78"), Value::Num(1.78));
    }

    #[test]
    fn parse_val_returns_text() {
        assert_eq!(parse_val("1123.a"), Value::Text("1123.a".to_string()));
        assert_eq!(parse_val(" ="), Value::Text(" =".to_string()));
    }
}
