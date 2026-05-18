import { createRequire } from "node:module";
import { setParserRuntime } from "./parser_runtime";

type ParserWasmNodeModule = {
  parse(source: string): unknown;
};

const require = createRequire(import.meta.url);
let parserModule: ParserWasmNodeModule | null = null;

export async function initParserWasmNode(): Promise<void> {
  if (!parserModule) {
    parserModule = require("./parser-pkg-node/drummark_core.js") as ParserWasmNodeModule;
    setParserRuntime({ parse: parserModule.parse });
  }
}

export function isParserWasmNodeReady(): boolean {
  return parserModule !== null;
}

export function parseWithParserWasmNode(source: string): unknown {
  if (!parserModule) {
    throw new Error("Parser WASM is not initialized.");
  }
  return parserModule.parse(source);
}
