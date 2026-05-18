import { beforeAll, describe, expect, it } from "vitest";
import { buildNormalizedScore } from "./dsl/normalize";
import { formatScoreJson } from "./cli_output";
import { initParserWasmNode } from "./wasm/parser_wasm_node";

describe("formatScoreJson", () => {
  let score: ReturnType<typeof buildNormalizedScore>;

  beforeAll(async () => {
    await initParserWasmNode();
    score = buildNormalizedScore(`title CLI Output
time 4/4
note 1/8
grouping 2+2

HH | x - x - x - x - |
SD | - - d - - - d - |`);
  });

  it("returns the raw AST for ast output", () => {
    const parsed = JSON.parse(formatScoreJson(score, "ast"));

    expect(parsed.headers.title?.value).toBe("CLI Output");
    expect(parsed.paragraphs).toHaveLength(1);
    expect(parsed.repeatSpans).toEqual([]);
  });

  it("omits the AST envelope for ir output", () => {
    const parsed = JSON.parse(formatScoreJson(score, "ir"));

    expect(parsed.ast).toBeUndefined();
    expect(parsed.header.title).toBe("CLI Output");
    expect(parsed.measures).toHaveLength(1);
  });
});
