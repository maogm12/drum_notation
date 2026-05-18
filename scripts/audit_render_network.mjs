import { spawn } from "node:child_process";
import { readFileSync, statSync } from "node:fs";
import { setTimeout as delay } from "node:timers/promises";
import { brotliCompressSync, gzipSync } from "node:zlib";
import { chromium } from "playwright";

const port = Number(process.env.DRUMMARK_AUDIT_PORT ?? 4173);
const origin = `http://127.0.0.1:${port}`;

function startPreview() {
  const child = spawn("npx", ["vite", "preview", "--host", "127.0.0.1", "--port", String(port), "--strictPort"], {
    cwd: process.cwd(),
    stdio: ["ignore", "pipe", "pipe"],
  });
  return child;
}

async function waitForPreview() {
  for (let i = 0; i < 100; i++) {
    try {
      const response = await fetch(origin);
      if (response.ok) return;
    } catch {
      // keep polling
    }
    await delay(100);
  }
  throw new Error(`Timed out waiting for ${origin}`);
}

function emptyLedger() {
  return {
    requests: [],
    parserWasm: 0,
    layoutWasm: 0,
    vexflow: 0,
    cumulativeTransfer: 0,
  };
}

async function recordResponse(ledger, response) {
  const url = response.url();
  if (!url.startsWith(origin)) return;
  const body = await response.body().catch(() => Buffer.alloc(0));
  const size = body.byteLength;
  ledger.cumulativeTransfer += size;
  ledger.requests.push({ url, size });
  if (url.endsWith(".wasm")) {
    if (size < 200_000) ledger.parserWasm += 1;
    else ledger.layoutWasm += 1;
  }
  if (/vexflow/i.test(url)) {
    ledger.vexflow += 1;
  }
}

async function runScenario(browser, name, run) {
  const context = await browser.newContext();
  const page = await context.newPage();
  const ledger = emptyLedger();
  page.on("response", (response) => {
    void recordResponse(ledger, response);
  });
  await run(page, ledger);
  await page.waitForLoadState("networkidle");
  await context.close();
  return { name, ...ledger };
}

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function reportScenario(result) {
  return {
    scenario: result.name,
    cacheColdTransfer: result.cumulativeTransfer,
    cumulativeTransfer: result.cumulativeTransfer,
    parserWasmFetches: result.parserWasm,
    layoutWasmFetches: result.layoutWasm,
    vexflowFetches: result.vexflow,
  };
}

function assetSize(path) {
  const bytes = readFileSync(path);
  return {
    path,
    raw: statSync(path).size,
    gzip: gzipSync(bytes).length,
    brotli: brotliCompressSync(bytes).length,
  };
}

const preview = startPreview();
try {
  await waitForPreview();
  const browser = await chromium.launch({ headless: true });

  const initial = await runScenario(browser, "initial-preview-suspended", async (page) => {
    await page.goto(`${origin}/?suspendPreview=1`);
    await page.waitForSelector(".app-shell");
  });
  assert(initial.parserWasm > 0, "Scenario 1 must fetch parser WASM");
  assert(initial.layoutWasm === 0, "Scenario 1 must not fetch layout WASM");
  assert(initial.vexflow === 0, "Scenario 1 must not fetch VexFlow");

  const layout = await runScenario(browser, "first-default-layout-render", async (page) => {
    await page.goto(origin);
    await page.waitForSelector(".staff-preview-page svg");
  });
  assert(layout.layoutWasm > 0, "Scenario 2 must fetch layout WASM");
  assert(layout.vexflow === 0, "Scenario 2 must not fetch VexFlow");

  const legacy = await runScenario(browser, "first-legacy-vexflow-render", async (page) => {
    await page.addInitScript(() => {
      window.localStorage.setItem("drummark-settings", JSON.stringify({ useLayoutEngine: false }));
    });
    await page.goto(origin);
    await page.waitForSelector(".staff-preview-page svg");
  });
  assert(legacy.vexflow > 0, "Scenario 3 must fetch VexFlow");
  assert(legacy.layoutWasm === 0, "Scenario 3 must not fetch layout WASM solely for legacy render");

  const context = await browser.newContext();
  const page = await context.newPage();
  const combined = emptyLedger();
  page.on("response", (response) => {
    void recordResponse(combined, response);
  });
  await page.goto(origin);
  await page.waitForSelector(".staff-preview-page svg");
  const beforeLegacy = combined.cumulativeTransfer;
  await page.evaluate(() => {
    window.localStorage.setItem("drummark-settings", JSON.stringify({ useLayoutEngine: false }));
  });
  await page.reload();
  await page.waitForSelector(".staff-preview-page svg");
  await page.waitForLoadState("networkidle");
  const postLayoutLegacy = {
    name: "legacy-after-default-layout",
    requests: combined.requests,
    parserWasm: combined.parserWasm,
    layoutWasm: combined.layoutWasm,
    vexflow: combined.vexflow,
    cumulativeTransfer: combined.cumulativeTransfer,
    incrementalTransfer: combined.cumulativeTransfer - beforeLegacy,
  };
  assert(postLayoutLegacy.vexflow > 0, "Scenario 4 must fetch VexFlow after layout path");
  await context.close();

  await browser.close();

  const report = [
    reportScenario(initial),
    {
      ...reportScenario(layout),
      incrementalTransfer: layout.requests
        .filter((request) => request.url.endsWith(".wasm") && request.size >= 200_000)
        .reduce((sum, request) => sum + request.size, 0),
    },
    {
      ...reportScenario(legacy),
      incrementalTransfer: legacy.requests
        .filter((request) => /vexflow/i.test(request.url))
        .reduce((sum, request) => sum + request.size, 0),
    },
    {
      scenario: postLayoutLegacy.name,
      incrementalTransfer: postLayoutLegacy.incrementalTransfer,
      cumulativeTransfer: postLayoutLegacy.cumulativeTransfer,
      parserWasmFetches: postLayoutLegacy.parserWasm,
      layoutWasmFetches: postLayoutLegacy.layoutWasm,
      vexflowFetches: postLayoutLegacy.vexflow,
    },
  ];
  console.log(JSON.stringify({
    assetSizes: {
      parserWasm: assetSize("src/wasm/parser-pkg-web/drummark_core_bg.wasm"),
      layoutWasm: assetSize("src/wasm/layout-pkg-web/drummark_core_bg.wasm"),
    },
    scenarios: report,
  }, null, 2));
} finally {
  preview.kill();
}
