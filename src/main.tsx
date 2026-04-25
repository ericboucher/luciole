import React from "react";
import ReactDOM from "react-dom/client";
import { App } from "./App";
import { OverlayApp } from "./overlay/OverlayApp";
import "./styles/tokens.css";
import "./styles/app.css";

const hash = globalThis.location?.hash ?? "";
const isOverlay = hash.startsWith("#/overlay");

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>{isOverlay ? <OverlayApp /> : <App />}</React.StrictMode>,
);
