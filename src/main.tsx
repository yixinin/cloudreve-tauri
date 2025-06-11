import React from "react";
import ReactDOM from "react-dom/client";
import { App } from "./App";
import EmptyContextMenu from './components/contexts/EmptyContextMenu';
import { CssBaseline, ThemeProvider, createTheme } from "@mui/material";

const theme = createTheme();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <EmptyContextMenu>
        <App />
      </EmptyContextMenu>
    </ThemeProvider>
  </React.StrictMode>
);
