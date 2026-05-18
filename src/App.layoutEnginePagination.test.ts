import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

describe("App layout-engine pagination contract", () => {
  it("renders the WASM layout-engine preview through the page-aware adapter", () => {
    const source = readFileSync(join(process.cwd(), "src", "App.tsx"), "utf8");
    const layoutBranch = source.slice(
      source.indexOf("if (useLayoutEngine)"),
      source.indexOf("import(\"./vexflow\")"),
    );

    expect(layoutBranch).toContain("renderScorePagesToSvgs");
    expect(layoutBranch).not.toContain("renderScoreToSvg");
    expect(layoutBranch).toContain('data-page="${i+1}"');
  });
});
