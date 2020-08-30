/*
TODO:
- Improve error handling while parsing. Ideally, we would get "unexpected token in line x col y, found: w expected z"
- Rename Ref to something else since there's a ref primitive in rust. CellRef?
- Test for parsers
- Test for combinators
*/
mod parser;
use parser::{cell, ExprTree, ValueNode};
use std::collections::HashSet;
use std::mem;

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
        self.rm_from_inbound(cur_ref, &old_cell.outbound_refs);

        // Create new cell
        let (expr, _) = cell(raw)?;
        let out = eval(self, &expr);
        let outbound_refs = outbound(&expr);
        let inbound_refs = old_cell.inbound_refs.clone();
        let new_cell = Cell {
            raw: raw.to_string(),
            expr,
            out,
            outbound_refs,
            inbound_refs,
        };

        // Replace placeholder with new cell and add new inbound references
        self.add_to_inbound(cur_ref, &new_cell.outbound_refs);
        self.grid[row][col] = new_cell;

        if self.has_cycle(cur_ref) {
            self.grid[row][col] = old_cell;
            return Err("This cell introduces a cycle!");
        }

        // Our references form a DAG, we can toposort it to have the correct
        // order we should re-eval our dependencies.
        let eval_order = self.toposort_inbound(cur_ref);
        for r in eval_order {
            let new_out = eval(self, &self.grid[r.row][r.col].expr);
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

    fn rm_from_inbound(&mut self, r: Ref, targets: &HashSet<Ref>) {
        for t in targets.iter() {
            self.grid[t.row][t.col].inbound_refs.remove(&r);
        }
    }

    fn add_to_inbound(&mut self, r: Ref, targets: &HashSet<Ref>) {
        for t in targets.iter() {
            self.grid[t.row][t.col].inbound_refs.insert(r);
        }
    }

    fn has_cycle(&self, start: Ref) -> bool {
        self._has_cycle(start, &mut HashSet::new())
    }

    fn _has_cycle(&self, start: Ref, visited: &mut HashSet<Ref>) -> bool {
        if !visited.insert(start) {
            return true;
        }
        for r in self.grid[start.row][start.col].outbound_refs.iter() {
            if self._has_cycle(*r, visited) {
                return true;
            }
        }
        false
    }

    fn toposort_inbound(&self, start: Ref) -> Vec<Ref> {
        let mut sorted = vec![];
        self._toposort_inbound(start, &mut sorted);
        sorted
    }

    fn _toposort_inbound(&self, start: Ref, result: &mut Vec<Ref>) {
        result.push(start);
        for r in self.grid[start.row][start.col].inbound_refs.iter() {
            self._toposort_inbound(*r, result);
        }
    }
}

fn eval(ss: &Spreadsheet, tree: &ExprTree) -> f64 {
    match tree {
        ExprTree::Leaf(ValueNode::Num(n)) => *n,
        ExprTree::Leaf(ValueNode::Ref(row, col)) => ss.grid[*row][*col].out,
        ExprTree::Unary(u) => u.op.apply(eval(ss, &u.child)),
        ExprTree::Binary(b) => b.op.apply(eval(ss, &b.left), eval(ss, &b.right)),
        ExprTree::Empty => panic!("Found empty tree node"),
    }
}

fn outbound(tree: &ExprTree) -> HashSet<Ref> {
    match tree {
        ExprTree::Leaf(ValueNode::Num(_)) => HashSet::new(),
        ExprTree::Leaf(ValueNode::Ref(row, col)) => [Ref {
            row: *row,
            col: *col,
        }]
        .iter()
        .copied()
        .collect(),
        ExprTree::Unary(u) => outbound(&u.child),
        ExprTree::Binary(b) => {
            let mut left = outbound(&b.left);
            let right = outbound(&b.right);
            left.extend(right);
            left
        }
        ExprTree::Empty => panic!("Found empty tree node"),
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
