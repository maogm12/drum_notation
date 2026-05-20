import { spawn } from "node:child_process";
import { readFileSync, statSync } from "node:fs";
import { setTimeout as delay } from "node:timers/promises";
import { brotliCompressSync, gzipSync } from "node:zlib";
import { chromium } from "playwright";

const port = Number(process.env.DRUMMARK_AUDIT_PORT ?? 4173);
const origin = `http://127.0.0.1:${port}`;
const legacyRendererPattern = new RegExp(["vex", "flow"].join(""), "i");

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
    legacyRenderer: 0,
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
  if (legacyRendererPattern.test(url)) {
    ledger.legacyRenderer += 1;
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
    legacyRendererFetches: result.legacyRenderer,
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
  assert(initial.legacyRenderer === 0, "Scenario 1 must not fetch a legacy renderer chunk");

  const layout = await runScenario(browser, "first-default-layout-render", async (page) => {
    await page.goto(origin);
    await page.waitForSelector(".staff-preview-page svg");
  });
  assert(layout.layoutWasm > 0, "Scenario 2 must fetch layout WASM");
  assert(layout.legacyRenderer === 0, "Scenario 2 must not fetch a legacy renderer chunk");

  const context = await browser.newContext();
  const page = await context.newPage();
  const combined = emptyLedger();
  page.on("response", (response) => {
    void recordResponse(combined, response);
  });
  await page.goto(origin);
  await page.waitForSelector(".staff-preview-page svg");
  await page.evaluate(() => {
    window.localStorage.setItem("drummark-settings", JSON.stringify({ useLayoutEngine: false }));
  });
  await page.reload();
  await page.waitForSelector(".staff-preview-page svg");
  await page.waitForLoadState("networkidle");
  const legacyPreference = {
    name: "legacy-preference-after-default-layout",
    requests: combined.requests,
    parserWasm: combined.parserWasm,
    layoutWasm: combined.layoutWasm,
    legacyRenderer: combined.legacyRenderer,
    cumulativeTransfer: combined.cumulativeTransfer,
  };
  assert(legacyPreference.legacyRenderer === 0, "Scenario 3 must ignore legacy renderer preference without fetching a legacy chunk");
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
      scenario: legacyPreference.name,
      cumulativeTransfer: legacyPreference.cumulativeTransfer,
      parserWasmFetches: legacyPreference.parserWasm,
      layoutWasmFetches: legacyPreference.layoutWasm,
      legacyRendererFetches: legacyPreference.legacyRenderer,
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
