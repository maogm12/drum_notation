import { describe, expect, it } from "vitest";
import { extractImports, scanImportBoundaries, splitWasmImportRules } from "./check_import_boundaries.mjs";

describe("import boundary scanner harness", () => {
  it("extracts static re-export and dynamic import specifiers", () => {
    expect(extractImports(`
      import x from "./a";
      import "./b";
      export { y } from "./c";
      const z = await import("./d");
    `)).toEqual(["./a", "./b", "./c", "./d"]);
  });

  it("flags forbidden browser-to-node imports in fixtures", () => {
    const violations = scanImportBoundaries([
      {
        path: "src/App.tsx",
        source: 'import { initLayoutWasmNode } from "./wasm/layout_wasm_node";',
      },
    ], splitWasmImportRules);

    expect(violations).toEqual([
      expect.objectContaining({
        file: "src/App.tsx",
        import: "./wasm/layout_wasm_node",
        rule: "browser production must not import node wasm",
      }),
    ]);
  });

  it("flags parser-facing imports of layout wrappers in fixtures", () => {
    const violations = scanImportBoundaries([
      {
        path: "src/wasm/skeleton.ts",
        source: 'import { initLayoutWasmBrowser } from "./layout_wasm_browser";',
      },
    ], splitWasmImportRules);

    expect(violations).toEqual([
      expect.objectContaining({
        file: "src/wasm/skeleton.ts",
        import: "./layout_wasm_browser",
        rule: "parser-facing code must not import layout wasm",
      }),
    ]);
  });

  it("flags CLI imports of browser wrappers in fixtures", () => {
    const violations = scanImportBoundaries([
      {
        path: "src/cli_runtime.ts",
        source: 'import { initLayoutWasmBrowser } from "./wasm/layout_wasm_browser";',
      },
    ], splitWasmImportRules);

    expect(violations).toEqual([
      expect.objectContaining({
        file: "src/cli_runtime.ts",
        import: "./wasm/layout_wasm_browser",
        rule: "cli runtime must not import browser wasm",
      }),
    ]);
  });

  it("allows explicitly named parity tests to import both parser and layout wrappers", () => {
    const violations = scanImportBoundaries([
      {
        path: "src/wasm/split_wasm_wrappers.test.ts",
        source: [
          'import { initParserWasmNode } from "./parser_wasm_node";',
          'import { initLayoutWasmNode } from "./layout_wasm_node";',
        ].join("\n"),
      },
    ], splitWasmImportRules);

    expect(violations).toEqual([]);
  });
});
