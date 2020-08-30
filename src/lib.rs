mod parser;
use parser::{cell, ExprTree, ValueNode};
use std::collections::HashSet;
use std::mem;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Coord {
    row: usize,
    col: usize,
}

#[derive(Clone)]
pub struct Cell {
    raw: String,
    expr: ExprTree,
    out: f64,
    outbound: HashSet<Coord>,
    inbound: HashSet<Coord>,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            raw: "".to_string(),
            expr: ExprTree::Empty,
            out: 0.,
            outbound: HashSet::new(),
            inbound: HashSet::new(),
        }
    }
}

impl Cell {
    fn new() -> Cell {
        Default::default()
    }
}

pub struct Spreadsheet {
    grid: Vec<Vec<Cell>>,
}

impl Default for Spreadsheet {
    fn default() -> Self {
        Spreadsheet {
            grid: vec![vec![Cell::new(); 100]; 100],
        }
    }
}

impl Spreadsheet {
    pub fn new() -> Spreadsheet {
        Default::default()
    }

    pub fn set(&mut self, row: usize, col: usize, raw: &str) -> Result<&Cell, &str> {
        if self.grid[0].len() <= col {
            self.resize_cols(col * 2);
        }
        if self.grid.len() <= row {
            self.resize_rows(row * 2);
        }

        let cur_coord = Coord { row, col };

        // Replace old cell with a placeholder to deal with expired inbound references
        let old_cell = mem::replace(&mut self.grid[row][col], Cell::new());
        self.rm_from_inbound(cur_coord, &old_cell.outbound);

        // Create new cell
        let (expr, _) = cell(raw)?;
        let out = eval(self, &expr);
        let outbound = outbound(&expr);
        let inbound = old_cell.inbound.clone();
        let new_cell = Cell {
            raw: raw.to_string(),
            expr,
            out,
            outbound,
            inbound,
        };

        // Replace placeholder with new cell and add new inbound references
        self.add_to_inbound(cur_coord, &new_cell.outbound);
        self.grid[row][col] = new_cell;

        if self.has_cycle(cur_coord) {
            self.grid[row][col] = old_cell;
            return Err("This cell introduces a cycle!");
        }

        // Our references form a DAG, we can toposort it to have the correct
        // order we should re-eval our dependencies.
        let eval_order = self.toposort_inbound(cur_coord);
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

    fn rm_from_inbound(&mut self, c: Coord, targets: &HashSet<Coord>) {
        for t in targets.iter() {
            self.grid[t.row][t.col].inbound.remove(&c);
        }
    }

    fn add_to_inbound(&mut self, c: Coord, targets: &HashSet<Coord>) {
        for t in targets.iter() {
            self.grid[t.row][t.col].inbound.insert(c);
        }
    }

    fn has_cycle(&self, start: Coord) -> bool {
        self._has_cycle(start, &mut HashSet::new())
    }

    fn _has_cycle(&self, start: Coord, visited: &mut HashSet<Coord>) -> bool {
        if !visited.insert(start) {
            return true;
        }
        for r in self.grid[start.row][start.col].outbound.iter() {
            if self._has_cycle(*r, visited) {
                return true;
            }
        }
        false
    }

    fn toposort_inbound(&self, start: Coord) -> Vec<Coord> {
        let mut sorted = vec![];
        self._toposort_inbound(start, &mut sorted);
        sorted
    }

    fn _toposort_inbound(&self, start: Coord, result: &mut Vec<Coord>) {
        result.push(start);
        for r in self.grid[start.row][start.col].inbound.iter() {
            self._toposort_inbound(*r, result);
        }
    }
}

fn eval(ss: &Spreadsheet, tree: &ExprTree) -> f64 {
    match tree {
        ExprTree::Leaf(ValueNode::Num(n)) => *n,
        ExprTree::Leaf(ValueNode::Coord(row, col)) => ss.grid[*row][*col].out,
        ExprTree::Unary(u) => u.op.apply(eval(ss, &u.child)),
        ExprTree::Binary(b) => b.op.apply(eval(ss, &b.left), eval(ss, &b.right)),
        ExprTree::Empty => panic!("Found empty tree node"),
    }
}

fn outbound(tree: &ExprTree) -> HashSet<Coord> {
    match tree {
        ExprTree::Leaf(ValueNode::Num(_)) => HashSet::new(),
        ExprTree::Leaf(ValueNode::Coord(row, col)) => [Coord {
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
            ss.grid[0][0].inbound,
            vec![Coord { row: 1, col: 1 }].into_iter().collect()
        );
        assert_eq!(ss.grid[0][0].outbound, HashSet::new());

        assert_eq!(ss.grid[0][1].inbound, HashSet::new());
        assert_eq!(ss.grid[0][1].outbound, HashSet::new());

        assert_eq!(ss.grid[1][1].inbound, HashSet::new());
        assert_eq!(
            ss.grid[1][1].outbound,
            vec![Coord { row: 0, col: 0 }].into_iter().collect()
        );
    }
}
