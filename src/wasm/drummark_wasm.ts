import init, { parse as wasmParse } from "./pkg/drummark_core";

let ready = false;
let initPromise: Promise<void> | null = null;

export async function initWasm(): Promise<void> {
  if (ready) return;
  if (!initPromise) {
    initPromise = init().then(() => {
      ready = true;
    });
  }
  return initPromise;
}

export function isWasmReady(): boolean {
  return ready;
}

export function parse(source: string): unknown {
  if (!ready) {
    throw new Error(
      "WASM parser not initialized. Call initWasm() before parsing.",
    );
  }
  return wasmParse(source);
}
