import init, { initSync, parse as wasmParse } from "./parser-pkg-web/drummark_core";
import { setParserRuntime } from "./parser_runtime";

let ready = false;
let initPromise: Promise<void> | null = null;

export async function initParserWasmBrowser(): Promise<void> {
  if (ready) return;
  if (!initPromise) {
    const wasmUrl = new URL("./parser-pkg-web/drummark_core_bg.wasm", import.meta.url);
    initPromise = init({ module_or_path: wasmUrl })
      .then(() => {
        ready = true;
        setParserRuntime({ parse: wasmParse });
      })
      .catch((error) => {
        initPromise = null;
        throw error;
      });
  }
  return initPromise;
}

export function initParserWasmBrowserForTests(module: BufferSource | WebAssembly.Module): void {
  if (ready) return;
  initSync({ module });
  ready = true;
  setParserRuntime({ parse: wasmParse });
}

export function isParserWasmBrowserReady(): boolean {
  return ready;
}

export function parseWithParserWasmBrowser(source: string): unknown {
  if (!ready) {
    throw new Error("Parser WASM is not initialized.");
  }
  return wasmParse(source);
}
