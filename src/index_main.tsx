import React from "react";
import ReactDOM from "react-dom/client";
import App from "./AppMain";

import "./core.css"
import "./index_main.css"

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
