type ParserRuntime = {
  parse(source: string): unknown;
};

let runtime: ParserRuntime | null = null;

export function setParserRuntime(nextRuntime: ParserRuntime): void {
  runtime = nextRuntime;
}

export function isParserRuntimeReady(): boolean {
  return runtime !== null;
}

export function parseWithParserRuntime(source: string): unknown {
  if (!runtime) {
    throw new Error("WASM parser not ready. Call initWasm() first.");
  }
  return runtime.parse(source);
}
