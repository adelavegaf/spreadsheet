struct Spreadsheet {
    grid: Vec<Vec<Value>>,
}
#[derive(Debug, PartialEq, Clone)]
enum Value {
    Num(f64),
    Text(String),
    Formula(String),
    Empty,
}

impl Spreadsheet {
    fn new() -> Spreadsheet {
        Spreadsheet {
            grid: vec![vec![Value::Empty; 100]; 100],
        }
    }

    fn set(&mut self, row: usize, col: usize, raw_val: &str) {
        self.grid[row][col] = parse_val(raw_val);
    }

    fn eval(&self, row: usize, col: usize) -> String {
        match &self.grid[row][col] {
            Value::Num(n) => n.to_string(),
            Value::Text(s) => s.to_string(),
            Value::Formula(f) => f.to_string(),
            Value::Empty => "".to_string(),
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

    mod spreadsheet {
        use super::*;
        #[test]
        fn new_creates_empty_grid() {
            let s = Spreadsheet::new();
            for row in &s.grid {
                for val in row {
                    assert_eq!(val, &Value::Empty);
                }
            }
            assert_eq!(s.grid.len(), 100);
            assert_eq!(s.grid[0].len(), 100);
        }

        #[test]
        fn set_parses_formula() {
            let mut s = Spreadsheet::new();
            let row = 12;
            let col = 33;
            s.set(row, col, "=1+1");
            assert_eq!(s.grid[row][col], Value::Formula("1+1".to_string()));
        }

        #[test]
        fn set_parses_num() {
            let mut s = Spreadsheet::new();
            let row = 1;
            let col = 44;
            s.set(row, col, "1.78");
            assert_eq!(s.grid[row][col], Value::Num(1.78));
        }

        #[test]
        fn set_parses_text() {
            let mut s = Spreadsheet::new();
            let row = 1;
            let col = 44;
            s.set(row, col, "1.78a");
            assert_eq!(s.grid[row][col], Value::Text("1.78a".to_string()));
        }
    }
}
