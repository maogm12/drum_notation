import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import {
  initLayoutWasmNode,
  buildLayoutSceneWithNodeWasm,
} from "./layout_wasm_node";
import {
  initParserWasmNode,
  parseWithParserWasmNode,
} from "./parser_wasm_node";

const WASM_DIR = join(process.cwd(), "src", "wasm");

describe("split wasm wrappers", () => {
  it("parses through the node parser package without importing layout wrappers", async () => {
    await initParserWasmNode();
    const result = parseWithParserWasmNode("time 4/4\nnote 1/8\n\n| p p b b |");

    expect(result).toMatchObject({
      headers: expect.any(Object),
      paragraphs: expect.any(Array),
    });

    const source = readFileSync(join(WASM_DIR, "parser_wasm_node.ts"), "utf8");
    expect(source).toContain("./parser-pkg-node/drummark_core.js");
    expect(source).not.toContain("layout");
  });

  it("builds layout scenes through the node layout package", async () => {
    await initLayoutWasmNode();
    const scene = buildLayoutSceneWithNodeWasm("time 4/4\nnote 1/8\n\n| p p b b |", {
      pageWidth: 612,
      pageHeight: 792,
    });

    expect(scene).toMatchObject({
      version: expect.any(String),
      pages: expect.any(Array),
    });

    const pages = (scene as { pages: unknown[] }).pages;
    expect(pages.length).toBeGreaterThan(0);
  });

  it("keeps browser wrappers pointed at web packages only", () => {
    const parserBrowser = readFileSync(join(WASM_DIR, "parser_wasm_browser.ts"), "utf8");
    const layoutBrowser = readFileSync(join(WASM_DIR, "layout_wasm_browser.ts"), "utf8");

    expect(parserBrowser).toContain("./parser-pkg-web/drummark_core");
    expect(parserBrowser).not.toContain("node:");
    expect(parserBrowser).not.toContain("layout-pkg");
    expect(layoutBrowser).toContain("./layout-pkg-web/drummark_core");
    expect(layoutBrowser).not.toContain("node:");
    expect(layoutBrowser).not.toContain("parser-pkg");
  });
});
