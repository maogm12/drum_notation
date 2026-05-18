import { createRequire } from "node:module";

type LayoutWasmNodeModule = {
  build_layout_scene(source: string, options: unknown): unknown;
};

const require = createRequire(import.meta.url);
let layoutModule: LayoutWasmNodeModule | null = null;

export async function initLayoutWasmNode(): Promise<void> {
  if (!layoutModule) {
    layoutModule = require("./layout-pkg-node/drummark_core.js") as LayoutWasmNodeModule;
  }
}

export function isLayoutWasmNodeReady(): boolean {
  return layoutModule !== null;
}

export function buildLayoutSceneWithNodeWasm(source: string, options: unknown): unknown {
  if (!layoutModule) {
    throw new Error("Layout WASM is not initialized.");
  }
  return layoutModule.build_layout_scene(source, options);
}
