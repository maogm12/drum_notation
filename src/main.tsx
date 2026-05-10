import { createRoot } from "preact/compat/client";
import { App } from "./App";
import { I18nProvider } from "./i18n/context";
import { initWasm } from "./wasm/drummark_wasm";
import "./styles.css";

// Pre-initialize WASM parser in background
initWasm().catch(console.warn);

createRoot(document.getElementById("root")!).render(
  <I18nProvider>
    <App />
  </I18nProvider>,
);
