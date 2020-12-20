import React, { useContext } from "react";
import { AppContext } from "./AppProvider";

export const Participants = () => {
  const { participants, isOnline } = useContext(AppContext);
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
