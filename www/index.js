import { Spreadsheet } from "spreadsheet";

const ss = Spreadsheet.new();
const cells = ss.cells();
const width = ss.width();
const height = ss.height();

const getIndex = (row, col) => {
  return row * width + col;
};

const updateCell = (index, raw, out) => {
  cells[index].raw = raw;
  cells[index].out = out;
  const inputEle = document.getElementById(`input-${index}`);
  if (document.activeElement === inputEle) {
    inputEle.value = raw;
  } else {
    inputEle.value = out;
  }
};

const updateCells = (updates) => {
  for (const [idx, cell] of Object.entries(updates)) {
    updateCell(idx, cell.raw, cell.out);
  }
};

const focusInput = (row, col) => {
  row = Math.min(Math.max(row, 0), height - 1);
  col = Math.min(Math.max(col, 0), width - 1);
  const idx = getIndex(row, col);
  const input = document.getElementById(`input-${idx}`);
  input.focus();
}

// UI set up
const tableEle = document.getElementById("table");
const headerRowEle = document.createElement("tr");

const paddingCol = document.createElement("td");
paddingCol.className = "cell-header";
headerRowEle.appendChild(paddingCol);

for (let i = 0; i < width; i++) {
  const colEle = document.createElement("td");
  colEle.innerHTML = i;
  colEle.className = "cell-header";
  headerRowEle.appendChild(colEle);
}
tableEle.appendChild(headerRowEle);

for (let i = 0; i < height; i++) {
  const rowEle = document.createElement("tr");
  
  const headerCol = document.createElement("td");
  headerCol.className = "cell-header";
  headerCol.innerHTML = i;
  rowEle.appendChild(headerCol);

  for (let j = 0; j < width; j++) {
    const idx = getIndex(i, j);
    const cell = cells[idx];

    const colEle = document.createElement("td");
    colEle.setAttribute("id", `cell-${idx}`);
    rowEle.appendChild(colEle);
    colEle.className = "cell";
    
    const inputEle = document.createElement("input");
    inputEle.setAttribute("id", `input-${idx}`);
    inputEle.value = cell.out;
    inputEle.className = "cell-input";

    inputEle.addEventListener("focus", (event) => {
      inputEle.value = cell.raw;
    });
    inputEle.addEventListener("blur", (event) => {
      inputEle.value = cell.out;
    });
    inputEle.addEventListener("keydown", (event) => {
      if (event.key === "Enter") {
        const updates = ss.set(i, j, inputEle.value);
        updateCells(updates);
        focusInput(i+1, j);
      } else if (event.key === "Escape") {
        inputEle.value = cell.out;
      } else if (event.key === "ArrowRight") {
        focusInput(i, j+1);
      } else if (event.key === "ArrowLeft") {
        focusInput(i, j-1);
      } else if (event.key === "ArrowUp") {
        focusInput(i-1, j);
      } else if (event.key === "ArrowDown") {
        focusInput(i+1, j);
      }
    });
    
    colEle.appendChild(inputEle);
  }
  tableEle.appendChild(rowEle);
}


