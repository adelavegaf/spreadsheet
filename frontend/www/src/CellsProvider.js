import React, { useState, useCallback, createContext, useRef } from "react";
import { Spreadsheet } from "spreadsheet";
import { getCellRowCol } from "./Utils";

export const CellsContext = createContext();

export const CellsProvider = (props) => {
  const ssRef = useRef(Spreadsheet.new());
  const [cells, setCells] = useState(ssRef.current.cells());
  const [width] = useState(ssRef.current.width());
  const [height] = useState(ssRef.current.height());

  const setCell = useCallback(
    (index, raw) => {
      setCells((prevCells) => {
        if (raw === prevCells[index].raw) {
          return prevCells;
        }
        const [row, col] = getCellRowCol(index, width);
        const updates = ssRef.current.set(row, col, raw);
        const newCells = [...prevCells];
        for (const [idx, cell] of Object.entries(updates)) {
          newCells[idx] = cell;
        }
        return newCells;
      });
    },
    [width]
  );

  const value = { cells, width, height, setCell };

  return (
    <CellsContext.Provider value={value}>
      {props.children}
    </CellsContext.Provider>
  );
};
