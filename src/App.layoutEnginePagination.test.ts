import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

describe("App layout-engine pagination contract", () => {
  it("renders preview through the page-aware layout adapter only", () => {
    const source = readFileSync(join(process.cwd(), "src", "App.tsx"), "utf8");

    expect(source).toContain("renderScorePagesToSvgs");
    const legacyDynamicImport = `import("${[".", "vex", "flow"].join("/")}")`;
    expect(source).not.toContain(legacyDynamicImport);
    expect(source).not.toContain("useLayoutEngine");
    expect(source).not.toContain("renderScoreToSvg");
    expect(source).toContain('data-page="${i+1}"');
  });
});
