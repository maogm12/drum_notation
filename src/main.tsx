import { createRoot } from "preact/compat/client";
import { App } from "./App";
import { I18nProvider } from "./i18n/context";
import { initParserWasmBrowser } from "./wasm/parser_wasm_browser";
import "./styles.css";

async function bootstrap() {
  await initParserWasmBrowser();
  createRoot(document.getElementById("root")!).render(
    <I18nProvider>
      <App />
    </I18nProvider>,
  );
}

bootstrap().catch(console.error);
