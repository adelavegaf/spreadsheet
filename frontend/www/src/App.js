/* eslint-disable no-unused-vars */
import React, {
  memo,
  useContext,
  useEffect,
  useState,
  useCallback,
  createContext,
} from "react";
import "./App.css";
import { Spreadsheet } from "spreadsheet";

const ss = Spreadsheet.new();
const initialCells = ss.cells();
const initCell = initialCells[0];
const width = ss.width();
const height = ss.height();

const CellsContext = createContext();

const CellsProvider = (props) => {
  const [cells, setCells] = useState(initialCells);

  const setCell = useCallback((row, col, raw) => {
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

  const value = { cells, setCell };

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

const Sheet = () => {
  const { cells, setCell } = useContext(CellsContext);
  const [focusedCellValue, setFocusedCellValue] = useState(initCell.raw);
  const [focusedCellIndex, setFocusedCellIndex] = useState(0);

  const onFocusedCellValueChange = (value) => {
    setFocusedCellValue(value);
  };

  const onFocusedCellUpdate = (newIndex, shouldUpdate) => {
    if (shouldUpdate) {
      const [pRow, pCol] = getCellRowCol(focusedCellIndex, width);
      setCell(pRow, pCol, focusedCellValue);
    }
    setFocusedCellIndex(newIndex);
    setFocusedCellValue(cells[newIndex].raw);
  };

  return (
    <>
      <FormulaBar
        value={focusedCellValue}
        onValueChange={onFocusedCellValueChange}
        focusedCellIndex={focusedCellIndex}
        onFocusedCellUpdate={onFocusedCellUpdate}
      />
      <Table
        width={width}
        height={height}
        focusedCellValue={focusedCellValue}
        focusedCellIndex={focusedCellIndex}
        onFocusedCellValueChange={onFocusedCellValueChange}
        onFocusedCellUpdate={onFocusedCellUpdate}
      />
    </>
  );
};

const FormulaBar = ({
  value,
  onValueChange,
  focusedCellIndex,
  onFocusedCellUpdate,
}) => {
  const onKeyDown = (event) => {
    if (event.key !== "Enter" && event.key !== "Escape") {
      return;
    }
    const [nextIndex, shouldUpdate] = keyToCellUpdate(
      event.key,
      focusedCellIndex,
      width
    );
    onFocusedCellUpdate(nextIndex, shouldUpdate);
  };

  return (
    <input
      value={value}
      style={{ width: "100%" }}
      onChange={(e) => onValueChange(e.target.value)}
      onKeyDown={onKeyDown}
    />
  );
};

const Table = ({
  width,
  height,
  focusedCellValue,
  focusedCellIndex,
  onFocusedCellValueChange,
  onFocusedCellUpdate,
}) => {
  return (
    <div className="table-container">
      <table id="table" cellSpacing="0">
        <TableHeader width={width} />
        <TableBody
          width={width}
          height={height}
          focusedCellValue={focusedCellValue}
          focusedCellIndex={focusedCellIndex}
          onFocusedCellValueChange={onFocusedCellValueChange}
          onFocusedCellUpdate={onFocusedCellUpdate}
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
          <th key={idx} className="cell-header">
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
  focusedCellValue,
  focusedCellIndex,
  onFocusedCellValueChange,
  onFocusedCellUpdate,
}) => {
  const { cells } = useContext(CellsContext);

  const rows = range(height).map((row) => {
    return (
      <tr key={row}>
        <td className="cell-header">{row + 1}</td>
        {range(width).map((col) => {
          const idx = getCellIndex(row, col, width);
          const cell = cells[idx];
          const isFocused = focusedCellIndex === idx;
          return isFocused ? (
            <FocusedTableCell
              key={idx}
              value={focusedCellValue}
              onChange={onFocusedCellValueChange}
            />
          ) : (
            <UnfocusedTableCell key={idx} index={idx} cell={cell} />
          );
        })}
      </tr>
    );
  });

  const onClick = (event) => {
    // See comment on UnfocusedTableCell's onClick to understand why this works
    const idx = event.cellIndex;
    if (idx === undefined || idx === null) {
      return;
    }
    onFocusedCellUpdate(idx, true);
  };

  const onKeyDown = (event) => {
    if (!UPDATE_KEYS_SET.has(event.key)) {
      return;
    }
    const [nextIndex, shouldUpdate] = keyToCellUpdate(
      event.key,
      focusedCellIndex,
      width
    );
    onFocusedCellUpdate(nextIndex, shouldUpdate);
  };

  return (
    <tbody onClick={onClick} onKeyDown={onKeyDown}>
      {rows}
    </tbody>
  );
};

const FocusedTableCell = ({ value, onChange }) => {
  return (
    <td className="cell">
      <input
        className="cell-input"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        autoFocus
      />
    </td>
  );
};

const _UnfocusedTableCell = ({ index, cell }) => {
  const onClick = (event) => {
    // HACK: we can't pass the onFocusedCellUpdate fn to our cells or
    // we will trigger a re-render of all cells whenever a single cell
    // changes.
    //
    // As a workaround, we will piggyback on the event to store the index
    // of the cell that was just clicked, to call onFocusedCellUpdate from
    // the table body. Browsers guarantee events are propagated from
    // specific -> general, so the cell is always to come first than the
    // table body.
    event.cellIndex = index;
  };

  // TODO: style cells based on their out contents:
  // Nums are right aligned
  // Text is left aligned
  // Errors are center aligned + corner with a red triangle + tooltip displaying error msg.

  return (
    <td className="cell">
      <input
        className="cell-input"
        value={cell.out.value}
        onClick={onClick}
        readOnly
      />
    </td>
  );
};

const UnfocusedTableCell = memo(_UnfocusedTableCell);

const UPDATE_KEYS_SET = new Set([
  "Enter",
  "ArrowDown",
  "ArrowUp",
  "ArrowRight",
  "ArrowLeft",
  "Escape",
]);

const keyToCellUpdate = (key, curIndex, width) => {
  let dy = 0;
  let dx = 0;
  let shouldUpdate = true;
  if (key === "Enter") {
    dy = 1;
  } else if (key === "ArrowDown") {
    dy = 1;
  } else if (key === "ArrowUp") {
    dy = -1;
  } else if (key === "ArrowRight") {
    dx = 1;
  } else if (key === "ArrowLeft") {
    dx = -1;
  } else if (key === "Escape") {
    shouldUpdate = false;
  }
  const [curRow, curCol] = getCellRowCol(curIndex, width);
  const row = Math.min(Math.max(curRow + dy, 0), height - 1);
  const col = Math.min(Math.max(curCol + dx, 0), width - 1);
  return [getCellIndex(row, col, width), shouldUpdate];
};

const range = (upper) => {
  return [...Array(upper).keys()];
};

const getCellIndex = (row, col, width) => {
  return row * width + col;
};

const getCellRowCol = (index, width) => {
  const col = index % width;
  const row = Math.floor(index / width);
  return [row, col];
};

export default App;
