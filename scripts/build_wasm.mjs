import { brotliCompressSync, gzipSync } from "node:zlib";
import { existsSync, mkdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { homedir } from "node:os";

const home = homedir();
for (const dir of [
  resolve(home, ".cargo/bin"),
  resolve(home, "brew/bin"),
  "/usr/local/bin",
]) {
  if (existsSync(dir)) {
    process.env.PATH = `${dir}:${process.env.PATH ?? ""}`;
  }
}

const repoRoot = resolve(new URL("..", import.meta.url).pathname);
const coreManifest = resolve(repoRoot, "crates/drummark-core/Cargo.toml");
const wasmArtifact = resolve(
  repoRoot,
  "target/wasm32-unknown-unknown/release/drummark_core.wasm",
);
const packages = [
  {
    name: "parser-web",
    target: "web",
    feature: "parser-wasm",
    outDir: resolve(repoRoot, "src/wasm/parser-pkg-web"),
    requiredExports: ["parse"],
    forbiddenExports: ["build_layout_scene"],
  },
  {
    name: "parser-node",
    target: "nodejs",
    feature: "parser-wasm",
    outDir: resolve(repoRoot, "src/wasm/parser-pkg-node"),
    requiredExports: ["parse"],
    forbiddenExports: ["build_layout_scene"],
  },
  {
    name: "layout-web",
    target: "web",
    feature: "layout-wasm",
    outDir: resolve(repoRoot, "src/wasm/layout-pkg-web"),
    requiredExports: ["build_layout_scene"],
    forbiddenExports: ["parse", "build_normalized_score", "build_render_score"],
  },
  {
    name: "layout-node",
    target: "nodejs",
    feature: "layout-wasm",
    outDir: resolve(repoRoot, "src/wasm/layout-pkg-node"),
    requiredExports: ["build_layout_scene"],
    forbiddenExports: ["parse", "build_normalized_score", "build_render_score"],
  },
];

function run(cmd, args, extraEnv = {}) {
  const result = spawnSync(cmd, args, {
    cwd: repoRoot,
    stdio: "inherit",
    env: { ...process.env, ...extraEnv },
  });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

function fail(message) {
  console.error(message);
  process.exit(1);
}

function capture(cmd, args) {
  const result = spawnSync(cmd, args, {
    cwd: repoRoot,
    stdio: ["ignore", "pipe", "pipe"],
    env: process.env,
    encoding: "utf8",
  });
  if (result.status !== 0) {
    return null;
  }
  return result.stdout.trim();
}

function resolveCargo() {
  const configured = process.env.CARGO;
  if (configured) {
    return configured;
  }

  const rustupCargo = capture("rustup", ["which", "cargo"]);
  if (rustupCargo && existsSync(rustupCargo)) {
    return rustupCargo;
  }

  return "cargo";
}

function resolveRustc(cargoPath) {
  const configured = process.env.RUSTC;
  if (configured) {
    return configured;
  }

  if (cargoPath.endsWith("/cargo")) {
    const siblingRustc = cargoPath.slice(0, -"/cargo".length) + "/rustc";
    if (existsSync(siblingRustc)) {
      return siblingRustc;
    }
  }

  const rustupRustc = capture("rustup", ["which", "rustc"]);
  if (rustupRustc && existsSync(rustupRustc)) {
    return rustupRustc;
  }

  return "rustc";
}

function resolveWasmBindgen() {
  const configured = process.env.WASM_BINDGEN;
  if (configured) {
    return configured;
  }

  const cargoHome = process.env.CARGO_HOME || (process.env.HOME ? resolve(process.env.HOME, ".cargo") : null);
  if (cargoHome) {
    const candidate = resolve(cargoHome, "bin/wasm-bindgen");
    if (existsSync(candidate)) {
      return candidate;
    }
  }

  return "wasm-bindgen";
}

const cargoPath = resolveCargo();
const rustcPath = resolveRustc(cargoPath);
const wasmBindgenPath = resolveWasmBindgen();

const parserTree = capture(cargoPath, [
  "tree",
  "--manifest-path",
  coreManifest,
  "--target",
  "wasm32-unknown-unknown",
  "--no-default-features",
  "--features",
  "parser-wasm",
  "-e",
  "normal",
]);

if (parserTree?.includes("drummark-layout")) {
  fail("parser-wasm dependency tree must not include drummark-layout");
}

const sizeReport = [];

for (const pkg of packages) {
  rmSync(pkg.outDir, { force: true, recursive: true });
  mkdirSync(pkg.outDir, { recursive: true });

  run(cargoPath, [
    "build",
    "--manifest-path",
    coreManifest,
    "--target",
    "wasm32-unknown-unknown",
    "--release",
    "--no-default-features",
    "--features",
    pkg.feature,
  ], {
    RUSTC: rustcPath,
  });

  run(wasmBindgenPath, [
    "--target",
    pkg.target,
    "--out-dir",
    pkg.outDir,
    "--omit-default-module-path",
    wasmArtifact,
  ]);

  if (pkg.target === "nodejs") {
    writeFileSync(
      resolve(pkg.outDir, "package.json"),
      JSON.stringify({ type: "commonjs" }, null, 2) + "\n",
    );
  }

  const declarationPath = resolve(pkg.outDir, "drummark_core.d.ts");
  const declarations = readFileSync(declarationPath, "utf8");
  for (const exportName of pkg.requiredExports) {
    if (!declarations.includes(`function ${exportName}`)) {
      fail(`${pkg.name} declarations are missing required export ${exportName}`);
    }
  }
  for (const exportName of pkg.forbiddenExports) {
    if (declarations.includes(`function ${exportName}`)) {
      fail(`${pkg.name} declarations unexpectedly expose ${exportName}`);
    }
  }

  const wasmPath = resolve(pkg.outDir, "drummark_core_bg.wasm");
  const bytes = readFileSync(wasmPath);
  sizeReport.push({
    package: pkg.name,
    path: wasmPath,
    rawBytes: statSync(wasmPath).size,
    gzipBytes: gzipSync(bytes).length,
    brotliBytes: brotliCompressSync(bytes).length,
  });
}

console.log("\nWASM asset size report");
for (const entry of sizeReport) {
  console.log(
    `${entry.package}: raw=${entry.rawBytes} gzip=${entry.gzipBytes} brotli=${entry.brotliBytes} path=${entry.path}`,
  );
}
