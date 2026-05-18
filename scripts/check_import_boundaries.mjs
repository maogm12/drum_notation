const IMPORT_RE =
  /\bimport\s+(?:[^'"]+\s+from\s+)?["']([^"']+)["']|\bexport\s+[^'"]+\s+from\s+["']([^"']+)["']|\bimport\s*\(\s*["']([^"']+)["']\s*\)/g;

export function extractImports(source) {
  const imports = [];
  for (const match of source.matchAll(IMPORT_RE)) {
    const specifier = match[1] ?? match[2] ?? match[3];
    if (specifier) {
      imports.push(specifier);
    }
  }
  return imports;
}

export function scanImportBoundaries(files, rules) {
  const violations = [];
  for (const file of files) {
    const imports = extractImports(file.source);
    for (const rule of rules) {
      if (!rule.from.test(file.path)) continue;
      for (const specifier of imports) {
        if (rule.allow?.some((allowed) => allowed.test(file.path))) continue;
        if (rule.forbidden.test(specifier)) {
          violations.push({
            file: file.path,
            import: specifier,
            rule: rule.name,
          });
        }
      }
    }
  }
  return violations;
}

export const splitWasmImportRules = [
  {
    name: "browser production must not import node wasm",
    from: /(^|\/)src\/.*\.(?:ts|tsx)$/,
    forbidden: /(?:^|\/)(?:.*_wasm_node|.*-pkg-node)(?:\/|$)/,
    allow: [/\.test\.(?:ts|tsx)$/],
  },
  {
    name: "parser-facing code must not import layout wasm",
    from: /(^|\/)src\/(?:wasm|dsl|scoreWorker|main).*\.tsx?$/,
    forbidden: /(?:^|\/)(?:layout_wasm_|layout-pkg-)/,
    allow: [/(?:integration|parity|split_wasm_wrappers)\.test\.tsx?$/],
  },
  {
    name: "cli runtime must not import browser wasm",
    from: /(^|\/)src\/cli(?:_runtime)?\.ts$/,
    forbidden: /(?:^|\/)(?:.*_wasm_browser|.*-pkg-web)(?:\/|$)/,
  },
];

export const productionSplitWasmImportRules = [
  {
    name: "browser production must not import combined or node wasm",
    from: /(^|\/)src\/(?!cli(?:_runtime)?\.ts$)(?!renderer\/svgRendererNode\.ts$)(?!wasm\/.*_wasm_node\.ts$)(?!.*\.test\.tsx?$).*\.tsx?$/,
    forbidden: /(?:drummark_wasm|wasm\/pkg|_wasm_node|pkg-node)/,
  },
  {
    name: "parser-facing production must not import layout wasm",
    from: /(^|\/)src\/(?:wasm\/skeleton|scoreWorker|dsl\/).*\.tsx?$/,
    forbidden: /(?:layout_wasm_|layout-pkg-)/,
  },
  {
    name: "cli production must not import browser wasm",
    from: /(^|\/)src\/(?:cli|cli_runtime|renderer\/svgRendererNode)\.ts$/,
    forbidden: /(?:_wasm_browser|pkg-web)/,
  },
];
