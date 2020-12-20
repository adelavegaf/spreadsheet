import React, {
  useState,
  useCallback,
  createContext,
  useRef,
  useEffect,
} from "react";
import { Spreadsheet } from "spreadsheet";
import { getCellRowCol } from "./Utils";

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
    (event) => {
      switch (event.type) {
        case "Connected":
          setUserId(event.id);
          break;
        case "Participants":
          setParticipants(event.ids);
          break;
        case "CellUpdated":
          localSetCell(event.cell_idx, event.raw);
          break;
        case "CellLocked":
          // TODO: handle cell locked events
          break;
        default:
          console.error("unhandled event", event);
          break;
      }
    },
    [localSetCell]
  );
  const [ws, isOnline] = useWs(onWsEvent);

  const setCell = useCallback(
    (index, raw) => {
      // TODO: this is sending events even if the cell didnt have its contents changed.
      if (isOnline && userId) {
        ws.current.send(
          JSON.stringify({
            type: "CellUpdated",
            cell_idx: index,
            user_id: userId,
            raw: raw,
          })
        );
      }
      localSetCell(index, raw);
    },
    [isOnline, userId, ws, localSetCell]
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
