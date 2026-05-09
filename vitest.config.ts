import { defineConfig } from "vitest/config";
import preact from "@preact/preset-vite";

export default defineConfig({
  plugins: [preact()],
  resolve: {
    alias: {
      react: "preact/compat",
      "react-dom": "preact/compat",
      "react/jsx-runtime": "preact/jsx-runtime",
      "react-dom/client": "preact/compat/client",
    },
  },
  test: {
    server: {
      deps: {
        inline: [/@radix-ui\/.*/],
      },
    },
  },
});
