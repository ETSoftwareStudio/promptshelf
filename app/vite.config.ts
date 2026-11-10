import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig, lazyPlugins } from "vite-plus";

// Three pages, one per Tauri window: the library, the quick panel, Settings.
export default defineConfig({
  fmt: {},
  lint: {
    jsPlugins: [{ name: "vite-plus", specifier: "vite-plus/oxlint-plugin" }],
    rules: { "vite-plus/prefer-vite-plus-imports": "error" },
    options: { typeAware: true, typeCheck: true },
  },
  plugins: lazyPlugins(() => [svelte()]),
  // Tauri expects a fixed port and shows its own errors.
  clearScreen: false,
  server: { port: 5173, strictPort: true },
  build: {
    target: "es2022",
    rollupOptions: {
      input: {
        library: "index.html",
        panel: "panel.html",
        settings: "settings.html",
      },
    },
  },
  test: {
    include: ["src/**/*.test.ts"],
  },
});
