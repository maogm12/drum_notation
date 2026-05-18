import { buildMusicXml } from "./dsl/musicxml";
import { buildNormalizedScore } from "./dsl/normalize";
import type { NormalizedScore } from "./dsl/types";
import { renderSourceToSvgNode } from "./renderer/svgRendererNode";
import { DEFAULT_RENDER_OPTIONS, type ScoreRenderOptions } from "./renderer/renderOptions";
import { formatScoreJson, type CliOutputFormat } from "./cli_output";
import { ensureCliRenderEnvironment } from "./cli_render_env";
import { initParserWasmNode } from "./wasm/parser_wasm_node";

export type CliParams = {
  input: string;
  format: CliOutputFormat;
  output: string | null;
};

export const CLI_USAGE =
  "Usage: npm run drummark -- <input-file> [--format ast|ir|svg|xml] [--output path]";

export const CLI_RENDER_OPTIONS: ScoreRenderOptions = {
  ...DEFAULT_RENDER_OPTIONS,
  pagePadding: { top: 20, right: 20, bottom: 20, left: 20 },
};

export function parseCliArgs(args: string[]): CliParams | null {
  const params: CliParams = {
    input: "",
    format: "ir",
    output: null,
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === "--format" && args[i + 1]) {
      params.format = args[i + 1] as CliOutputFormat;
      i++;
    } else if (arg === "--output" && args[i + 1]) {
      params.output = args[i + 1] ?? null;
      i++;
    } else if (arg && !arg.startsWith("-")) {
      params.input = arg;
    }
  }

  return params.input ? params : null;
}

export function resolveCliOutputPath(params: CliParams): string | null {
  if (params.output) return params.output;
  if (params.format !== "xml" && params.format !== "svg") return null;
  return params.input.replace(/\.[^/.]+$/, "") + (params.format === "xml" ? ".xml" : ".svg");
}

export function formatCliWarnings(score: NormalizedScore): string[] {
  if (score.errors.length === 0) return [];
  return [
    "Parser warnings/errors:",
    ...score.errors.map((error) => `Line ${error.line}, Col ${error.column}: ${error.message}`),
  ];
}

export async function buildCliOutput(
  source: string,
  format: CliOutputFormat,
): Promise<{
  score: NormalizedScore;
  result: string;
}> {
  await initParserWasmNode();
  const score = buildNormalizedScore(source);

  if (format === "ast" || format === "ir") {
    return { score, result: formatScoreJson(score, format) };
  }

  if (format === "xml") {
    return { score, result: buildMusicXml(score) };
  }

  ensureCliRenderEnvironment();
  const result = await renderSourceToSvgNode(source, {
    staffScale: CLI_RENDER_OPTIONS.staffScale,
    pageWidth: CLI_RENDER_OPTIONS.pageWidth,
    topMargin: CLI_RENDER_OPTIONS.pagePadding.top,
    bottomMargin: CLI_RENDER_OPTIONS.pagePadding.bottom,
    leftMargin: CLI_RENDER_OPTIONS.pagePadding.left,
    rightMargin: CLI_RENDER_OPTIONS.pagePadding.right,
  });
  return { score, result };
}
