import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";
const theme = window.matchMedia("(prefers-color-scheme: dark)");
const syncTheme = () => document.documentElement.classList.toggle("dark", theme.matches);
syncTheme();
theme.addEventListener("change", syncTheme);


ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
