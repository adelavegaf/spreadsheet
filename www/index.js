import { Spreadsheet } from "spreadsheet";

const ss = Spreadsheet.new();
const width = ss.width();
const height = ss.height();

// We only need this during initialization
const cells = ss.cells();

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

// UI set up
const tableEle = document.getElementById("table");
for (let i = 0; i < height; i++) {
  const rowEle = document.createElement("tr");
  for (let j = 0; j < width; j++) {
    const idx = getIndex(i, j);
    const cell = cells[idx];

    const colEle = document.createElement("td");
    colEle.setAttribute("id", `cell-${idx}`);
    rowEle.appendChild(colEle);
    
    const inputEle = document.createElement("input");
    inputEle.setAttribute("id", `input-${idx}`);
    inputEle.value = cell.out;

    inputEle.addEventListener("focus", (event) => {
      inputEle.value = cell.raw;
    });
    inputEle.addEventListener("blur", (event) => {
      inputEle.value = cell.out;
    });
    inputEle.addEventListener("keyup", (event) => {
      if (event.key === "Enter") {
        const updates = ss.set(i, j, inputEle.value);
        updateCells(updates);

        const bottomCellIdx = getIndex(i+1, j);
        const bottomInput = document.getElementById(`input-${bottomCellIdx}`);
        bottomInput.focus();
      } else if (event.key === "Escape") {
        inputEle.value = cell.out;
      }
    });
    
    colEle.appendChild(inputEle);
  }
  tableEle.appendChild(rowEle);
}


