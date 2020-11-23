import { memory } from "spreadsheet/spreadsheet_bg";
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
  const inputEle = document.getElementById(`input-${index}`);
  inputEle.value = raw;
  const outEle = document.getElementById(`out-${index}`);
  outEle.innerHTML = out;
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
    inputEle.value = cell.raw;
    colEle.appendChild(inputEle);
    
    const outEle = document.createElement("p");
    outEle.setAttribute("id", `out-${idx}`);
    outEle.innerHTML = cell.out;
    colEle.appendChild(outEle);

    colEle.addEventListener("keypress", (event) => {
      if (event.key != "Enter") {
        return;
      }
      const updates = ss.set(i, j, inputEle.value);
      updateCells(updates);
    });
  }
  tableEle.appendChild(rowEle);
}


