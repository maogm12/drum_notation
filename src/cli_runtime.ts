import { buildMusicXml } from "./dsl/musicxml";
import { buildNormalizedScore } from "./dsl/normalize";
import type { ParseMode } from "./dsl";
import type { NormalizedScore } from "./dsl/types";
import { renderScoreToSvg } from "./vexflow/renderer";
import { DEFAULT_RENDER_OPTIONS, type VexflowRenderOptions } from "./vexflow/types";
import { formatScoreJson, type CliOutputFormat } from "./cli_output";
import { ensureCliRenderEnvironment } from "./cli_render_env";

export type CliParams = {
  input: string;
  format: CliOutputFormat;
  output: string | null;
  parser: ParseMode;
};

export const CLI_USAGE =
  "Usage: npm run drummark -- <input-file> [--format ast|ir|svg|xml] [--output path] [--parser wasm|lezer|regex]";

export const CLI_RENDER_OPTIONS: VexflowRenderOptions = {
  ...DEFAULT_RENDER_OPTIONS,
  pagePadding: { top: 20, right: 20, bottom: 20, left: 20 },
};

export function parseCliArgs(args: string[]): CliParams | null {
  const params: CliParams = {
    input: "",
    format: "ir",
    output: null,
    parser: "lezer",
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === "--format" && args[i + 1]) {
      params.format = args[i + 1] as CliOutputFormat;
      i++;
    } else if (arg === "--output" && args[i + 1]) {
      params.output = args[i + 1] ?? null;
      i++;
    } else if (arg === "--parser" && args[i + 1]) {
      params.parser = args[i + 1] as ParseMode;
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
  parser: ParseMode = "lezer",
): Promise<{
  score: NormalizedScore;
  result: string;
}> {
  const score = buildNormalizedScore(source, parser);

  if (format === "ast" || format === "ir") {
    return { score, result: formatScoreJson(score, format) };
  }

  if (format === "xml") {
    return { score, result: buildMusicXml(score) };
  }

  ensureCliRenderEnvironment();
  const result = await renderScoreToSvg(score, CLI_RENDER_OPTIONS);
  return { score, result };
}
