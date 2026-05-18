import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";
import { buildNormalizedScore } from "./src/dsl/index";
import { renderScoreToSvg } from "./src/vexflow/index";
import { highlightDslStatic } from "./src/drummark";
import { DEFAULT_RENDER_OPTIONS } from "./src/renderer/renderOptions";
import { ensureCliRenderEnvironment } from "./src/cli_render_env";
import { initParserWasmBrowserForTests } from "./src/wasm/parser_wasm_browser";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

ensureCliRenderEnvironment({ installFileReader: true });

async function buildDocs(templatePath: string, outputPath: string) {
    console.log(`Building ${outputPath} from ${templatePath}...`);
    let html = fs.readFileSync(templatePath, 'utf8');
    const lang = outputPath.includes('_zh') ? 'zh' : 'en';

    const placeholderRegex = /<div class="example-inject" data-example="([^"]+)"><\/div>/g;
    
    let match;
    const replacements: { placeholder: string, content: string }[] = [];

    // We need to collect all matches first because async/await in replace is tricky
    const matches: string[] = [];
    const ids: string[] = [];
    while ((match = placeholderRegex.exec(html)) !== null) {
        matches.push(match[0]);
        ids.push(match[1]);
    }

    for (let i = 0; i < ids.length; i++) {
        const id = ids[i];
        const placeholder = matches[i];
        console.log(`  -> Processing example: ${id}`);

        const drumFile = path.join(__dirname, 'docs', 'examples', `${id}.drum`);
        if (!fs.existsSync(drumFile)) {
            console.warn(`     Warning: Example file ${drumFile} not found.`);
            continue;
        }

        const dsl = fs.readFileSync(drumFile, 'utf8');
        const encodedDsl = encodeURIComponent(dsl);
        
        // 1. Highlight
        const highlightedDsl = highlightDslStatic(dsl);

        // 2. Render Score
        let renderedSvg = "";
        try {
            globalThis.document.body.innerHTML = '<div id="vd-container"></div>';
            const score = buildNormalizedScore(dsl);
            renderedSvg = await renderScoreToSvg(score, {
                ...DEFAULT_RENDER_OPTIONS,
            });
        } catch (e: any) {
            console.error(`     Error rendering ${id}:`, e.message);
            if (e.stack) console.error(`     ${e.stack.split('\n').slice(0,5).join('\n')}`);
            renderedSvg = `<div class="staff-error">Render Error: ${e.message}</div>`;
        }

        // 3. Construct HTML
        const exampleTitle = lang === 'zh' ? '示例' : 'Example';
        const copyLabel = lang === 'zh' ? '复制' : 'Copy';
        const resultTitle = lang === 'zh' ? '生成结果' : 'Score Result';
        
        const sectionBody = `
            <div class="docs-example-row">
                <div class="docs-section-pane">
                    <div class="docs-pane-title">${exampleTitle}</div>
                    <div class="docs-code-block">
                        <button
                            type="button"
                            class="docs-copy-button"
                            data-copy="${encodedDsl}"
                            data-copy-label="${copyLabel}"
                            aria-label="${copyLabel}"
                        >${copyLabel}</button>
                        <pre class="dsl-code-block">${highlightedDsl}</pre>
                    </div>
                </div>
                <div class="docs-section-pane">
                    <div class="docs-pane-title">${resultTitle}</div>
                    <div class="docs-preview-shell">
                        <div class="docs-preview-frame">
                            <div class="staff-preview-container">
                                <div class="staff-preview">${renderedSvg}</div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>`;
        
        replacements.push({ placeholder, content: sectionBody });
    }

    for (const r of replacements) {
        html = html.replace(r.placeholder, r.content);
    }

    fs.writeFileSync(outputPath, html);
    console.log(`Saved ${outputPath}`);
}

async function run() {
    console.log("Initializing WASM parser...");
    initParserWasmBrowserForTests(fs.readFileSync(path.join(__dirname, "src/wasm/parser-pkg-web/drummark_core_bg.wasm")));
    await buildDocs('docs.template.html', 'docs.html');
    await buildDocs('docs_zh.template.html', 'docs_zh.html');
    console.log("Build complete.");
}

run().catch(console.error);
