/* eslint-disable no-unused-vars */
import React, {
  memo,
  useContext,
  useEffect,
  useState,
  useCallback,
  useMemo,
  createContext,
} from "react";
import "./App.css";
import { Spreadsheet } from "spreadsheet";

const ss = Spreadsheet.new();
const initialCells = ss.cells();
const initCell = initialCells[0];
const initFocusedCell = { row: 0, col: 0 };
const width = ss.width();
const height = ss.height();

const CellsContext = createContext();

const CellsProvider = (props) => {
  const [cells, setCells] = useState(initialCells);

  const updateCell = useCallback((row, col, raw) => {
    setCells((prevCells) => {
      const idx = getCellIndex(row, col, width);
      if (raw === prevCells[idx].raw) {
        return prevCells;
      }
      const updates = ss.set(row, col, raw);

      const newCells = [...prevCells];
      for (const [idx, cell] of Object.entries(updates)) {
        newCells[idx] = cell;
      }
      return newCells;
    });
  }, []);

  const value = { cells, updateCell };

  return (
    <CellsContext.Provider value={value}>
      {props.children}
    </CellsContext.Provider>
  );
};

const App = () => {
  return (
    <CellsProvider>
      {/* <Participants participants={participants} isOnline={isOnline} /> */}
      <Sheet />
    </CellsProvider>
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

const FocusedCellValueContext = createContext();

const FocusedCellValueProvider = (props) => {
  const [focusedCellValue, setFocusedCellValue] = useState(initCell.raw);

  const onFocusedCellValueChange = (value) => {
    setFocusedCellValue(value);
  };

  const value = { focusedCellValue, onFocusedCellValueChange };

  return (
    <FocusedCellValueContext.Provider value={value}>
      {props.children}
    </FocusedCellValueContext.Provider>
  );
};

const Sheet = () => {
  // sheet specific
  const [focusedCellIndex, setFocusedCellIndex] = useState(0);

  const onCellFocus = useCallback(
    (row, col) => {
      const idx = getCellIndex(row, col, width);
      setFocusedCellIndex(idx);
    },
    [setFocusedCellIndex]
  );

  return (
    <FocusedCellValueProvider>
      <FormulaBar />
      <Table
        width={width}
        height={height}
        focusedCellIndex={focusedCellIndex}
        onCellFocus={onCellFocus}
      />
    </FocusedCellValueProvider>
  );
};

const FormulaBar = () => {
  const { focusedCellValue, onFocusedCellValueChange } = useContext(
    FocusedCellValueContext
  );
  return (
    <input
      value={focusedCellValue}
      style={{ width: "100%" }}
      onChange={(e) => onFocusedCellValueChange(e.target.value)}
    />
  );
};

const Table = ({ width, height, focusedCellIndex, onCellFocus }) => {
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
          focusedCellIndex={focusedCellIndex}
          onCellFocus={onCellFocus}
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

const TableBody = ({ width, height, focusedCellIndex, onCellFocus }) => {
  const { cells, updateCell } = useContext(CellsContext);
  const rows = range(height).map((row) => {
    return (
      <tr key={row}>
        <td className="cell-header">{row + 1}</td>
        {range(width).map((col) => {
          const idx = getCellIndex(row, col, width);
          const cell = cells[idx];
          return (
            <MemoTableCell
              key={idx}
              row={row}
              col={col}
              cell={cell}
              isFocused={focusedCellIndex === idx}
              onFocus={onCellFocus}
              onUpdate={updateCell}
            />
          );
        })}
      </tr>
    );
  });

  return <tbody>{rows}</tbody>;
};

const TableCell = ({ row, col, cell, isFocused, onFocus, onUpdate }) => {
  return isFocused ? (
    <FocusedTableCell row={row} col={col} cell={cell} onBlur={onUpdate} />
  ) : (
    <UnfocusedTableCell row={row} col={col} cell={cell} onFocus={onFocus} />
  );
};

const FocusedTableCell = ({ row, col, cell, onBlur }) => {
  const { focusedCellValue, onFocusedCellValueChange } = useContext(
    FocusedCellValueContext
  );

  useEffect(() => {
    onFocusedCellValueChange(cell.raw);
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  return (
    <td className="cell">
      <input
        className="cell-input"
        value={focusedCellValue}
        onChange={(e) => onFocusedCellValueChange(e.target.value)}
        onBlur={() => onBlur(row, col, focusedCellValue)}
        autoFocus
      />
    </td>
  );
};

const UnfocusedTableCell = ({ row, col, cell, onFocus }) => {
  return (
    <td className="cell">
      <input
        className="cell-input"
        onFocus={() => onFocus(row, col)}
        value={cell.raw ? cell.out : ""}
        readOnly
      />
    </td>
  );
};

const MemoTableCell = memo(TableCell);

const range = (upper) => {
  return [...Array(upper).keys()];
};

const getCellIndex = (row, col, width) => {
  return row * width + col;
};

export default App;
