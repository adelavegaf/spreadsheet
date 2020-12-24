import React, {
  useState,
  useCallback,
  createContext,
  useRef,
  useEffect,
} from "react";
import { Spreadsheet } from "spreadsheet";
import { getCellIndex, getCellRowCol } from "./Utils";

export const AppContext = createContext();

export const AppProvider = (props) => {
  // Spreadsheet
  const ssRef = useRef(Spreadsheet.new());
  const [cells, setCells] = useState(ssRef.current.cells());
  const [width] = useState(ssRef.current.width());
  const [height] = useState(ssRef.current.height());
  const localSetCell = useCallback(
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
  // Web socket
  const [userId, setUserId] = useState(0);
  const [participants, setParticipants] = useState([]);
  const onWsEvent = useCallback(
    (response) => {
      switch (response.type) {
        case "Connected":
          setUserId(response.user_id);
          // TODO: Ideally we would wait until we got the cells to create the SS WASM object.
          response.cells.map((c) => {
            console.log("setting");
            localSetCell(getCellIndex(c.row, c.col, width), c.raw);
          });
          break;
        case "Participants":
          setParticipants(response.ids);
          break;
        case "CellUpdated":
          localSetCell(
            getCellIndex(response.cell.row, response.cell.col, width),
            response.cell.raw
          );
          break;
        default:
          console.error("unhandled response", response);
          break;
      }
    },
    [width, localSetCell]
  );
  const [ws, isOnline] = useWs(onWsEvent);

  const setCell = useCallback(
    (index, raw) => {
      // TODO: this is sending events even if the cell didnt have its contents changed.
      if (isOnline && userId) {
        const [row, col] = getCellRowCol(index, width);
        ws.current.send(
          JSON.stringify({
            type: "UpdateCell",
            user_id: userId,
            sheet_id: 1,
            row: row,
            col: col,
            raw: raw,
          })
        );
      }
      localSetCell(index, raw);
    },
    [isOnline, userId, ws, width, localSetCell]
  );

  const value = {
    cells,
    width,
    height,
    isOnline,
    userId,
    participants,
    setCell,
  };
  return (
    <AppContext.Provider value={value}>{props.children}</AppContext.Provider>
  );
};

const useWs = (onEvent) => {
  const [isOnline, setIsOnline] = useState(false);
  const ws = useRef(null);

  useEffect(() => {
    ws.current = new WebSocket("ws://localhost:8888/ws/");

    ws.current.onopen = () => {
      setIsOnline(true);
    };

    ws.current.onmessage = (e) => {
      const event = JSON.parse(e.data);
      console.log("event", event);
      onEvent(event);
    };

    ws.current.onclose = () => {
      setIsOnline(false);
    };

    return () => {
      ws.current.close();
    };
  }, [onEvent]);

  return [ws, isOnline];
};
