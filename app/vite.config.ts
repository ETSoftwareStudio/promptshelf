import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig, lazyPlugins, type Plugin } from "vite-plus";
// @ts-expect-error: a plain JS script, also run on its own.
import { thirdParty } from "./scripts/third-party.mjs";

// Settings → About → Third-party Licences imports "virtual:third-party". The
// list is made when the app is built, from Cargo.lock and node_modules. A
// build fails without it; `vp dev` shows the error in the list instead.
function thirdPartyList(): Plugin {
  const id = "virtual:third-party";
  let build = false;
  return {
    name: "promptshelf-third-party",
    configResolved: (config) => void (build = config.command === "build"),
    resolveId: (source) => (source === id ? `\0${id}` : undefined),
    load(loaded) {
      if (loaded !== `\0${id}`) return undefined;
      let list;
      try {
        list = thirdParty(import.meta.dirname);
      } catch (e) {
        if (build) throw e;
        list = { crates: [], npm: [], fonts: [], error: String(e) };
      }
      return `export default ${JSON.stringify(list)};`;
    },
  };
}

// Three pages, one per Tauri window: the library, the quick panel, Settings.
export default defineConfig({
  fmt: {},
  lint: {
    jsPlugins: [{ name: "vite-plus", specifier: "vite-plus/oxlint-plugin" }],
    rules: { "vite-plus/prefer-vite-plus-imports": "error" },
    options: { typeAware: true, typeCheck: true },
  },
  plugins: lazyPlugins(() => [svelte(), thirdPartyList()]),
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
