import React from "react";
import { AppProvider } from "./AppProvider";
import "./App.css";
import { Sheet } from "./Sheet";
import { Participants } from "./Participants";

const App = () => {
  return (
    <AppProvider>
      <Participants />
      <Sheet />
    </AppProvider>
  );
};

export default App;
