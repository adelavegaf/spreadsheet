import { useEffect, useRef, useState } from "react";
import './App.css';
import { Spreadsheet } from "spreadsheet";

const ss = Spreadsheet.new();
const initialCells = ss.cells();
const width = ss.width();
const height = ss.height();

const App = () => {
  const [participants, setParticipants] = useState([]);
  const ws = useRef(null);

  useEffect(() => {
    ws.current = new WebSocket("ws://localhost:8888/ws/");

    ws.current.onopen = () => {
      console.log("connected");
    };
  
    ws.current.onmessage = (e) => {
      console.log("message", e);
      const event = JSON.parse(e.data);
      switch (event.type) {
        case "Participants":
          setParticipants(event.ids);
          break;
        default:
          break;
      }
    };
  
    ws.current.onclose = () => {
      console.log("disconnected");
    };

    return () => {
        ws.current.close();
    };
  }, []);

  return (
    <>
      <Participants participants={participants}/>
      <Table/>
    </>
  )
};

const Participants = ({participants}) => {
  return (
    <div className="participant-container">
      {participants.map(p => {
        return <span key={p} className="participant-tag">{p}</span>
      })}
    </div>
  )
}

const Table = () => {
  const [cells, setCells] = useState(initialCells);
  const [selectedCell, setSelectedCell] = useState({row: 0, col: 0});

  const onCellFocus = (row, col) => {
    setSelectedCell({row: row, col: col});
  };

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

  return (
    <div className="table-container">
      <table id="table" cellSpacing="0">
        <TableHeader width={width}/>
        <TableBody width={width} height={height} cells={cells} selectedCell={selectedCell} onCellFocus={onCellFocus} onCellBlur={onCellBlur}/>
      </table>
    </div>
  );
};

const TableHeader = ({width}) => {
  return (
    <thead>
      <tr>
        <th className="cell-header"/>
        {
          range(width).map(idx => <th key={`header-${idx}`} className="cell-header">{colToLetters(idx)}</th>)
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

const TableBody = ({width, height, cells, selectedCell, onCellFocus, onCellBlur}) => {
  const rows = range(height).map(row => {
      return (
        <tr key={`row-${row}`}>
          <td className="cell-header">{row + 1}</td>
          {
          range(width).map((col) => {
            const idx = getCellIndex(row, col, width);
            const isFocused = selectedCell.row === row && selectedCell.col === col;
            return (
              <TableCell key={`cell-${col}-${row}`} row={row} col={col} cell={cells[idx]} isFocused={isFocused} onCellFocus={onCellFocus} onCellBlur={onCellBlur}/>
            );
          })
          }
        </tr>
      );
  });

  return (
    <tbody>
      {rows}
    </tbody>
  );
};

const TableCell = ({row, col, cell, isFocused, onCellFocus, onCellBlur}) => {
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
    onCellFocus(row, col);
  };

  const onBlur = (e) => {
    onCellBlur(row, col, value);
  };

  return (
    <td className="cell" >
      <input id={`input-${row}-${col}`} className="cell-input" onChange={onChange} onKeyDown={onKeyDown} onFocus={onFocus} onBlur={onBlur} value={value}/>
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
