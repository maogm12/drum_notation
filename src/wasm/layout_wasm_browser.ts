import init, { initSync, build_layout_scene } from "./layout-pkg-web/drummark_core";

let ready = false;
let initPromise: Promise<void> | null = null;

export async function initLayoutWasmBrowser(): Promise<void> {
  if (ready) return;
  if (!initPromise) {
    const wasmUrl = new URL("./layout-pkg-web/drummark_core_bg.wasm", import.meta.url);
    initPromise = init({ module_or_path: wasmUrl })
      .then(() => {
        ready = true;
      })
      .catch((error) => {
        initPromise = null;
        throw error;
      });
  }
  return initPromise;
}

export function initLayoutWasmBrowserForTests(module: BufferSource | WebAssembly.Module): void {
  if (ready) return;
  initSync({ module });
  ready = true;
}

export function isLayoutWasmBrowserReady(): boolean {
  return ready;
}

export function buildLayoutSceneWithBrowserWasm(source: string, options: unknown): unknown {
  if (!ready) {
    throw new Error("Layout WASM is not initialized.");
  }
  return build_layout_scene(source, options);
}
