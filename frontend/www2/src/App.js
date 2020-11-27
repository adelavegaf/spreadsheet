import { useEffect, useRef, useState } from "react";
import logo from './logo.svg';
import './App.css';
import { Spreadsheet } from "spreadsheet";

const App = () => {
  const ss = Spreadsheet.new();
  const initialCells = ss.cells();
  const width = ss.width();
  const height = ss.height();

  const [cells, setCells] = useState(initialCells);

  const onCellFocus = (row, col, out) => {};

  const onCellBlur = (row, col, raw) => {
    const idx = getCellIndex(row, col, width);
    if (raw === cells[idx].raw) {
      return;
    }
    const updates = ss.set(row, col, raw);
    setCells(prevCells => {
      const newCells = [...prevCells];
      for (const [idx, cell] of Object.entries(updates)) {
        newCells[idx] = cell;
      }
      return newCells;
    });
  };

  const rows = range(height).map(row => {
    return <TableRow key={`row-${row}`} row={row} width={width} cells={cells} onCellFocus={onCellFocus} onCellBlur={onCellBlur}/>
  });

  return (
    <table id="table" cellSpacing="0">
      <TableHeader width={width}/>
      <tbody>
        {rows}
      </tbody>
    </table>
  );
};

const TableHeader = ({width}) => {
  return (
    <thead>
      <tr>
        <td className="cell-header"/>
        {
          range(width).map(idx => <td key={`header-${idx}`} className="cell-header">{colToLetters(idx)}</td>)
        }
      </tr>
    </thead>
  )
};

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

const TableRow = ({row, width, cells, onCellFocus, onCellBlur}) => {
  const tableCells = range(width).map(col => {
    const idx = getCellIndex(row, col, width);
    return (
      <TableCell key={`cell-${col}-${row}`} row={row} col={col} cell={cells[idx]} onCellFocus={onCellFocus} onCellBlur={onCellBlur}/>
    )
  });
  return (
    <tr>
      <td className="cell-header">{row + 1}</td>
      {tableCells}
    </tr>
  );
};

const TableCell = ({row, col, cell, onCellFocus, onCellBlur}) => {
  const [value, setValue] = useState(cell.out);
  // Ideally we would do this with useEffect but it was painfully slow to register
  // an effect on all of the cells.
  const onKeyDown = (event) => {
    let dy = 0;
    let dx = 0;
    if (event.key === "Enter") {
      dy = 1;
    } else if (event.key === "ArrowDown") {
      dy = 1;
    } else if (event.key === "ArrowUp") {
      dy = -1;
    } else if (event.key === "ArrowRight") {
      dx = 1;
    } else if (event.key === "ArrowLeft") {
      dx = -1;
    }
    const input = document.getElementById(`input-${row + dy}-${col + dx}`);
    if (input) {
      input.focus();
    }
  };

  const onChange = (e) => {
    setValue(e.target.value);
  };

  const onFocus = (e) => {
    console.log("focus");
    onCellFocus(row, col, e.target.value);
    setValue(cell.raw);
  };

  const onBlur = (e) => {
    console.log("blur");
    onCellBlur(row, col, e.target.value);
    setValue(cell.out);
  };

  return (
    <td className="cell" onKeyDown={onKeyDown}>
      <input id={`input-${row}-${col}`} className="cell-input" onChange={onChange} onFocus={onFocus} onBlur={onBlur} value={value}/>
    </td>
  )
};

const range = (upper) => {
  return [...Array(upper).keys()];
};

const getCellIndex = (row, col, width) => {
  return row * width + col;
};

export default App;
