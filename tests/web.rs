#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use spreadsheet::{Cell, Spreadsheet};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn set_evals_formula_with_numbers() {
  let mut ss = Spreadsheet::new();
  let r1 = 0;
  let c1 = 0;
  ss.set(r1, c1, "=1+2*10-2").unwrap();
  assert_eq!(ss.get(r1, c1).out(), 19.);

  let r2 = 0;
  let c2 = 1;
  ss.set(r2, c2, "=1+-(1+2*10)").unwrap();
  assert_eq!(ss.get(r2, c2).out(), -20.);
}

#[wasm_bindgen_test]
fn set_evals_formula_with_ref() {
  let mut ss = Spreadsheet::new();
  ss.set(0, 0, "1").unwrap();
  ss.set(0, 1, "2").unwrap();
  ss.set(1, 0, "3").unwrap();
  ss.set(1, 1, "4").unwrap();
  let r1 = 2;
  let c1 = 2;
  ss.set(r1, c1, "=[0,0]+[0,1]+[1,0]+[1,1]").unwrap();
  assert_eq!(ss.get(r1, c1).out(), 10.);
}

#[wasm_bindgen_test]
fn set_detects_ref_cycle() {
  let mut ss = Spreadsheet::new();
  ss.set(0, 0, "=[0,1]").unwrap();
  ss.set(1, 0, "=[1,0]").unwrap();
  assert!(ss.set(0, 1, "=[0,0]").is_err())
}

#[wasm_bindgen_test]
fn set_works_when_multiple_cells_ref_same_cell() {
  let mut ss = Spreadsheet::new();
  ss.set(0, 0, "1").unwrap();
  ss.set(0, 1, "=[0,0]*10").unwrap();
  ss.set(1, 1, "=[0,0]+[1,0]").unwrap();
  assert_eq!(ss.get(1, 1).out(), 11.);
}

#[wasm_bindgen_test]
fn set_evals_all_inbound() {
  let mut ss = Spreadsheet::new();
  ss.set(0, 0, "10").unwrap();
  ss.set(0, 1, "=[0,0]*2").unwrap();
  ss.set(1, 0, "=[0,0]*3").unwrap();
  ss.set(1, 1, "=[0,1]*4").unwrap();

  ss.set(0, 0, "1").unwrap();
  assert_eq!(ss.get(0, 0).out(), 1.);
  assert_eq!(ss.get(0, 1).out(), 2.);
  assert_eq!(ss.get(1, 0).out(), 3.);
  assert_eq!(ss.get(1, 1).out(), 12.);
}

#[wasm_bindgen_test]
fn set_returns_all_updated() {
  let mut ss = Spreadsheet::new();
  ss.set(0, 0, "10").unwrap();
  ss.set(0, 1, "=[0,0]*2").unwrap();
  ss.set(1, 0, "=[0,0]*3").unwrap();
  ss.set(1, 1, "=[0,1]*4").unwrap();

  let updates = ss.set(0, 0, "1").unwrap();
  let idx_to_cells: HashMap<usize, Cell> = JsValue::into_serde(&updates).unwrap();

  let idx1 = ss.get_index(0, 0);
  assert_eq!(idx_to_cells.get(&idx1).unwrap().out(), 1.);
  let idx2 = ss.get_index(0, 1);
  assert_eq!(idx_to_cells.get(&idx2).unwrap().out(), 2.);
  let idx3 = ss.get_index(1, 0);
  assert_eq!(idx_to_cells.get(&idx3).unwrap().out(), 3.);
  let idx4 = ss.get_index(1, 1);
  assert_eq!(idx_to_cells.get(&idx4).unwrap().out(), 12.);
}
