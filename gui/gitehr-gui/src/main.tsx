import React from "react";
import ReactDOM from "react-dom/client";
import { createTheme, MantineProvider } from "@mantine/core";
import "@mantine/core/styles.css";
import App from "./App";

const theme = createTheme({
  fontFamily: "'IBM Plex Sans', sans-serif",
  headings: { fontFamily: "'Space Grotesk', sans-serif" },
  primaryColor: "teal",
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <MantineProvider theme={theme} defaultColorScheme="light">
      <App />
    </MantineProvider>
  </React.StrictMode>,
);
