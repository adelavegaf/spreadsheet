/* eslint-disable no-unused-vars */
import React from "react";
import { useEffect, useState, useCallback, useMemo } from "react";
import "./App.css";
import { Spreadsheet } from "spreadsheet";

const ss = Spreadsheet.new();
const initialCells = ss.cells();
const initialCell = initialCells[0];
const initialCurCell = { row: 0, col: 0 };
const width = ss.width();
const height = ss.height();

const hookset = new Set();

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
  const [curCell, setCurCell] = useState(initialCurCell);
  const [curRaw, setCurRaw] = useState(initialCell.raw);

  const onRawUpdate = useCallback((raw) => {
    setCurRaw(raw);
  }, []);

  const onCellUpdate = useCallback(
    (row, col, raw) => {
      const idx = getCellIndex(row, col, width);
      if (raw === cells[idx].raw) {
        return;
      }
      const updates = ss.set(row, col, raw);
      setCells((prevCells) => {
        const newCells = [...prevCells];
        for (const [idx, cell] of Object.entries(updates)) {
          newCells[idx] = cell;
        }
        return newCells;
      });
    },
    [cells]
  );

  const onCellSelect = useCallback((row, col) => {
    // Update state
    setCurCell({ row, col });
  }, []);

  hookset.add(onRawUpdate);
  hookset.add(onCellUpdate);
  hookset.add(onCellSelect);

  console.log(hookset, hookset.size);

  return (
    <>
      <FormulaBar value={curRaw} onChange={onRawUpdate} />
      <Table
        width={width}
        height={height}
        cells={cells}
        curCell={curCell}
        onCellSelect={onCellSelect}
        onCellUpdate={onCellUpdate}
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
  curCell,
  onCellSelect,
  onCellUpdate,
}) => {
  return useMemo(() => {
    return (
      <div className="table-container">
        <table id="table" cellSpacing="0">
          <TableHeader width={width} />
          <TableBody
            width={width}
            height={height}
            cells={cells}
            curCell={curCell}
            onCellSelect={onCellSelect}
            onCellUpdate={onCellUpdate}
          />
        </table>
      </div>
    );
  }, [width, height, cells, curCell, onCellSelect, onCellUpdate]);
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
  curCell,
  onCellSelect,
  onCellUpdate,
}) => {
  const rows = range(height).map((row) => {
    return (
      <tr key={`row-${row}`}>
        <td className="cell-header">{row + 1}</td>
        {range(width).map((col) => {
          const idx = getCellIndex(row, col, width);
          const isFocused = curCell.row === row && curCell.col === col;
          return (
            <TableCell
              key={`cell-${col}-${row}`}
              row={row}
              col={col}
              cell={cells[idx]}
              isFocused={isFocused}
              onCellSelect={onCellSelect}
              onCellUpdate={onCellUpdate}
            />
          );
        })}
      </tr>
    );
  });

  return <tbody>{rows}</tbody>;
};

const TableCell = ({
  row,
  col,
  cell,
  isFocused,
  onCellSelect,
  onCellUpdate,
}) => {
  const [value, setValue] = useState("");
  useEffect(() => {
    if (isFocused) {
      setValue(cell.raw);
      return;
    }
    setValue(cell.raw.length > 0 ? cell.out : "");
  }, [isFocused, cell]);

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

  const onFocus = () => {
    onCellSelect(row, col);
  };

  const onBlur = () => {
    onCellUpdate(row, col, value);
  };

  return (
    <td className="cell">
      <input
        id={`input-${row}-${col}`}
        className="cell-input"
        onChange={onChange}
        onKeyDown={onKeyDown}
        onFocus={onFocus}
        onBlur={onBlur}
        value={value}
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
