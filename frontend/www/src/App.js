import { useEffect, useRef, useState } from "react";
import "./App.css";
import { Spreadsheet } from "spreadsheet";

const ss = Spreadsheet.new();
const initialCells = ss.cells();
const width = ss.width();
const height = ss.height();

const App = () => {
  // websocket specifics
  const [userId, setUserId] = useState(-1);
  const [isOnline, setIsOnline] = useState(false);
  const [participants, setParticipants] = useState([]);
  const ws = useRef(null);

  // sheet specific
  const [cells, setCells] = useState(initialCells);
  const [selectedCell, setSelectedCell] = useState({ row: 0, col: 0 });

  const updateCell = (row, col, raw) => {
    const updates = ss.set(row, col, raw);
    setCells((prevCells) => {
      const newCells = [...prevCells];
      for (const [idx, cell] of Object.entries(updates)) {
        newCells[idx] = cell;
      }
      return newCells;
    });
  };

  const onCellUpdate = (row, col, raw) => {
    const idx = getCellIndex(row, col, width);
    if (raw === cells[idx].raw) {
      return;
    }

    updateCell(row, col, raw);

    // Advertise update to other users via websocket
    ws.current.send(
      JSON.stringify({ type: "CellUpdated", row: row, col: col, raw: raw })
    );
  };

  const onCellSelect = (row, col) => {
    // Update state
    console.log("on cell select");
    setSelectedCell({ row: row, col: col });
    // Advertise update to other users via websocket
    // Enable once we figure out how to make it not lag.
    // ws.current.send(JSON.stringify({type: "CellLocked", row: row, col: col, locker_id: userId}));
  };

  useEffect(() => {
    ws.current = new WebSocket("ws://localhost:8888/ws/");

    ws.current.onopen = () => {
      setIsOnline(true);
    };

    ws.current.onmessage = (e) => {
      const event = JSON.parse(e.data);
      console.log("event", event);
      switch (event.type) {
        case "Connected":
          setUserId(event.id);
          break;
        case "Participants":
          setParticipants(event.ids);
          break;
        case "CellUpdated":
          updateCell(event.row, event.col, event.raw);
          break;
        case "CellLocked":
          console.log("cell locked");
          break;
        default:
          console.error("unhandled event", event);
          break;
      }
    };

    ws.current.onclose = () => {
      setIsOnline(false);
    };

    return () => {
      ws.current.close();
    };
  }, []);

  return (
    <>
      <Participants participants={participants} isOnline={isOnline} />
      <FormulaBar cells={cells} selectedCell={selectedCell} />
      <Table
        width={width}
        height={height}
        cells={cells}
        selectedCell={selectedCell}
        onCellSelect={onCellSelect}
        onCellUpdate={onCellUpdate}
      />
    </>
  );
};

const Participants = ({ participants, isOnline }) => {
  return (
    <div className="participant-container">
      <span
        className={isOnline ? "online-status online" : "online-status offline"}
      />
      {participants.map((p) => {
        return (
          <span key={p} className="participant-tag">
            {p}
          </span>
        );
      })}
    </div>
  );
};

const FormulaBar = ({ cells, selectedCell }) => {
  const { row, col } = selectedCell;
  const idx = getCellIndex(row, col, width);
  const cell = cells[idx];
  return <input value={cell.raw} style={{ width: "100%" }} readOnly />;
};

const Table = ({
  width,
  height,
  cells,
  selectedCell,
  onCellSelect,
  onCellUpdate,
}) => {
  return (
    <div className="table-container">
      <table id="table" cellSpacing="0">
        <TableHeader width={width} />
        <TableBody
          width={width}
          height={height}
          cells={cells}
          selectedCell={selectedCell}
          onCellSelect={onCellSelect}
          onCellUpdate={onCellUpdate}
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
  selectedCell,
  onCellSelect,
  onCellUpdate,
}) => {
  const rows = range(height).map((row) => {
    return (
      <tr key={`row-${row}`}>
        <td className="cell-header">{row + 1}</td>
        {range(width).map((col) => {
          const idx = getCellIndex(row, col, width);
          const isFocused =
            selectedCell.row === row && selectedCell.col === col;
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

  const onFocus = (e) => {
    onCellSelect(row, col);
  };

  const onBlur = (e) => {
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
