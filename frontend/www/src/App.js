/* eslint-disable no-unused-vars */
import React, {
  memo,
  useContext,
  useEffect,
  useState,
  useCallback,
  createContext,
} from "react";
import { CellsContext, CellsProvider } from "./CellsProvider";
import "./App.css";
import { Sheet } from "./Sheet";

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

export default App;
