mod parser;
use parser::{cell, ExprTree, ValueNode};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::mem;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Clone, Serialize, Deserialize)]
pub struct Cell {
    raw: String,
    out: f64,
    #[serde(skip)]
    expr: ExprTree,
    #[serde(skip)]
    outbound: HashSet<usize>,
    #[serde(skip)]
    inbound: HashSet<usize>,
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

    pub fn out(&self) -> f64 {
        self.out
    }
}

#[wasm_bindgen]
pub struct Spreadsheet {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

impl Default for Spreadsheet {
    fn default() -> Self {
        let width = 26;
        let height = 100;
        Spreadsheet {
            width,
            height,
            cells: vec![Cell::new(); width * height],
        }
    }
}

#[wasm_bindgen]
impl Spreadsheet {
    pub fn new() -> Spreadsheet {
        Default::default()
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn cells(&self) -> Result<JsValue, JsValue> {
        // This is expensive and should only be called to initialize the frontend.
        let serialized_cells = JsValue::from_serde(&self.cells)
            .map_err(|_| JsValue::from("could not serialize cells"))?;
        Ok(serialized_cells)
    }

    pub fn set(&mut self, row: usize, col: usize, raw: &str) -> Result<JsValue, JsValue> {
        if col >= self.width {
            return Err(JsValue::from("column out of bounds"));
        }
        if row >= self.height {
            return Err(JsValue::from("row out of bounds"));
        }

        let cur_idx = self.get_index(row, col);

        // Replace old cell with a placeholder to deal with expired inbound references
        let old_cell = mem::replace(&mut self.cells[cur_idx], Cell::new());
        for out_idx in &old_cell.outbound {
            self.cells[*out_idx].inbound.remove(&cur_idx);
        }

        // Create new cell
        let (expr, _) = cell(raw)?;
        let out = eval(self, &expr);
        let mut outbound = HashSet::new();
        fill_outbound(self, &expr, &mut outbound);
        let inbound = old_cell.inbound.clone();
        let new_cell = Cell {
            raw: raw.to_string(),
            expr,
            out,
            outbound,
            inbound,
        };

        // Add new inbound references and replace placeholder with new cell
        for out_idx in &new_cell.outbound {
            self.cells[*out_idx].inbound.insert(cur_idx);
        }
        self.cells[cur_idx] = new_cell;

        if self.has_cycle(cur_idx) {
            self.cells[cur_idx] = old_cell;
            return Err(JsValue::from("This cell introduces a cycle!"));
        }

        // Our references form a DAG, we can toposort it to have the correct
        // order we should re-eval our dependencies.
        let eval_order = self.toposort_inbound(cur_idx);
        for in_idx in &eval_order {
            let new_out = eval(self, &self.cells[*in_idx].expr);
            self.cells[*in_idx].out = new_out;
        }

        // Serialize all cells that were modified for frontend to update.
        let mut idx_to_cell = HashMap::new();
        for in_idx in &eval_order {
            idx_to_cell.insert(*in_idx, &self.cells[*in_idx]);
        }
        let res =
            JsValue::from_serde(&idx_to_cell).map_err(|_| JsValue::from("could not serialize"))?;
        Ok(res)
    }

    pub fn get_index(&self, row: usize, col: usize) -> usize {
        row * self.width + col
    }

    fn has_cycle(&self, start: usize) -> bool {
        self._has_cycle(start, &mut HashSet::new())
    }

    fn _has_cycle(&self, start: usize, visited: &mut HashSet<usize>) -> bool {
        if !visited.insert(start) {
            return true;
        }
        for r in self.cells[start].outbound.iter() {
            if self._has_cycle(*r, visited) {
                return true;
            }
        }
        false
    }

    fn toposort_inbound(&self, start: usize) -> Vec<usize> {
        let mut sorted = vec![];
        self._toposort_inbound(start, &mut sorted);
        sorted
    }

    fn _toposort_inbound(&self, start: usize, result: &mut Vec<usize>) {
        result.push(start);
        for r in self.cells[start].inbound.iter() {
            self._toposort_inbound(*r, result);
        }
    }
}

// methods not exported through web assembly
impl Spreadsheet {
    pub fn get(&self, row: usize, col: usize) -> &Cell {
        let idx = self.get_index(row, col);
        &self.cells[idx]
    }
}

fn eval(ss: &Spreadsheet, tree: &ExprTree) -> f64 {
    match tree {
        ExprTree::Leaf(ValueNode::Num(n)) => *n,
        ExprTree::Leaf(ValueNode::Coord(row, col)) => ss.get(*row, *col).out,
        ExprTree::Unary(u) => u.op.apply(eval(ss, &u.child)),
        ExprTree::Binary(b) => b.op.apply(eval(ss, &b.left), eval(ss, &b.right)),
        ExprTree::Empty => panic!("Found empty tree node"),
    }
}

fn fill_outbound(ss: &Spreadsheet, tree: &ExprTree, outbound: &mut HashSet<usize>) {
    match tree {
        ExprTree::Leaf(ValueNode::Num(_)) => (),
        ExprTree::Leaf(ValueNode::Coord(row, col)) => {
            outbound.insert(ss.get_index(*row, *col));
        }
        ExprTree::Unary(u) => fill_outbound(ss, &u.child, outbound),
        ExprTree::Binary(b) => {
            fill_outbound(ss, &b.left, outbound);
            fill_outbound(ss, &b.right, outbound);
        }
        ExprTree::Empty => panic!("Found empty tree node"),
    }
}
