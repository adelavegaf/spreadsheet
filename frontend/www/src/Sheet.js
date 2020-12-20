import React, { memo, useContext, useState } from "react";
import { AppContext } from "./AppProvider";
import { getCellIndex, getCellRowCol } from "./Utils";

export const Sheet = () => {
  const { cells, width, height, setCell } = useContext(AppContext);
  const [focusedCellIndex, setFocusedCellIndex] = useState(0);
  const [focusedCellValue, setFocusedCellValue] = useState(
    cells[focusedCellIndex].raw
  );

  const onFocusedCellValueChange = (value) => {
    setFocusedCellValue(value);
  };

  const onFocusedCellUpdate = (newIndex, shouldUpdate) => {
    if (shouldUpdate) {
      setCell(focusedCellIndex, focusedCellValue);
    }
    setFocusedCellIndex(newIndex);
    setFocusedCellValue(cells[newIndex].raw);
  };

  return (
    <>
      <FormulaBar
        value={focusedCellValue}
        width={width}
        height={height}
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
  width,
  height,
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
      width,
      height
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
  const { cells } = useContext(AppContext);

  let idx = 0;
  const rows = range(height).map((row) => {
    return (
      <tr key={row}>
        <td className="cell-header">{row + 1}</td>
        {range(width).map(() => {
          const isFocused = focusedCellIndex === idx;
          const cell = cells[idx];
          const tableCell = isFocused ? (
            <FocusedTableCell
              key={idx}
              value={focusedCellValue}
              onChange={onFocusedCellValueChange}
            />
          ) : (
            <UnfocusedTableCell key={idx} index={idx} cell={cell} />
          );
          idx++;
          return tableCell;
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
      width,
      height
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

  let className = "cell-input";
  switch (cell.out.type) {
    case "Error":
      className += " cell-error";
      break;
    case "Text":
      className += " cell-text";
      break;
    case "Num":
      className += " cell-num";
      break;
  }

  return (
    <td className="cell">
      <input
        className={className}
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

const keyToCellUpdate = (key, curIndex, width, height) => {
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
