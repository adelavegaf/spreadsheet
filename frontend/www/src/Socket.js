// import { useEffect, useRef, useState } from "react";

// const [userId, setUserId] = useState(-1);
// const [isOnline, setIsOnline] = useState(false);
// const [participants, setParticipants] = useState([]);
// const ws = useRef(null);

// const init = useEffect(() => {
//   ws.current = new WebSocket("ws://localhost:8888/ws/");

//   ws.current.onopen = () => {
//     setIsOnline(true);
//   };

//   ws.current.onmessage = (e) => {
//     const event = JSON.parse(e.data);
//     console.log("event", event);
//     switch (event.type) {
//       case "Connected":
//         setUserId(event.id);
//         break;
//       case "Participants":
//         setParticipants(event.ids);
//         break;
//       case "CellUpdated":
//         updateCell(event.row, event.col, event.raw);
//         break;
//       case "CellLocked":
//         console.log("cell locked");
//         break;
//       default:
//         console.error("unhandled event", event);
//         break;
//     }
//   };

//   ws.current.onclose = () => {
//     setIsOnline(false);
//   };

//   return () => {
//     ws.current.close();
//   };
// }, []);

// Advertise cell update to other users via websocket
// TODO: enable once we figure out how to make it not lag.
// ws.current.send(
//   JSON.stringify({ type: "CellUpdated", row: row, col: col, raw: raw })
// );

// Advertise lock update to other users via websocket
// TODO: enable once we figure out how to make it not lag.
// ws.current.send(JSON.stringify({type: "CellLocked", row: row, col: col, locker_id: userId}));
