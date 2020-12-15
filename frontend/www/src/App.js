/* eslint-disable no-unused-vars */
import React from "react";
import { useEffect, useState, useCallback, useMemo } from "react";
import "./App.css";
import { Spreadsheet } from "spreadsheet";

const ss = Spreadsheet.new();
const initialCells = ss.cells();
const initCell = initialCells[0];
const initFocusedCell = { row: 0, col: 0 };
const width = ss.width();
const height = ss.height();

const App = () => {
  return (
    <>
      {/* <Participants participants={participants} isOnline={isOnline} /> */}
      <Sheet />
    </>
  );
};

// const Participants = ({ participants, isOnline }) => {
//   return (
//     <div className="participant-container">
//       <span
//         className={isOnline ? "online-status online" : "online-status offline"}
//       />
//       {participants.map((p) => {
//         return (
//           <span key={p} className="participant-tag">
//             {p}
//           </span>
//         );
//       })}
//     </div>
//   );
// };

const Sheet = () => {
  // sheet specific
  const [cells, setCells] = useState(initialCells);
  const [focusedCell, setFocusedCell] = useState(initFocusedCell);
  const [focusedCellValue, setFocusedCellValue] = useState(initCell.raw);

  const onCellFocus = (row, col) => {
    const idx = getCellIndex(row, col, width);
    setFocusedCell({ row, col });
    setFocusedCellValue(cells[idx].raw);
  };

  const onFocusedCellValueChange = (value) => {
    setFocusedCellValue(value);
  };

  const onFocusedCellBlur = (row, col) => {
    console.log("blurring", row, col);
    const idx = getCellIndex(row, col, width);
    if (focusedCellValue === cells[idx].raw) {
      return;
    }
    const updates = ss.set(row, col, focusedCellValue);
    setCells((prevCells) => {
      const newCells = [...prevCells];
      for (const [idx, cell] of Object.entries(updates)) {
        newCells[idx] = cell;
      }
      return newCells;
    });
  };

  return (
    <>
      <FormulaBar
        value={focusedCellValue}
        onChange={onFocusedCellValueChange}
      />
      <Table
        width={width}
        height={height}
        cells={cells}
        onCellFocus={onCellFocus}
        focusedCell={focusedCell}
        focusedCellValue={focusedCellValue}
        onFocusedCellValueChange={onFocusedCellValueChange}
        onFocusedCellBlur={onFocusedCellBlur}
      />
    </>
  );
};

const FormulaBar = ({ value, onChange }) => {
  return (
    <input
      value={value}
      style={{ width: "100%" }}
      onChange={(e) => onChange(e.target.value)}
    />
  );
};

const Table = ({
  width,
  height,
  cells,
  onCellFocus,
  focusedCell,
  focusedCellValue,
  onFocusedCellValueChange,
  onFocusedCellBlur,
}) => {
  // Ideally we would do this with useEffect but it was painfully slow to register
  // an effect on all of the cells.
  // const onKeyDown = (event) => {
  //   let dy = 0;
  //   let dx = 0;
  //   if (event.key === "Enter") {
  //     dy = 1;
  //   } else if (event.key === "ArrowDown") {
  //     dy = 1;
  //   } else if (event.key === "ArrowUp") {
  //     dy = -1;
  //   } else if (event.key === "ArrowRight") {
  //     dx = 1;
  //   } else if (event.key === "ArrowLeft") {
  //     dx = -1;
  //   }
  //   const input = document.getElementById(`input-${row + dy}-${col + dx}`);
  //   if (input) {
  //     input.focus();
  //   }
  // };

  return (
    <div className="table-container">
      <table id="table" cellSpacing="0">
        <TableHeader width={width} />
        <TableBody
          width={width}
          height={height}
          cells={cells}
          onCellFocus={onCellFocus}
          focusedCell={focusedCell}
          focusedCellValue={focusedCellValue}
          onFocusedCellValueChange={onFocusedCellValueChange}
          onFocusedCellBlur={onFocusedCellBlur}
        />
      </table>
    </div>
  );
};

const TableHeader = ({ width }) => {
  return (
    <thead>
      <tr>
        <th className="cell-header" />
        {range(width).map((idx) => (
          <th key={`header-${idx}`} className="cell-header">
            {colToLetters(idx)}
          </th>
        ))}
      </tr>
    </thead>
  );
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
  const asciiCode = remainders
    .map((n) => {
      return asciiOffset + n;
    })
    .reverse();

  return String.fromCharCode(asciiCode);
};

const TableBody = ({
  width,
  height,
  cells,
  onCellFocus,
  focusedCell,
  focusedCellValue,
  onFocusedCellValueChange,
  onFocusedCellBlur,
}) => {
  const rows = range(height).map((row) => {
    return (
      <tr key={`row-${row}`}>
        <td className="cell-header">{row + 1}</td>
        {range(width).map((col) => {
          const idx = getCellIndex(row, col, width);
          const cell = cells[idx];
          const key = `cell-${idx}`;
          const isFocused = focusedCell.row === row && focusedCell.col === col;
          return isFocused ? (
            <FocusedTableCell
              key={key}
              row={row}
              col={col}
              value={focusedCellValue}
              onChange={onFocusedCellValueChange}
              onBlur={onFocusedCellBlur}
            />
          ) : (
            <TableCell
              key={key}
              row={row}
              col={col}
              value={cell.raw ? cell.out : ""}
              onFocus={onCellFocus}
            />
          );
        })}
      </tr>
    );
  });

  return <tbody>{rows}</tbody>;
};

const FocusedTableCell = ({ row, col, value, onChange, onBlur }) => {
  return (
    <td className="cell">
      <input
        className="cell-input"
        onChange={(e) => onChange(e.target.value)}
        onBlur={() => onBlur(row, col)}
        value={value}
        autoFocus
      />
    </td>
  );
};

const TableCell = ({ row, col, value, onFocus }) => {
  return (
    <td className="cell">
      <input
        className="cell-input"
        onFocus={() => onFocus(row, col)}
        value={value}
        readOnly
      />
    </td>
  );
};

const range = (upper) => {
  return [...Array(upper).keys()];
};

const getCellIndex = (row, col, width) => {
  return row * width + col;
};

export default App;
