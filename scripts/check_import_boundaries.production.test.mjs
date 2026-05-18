import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { describe, expect, it } from "vitest";
import {
  productionSplitWasmImportRules,
  scanImportBoundaries,
} from "./check_import_boundaries.mjs";

const ROOT = process.cwd();

function collectSourceFiles(dir) {
  const files = [];
  for (const entry of readdirSync(dir)) {
    const path = join(dir, entry);
    const stat = statSync(path);
    if (stat.isDirectory()) {
      if (entry.endsWith("-pkg-web") || entry.endsWith("-pkg-node") || entry === "pkg") {
        continue;
      }
      files.push(...collectSourceFiles(path));
    } else if (/\.(?:ts|tsx)$/.test(entry)) {
      files.push({
        path: relative(ROOT, path),
        source: readFileSync(path, "utf8"),
      });
    }
  }
  return files;
}

describe("production split wasm import boundaries", () => {
  it("keeps active production imports on their side of the split", () => {
    const violations = scanImportBoundaries(
      collectSourceFiles(join(ROOT, "src")),
      productionSplitWasmImportRules,
    );

    expect(violations).toEqual([]);
  });
});
