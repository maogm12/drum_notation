import { describe, expect, it } from "vitest";
import {
  buildCliOutput,
  CLI_USAGE,
  formatCliWarnings,
  parseCliArgs,
  resolveCliOutputPath,
} from "./cli_runtime";

const SIMPLE_SOURCE = `title CLI Output
time 4/4
note 1/8
grouping 2+2

HH | x - x - x - x - |
SD | - - d - - - d - |`;

describe("cli runtime", () => {
  it("parses input, format, and output flags", () => {
    expect(parseCliArgs(["score.drum", "--format", "svg", "--output", "score.svg"])).toEqual({
      input: "score.drum",
      format: "svg",
      output: "score.svg",
      parser: "lezer",
    });
  });

  it("returns null and uses the shared usage string when input is missing", () => {
    expect(parseCliArgs(["--format", "ast"])).toBeNull();
    expect(CLI_USAGE).toContain("npm run drummark");
  });

  it("derives default output paths only for file outputs", () => {
    expect(resolveCliOutputPath({ input: "score.drum", format: "xml", output: null, parser: "lezer" })).toBe("score.xml");
    expect(resolveCliOutputPath({ input: "score.drum", format: "svg", output: null, parser: "lezer" })).toBe("score.svg");
    expect(resolveCliOutputPath({ input: "score.drum", format: "ir", output: null, parser: "lezer" })).toBeNull();
  });

  it("formats warnings from normalized parser errors", async () => {
    const { score } = await buildCliOutput(`time 4/4
divisions 4

HH | %% |`, "ast");

    expect(formatCliWarnings(score)).toEqual([
      "Parser warnings/errors:",
      "Line 4, Col 1: Measure repeat at bar 1 does not have 2 preceding measure(s)",
    ]);
  });

  it("builds AST output", async () => {
    const { result } = await buildCliOutput(SIMPLE_SOURCE, "ast");
    const parsed = JSON.parse(result);

    expect(parsed.headers.title?.value).toBe("CLI Output");
    expect(parsed.paragraphs).toHaveLength(1);
  });

  it("builds IR output", async () => {
    const { result } = await buildCliOutput(SIMPLE_SOURCE, "ir");
    const parsed = JSON.parse(result);

    expect(parsed.ast).toBeUndefined();
    expect(parsed.header.title).toBe("CLI Output");
    expect(parsed.measures).toHaveLength(1);
  });

  it("builds MusicXML output", async () => {
    const { result } = await buildCliOutput(SIMPLE_SOURCE, "xml");

    expect(result).toContain("<score-partwise");
    expect(result).toContain("<part-name>Drumset</part-name>");
  });

  it("builds SVG output through the shared render bootstrap", async () => {
    const { result } = await buildCliOutput(SIMPLE_SOURCE, "svg");

    expect(result).toContain("<svg");
    expect(result).toContain("vf-stavenote");
  });
});
