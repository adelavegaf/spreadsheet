CREATE TABLE sheets (
  id SERIAL PRIMARY KEY
);

CREATE TABLE cells (
  id SERIAL PRIMARY KEY,
  sheet_id INT NOT NULL,
  row INT NOT NULL,
  col INT NOT NULL,
  raw VARCHAR NOT NULL,
  UNIQUE(sheet_id, row, col)
);