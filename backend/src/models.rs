use super::schema::cells;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Queryable)]
pub struct Cell {
  pub id: i32,
  pub sheet_id: i32,
  pub row: i32,
  pub col: i32,
  pub raw: String,
}

#[derive(Insertable)]
#[table_name = "cells"]
pub struct NewCell {
  pub sheet_id: i32,
  pub row: i32,
  pub col: i32,
  pub raw: String,
}

#[derive(Queryable)]
pub struct Sheet {
  pub id: i32,
}
