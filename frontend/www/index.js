import { Spreadsheet } from "spreadsheet";

const ss = Spreadsheet.new();
const cells = ss.cells();
const width = ss.width();
const height = ss.height();

const getIndex = (row, col) => {
  return row * width + col;
};

const setInputValue = (inputEle, cell) => {
  if (document.activeElement === inputEle) {
    inputEle.value = cell.raw;
  } else {
    inputEle.value = cell.raw.length === 0 ? "" : cell.out;
  }
};

const updateCell = (index, raw, out) => {
  const cell = cells[index]
  cell.raw = raw;
  cell.out = out;
  const inputEle = document.getElementById(`input-${index}`);
  setInputValue(inputEle, cell);
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

const colToLetters = (col) => {
  const base = 26;
  let remainders = [];

  remainders.push(col % base);
  let quotient = Math.floor(col / base);

  while (quotient !== 0) {
    remainders.push(quotient % base);
    quotient = Math.floor(quotient / base);
  }

  const asciiOffset = "A".charCodeAt(0);
  const asciiCode = remainders.map((n) => {
    return asciiOffset + n;
  }).reverse();

  return String.fromCharCode(asciiCode);
};

// UI set up
const tableEle = document.getElementById("table");
const headerRowEle = document.createElement("tr");

const paddingCol = document.createElement("td");
paddingCol.className = "cell-header";
headerRowEle.appendChild(paddingCol);

for (let i = 0; i < width; i++) {
  const colEle = document.createElement("td");
  colEle.innerHTML = colToLetters(i);
  colEle.className = "cell-header";
  headerRowEle.appendChild(colEle);
}
tableEle.appendChild(headerRowEle);

for (let i = 0; i < height; i++) {
  const rowEle = document.createElement("tr");
  tableEle.appendChild(rowEle);
  
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
    inputEle.className = "cell-input";
    setInputValue(inputEle, cell);

    inputEle.addEventListener("focus", (event) => {
      inputEle.value = cell.raw;
    });
    inputEle.addEventListener("blur", (event) => {
      if (inputEle.value !== cell.raw) {
        const updates = ss.set(i, j, inputEle.value);
        updateCells(updates);
      } else {
        setInputValue(inputEle, cell);
      }
    });
    inputEle.addEventListener("keydown", (event) => {
      if (event.key === "Enter") {
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
}


