import React from "react";
import ReactDOM from "react-dom/client";
import DomRouter from "./Router";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <DomRouter />
  </React.StrictMode>,
);
