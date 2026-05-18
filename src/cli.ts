import fs from "fs";
import path from "path";
import {
  buildCliOutput,
  CLI_USAGE,
  formatCliWarnings,
  parseCliArgs,
  resolveCliOutputPath,
} from "./cli_runtime";
import { initParserWasmNode } from "./wasm/parser_wasm_node";

async function main() {
  const params = parseCliArgs(process.argv.slice(2));

  if (!params) {
    console.error(CLI_USAGE);
    process.exit(1);
  }

  await initParserWasmNode();

  const source = fs.readFileSync(path.resolve(params.input), "utf-8");
  const { score, result } = await buildCliOutput(source, params.format);
  const warnings = formatCliWarnings(score);
  for (const warning of warnings) {
    console.warn(warning);
  }

  const outputPath = resolveCliOutputPath(params);
  if (outputPath) {
    fs.writeFileSync(path.resolve(outputPath), result);
    console.log(`Saved ${params.format} to ${outputPath}`);
  } else {
    console.log(result);
  }
}

main().catch(console.error);
