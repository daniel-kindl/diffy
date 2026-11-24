import { readFileSync } from "node:fs";
import { defineConfig } from "vite";

const monaco = JSON.parse(readFileSync("node_modules/monaco-editor/package.json", "utf8"));

export default defineConfig({
  clearScreen: false,
  define: { __MONACO_VERSION__: JSON.stringify(monaco.version) },
  server: { port: 1420, strictPort: true, watch: { ignored: ["**/src-tauri/**", "**/fixtures/out/**", "**/bench-results/**"] } },
  worker: { format: "es" },
  build: { outDir: "dist", emptyOutDir: true, sourcemap: false },
});
