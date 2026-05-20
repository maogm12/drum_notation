import { defineConfig } from "vite";
import preact from "@preact/preset-vite";
import { resolve } from "path";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [preact()],
  base: "/DrumMark/",
  appType: "mpa",
  resolve: {
    alias: {
      react: "preact/compat",
      "react-dom": "preact/compat",
      "react/jsx-runtime": "preact/jsx-runtime",
      "react-dom/client": "preact/compat/client",
    },
  },
  optimizeDeps: {
    force: true,
  },
  server: {
    host: true,
    port: 5173,
    hmr: {
      overlay: false,
    },
    watch: {
      ignored: ["!**/src/wasm/pkg/**"],
    },
  },
  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        docs: resolve(__dirname, "docs.html"),
        docs_zh: resolve(__dirname, "docs_zh.html"),
      },
    },
  },
});
