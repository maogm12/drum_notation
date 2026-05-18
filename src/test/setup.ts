import { beforeAll } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { initParserWasmBrowserForTests } from "../wasm/parser_wasm_browser";
import { initLayoutWasmBrowserForTests } from "../wasm/layout_wasm_browser";

beforeAll(async () => {
  const root = process.cwd();
  initParserWasmBrowserForTests(readFileSync(join(root, "src/wasm/parser-pkg-web/drummark_core_bg.wasm")));
  initLayoutWasmBrowserForTests(readFileSync(join(root, "src/wasm/layout-pkg-web/drummark_core_bg.wasm")));
});
