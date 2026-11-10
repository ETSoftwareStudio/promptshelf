// Lists the crates and npm packages the desktop app ships, with each one's
// licence, for Settings → About → Third-party Licences.
//
// The Vite build calls thirdParty() (see vite.config.ts), so the list always
// follows app/src-tauri/Cargo.lock and the installed npm packages. Nobody
// edits a list by hand. Run it alone to print the list:
//   node scripts/third-party.mjs
//
// Crates: every crate from crates.io that the app links on the five release
// targets. Build and dev dependencies are left out; they do not ship.
// npm: the production dependencies, plus the parts of Svelte that the
// compiler puts into the bundle.

import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, realpathSync } from "node:fs";
import { basename, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const TARGETS = [
  "aarch64-apple-darwin",
  "x86_64-apple-darwin",
  "x86_64-pc-windows-msvc",
  "x86_64-unknown-linux-gnu",
  "aarch64-unknown-linux-gnu",
];

// Svelte's runtime and the two packages it imports at run time. The rest of
// Svelte's dependencies are the compiler, which does not ship.
const SVELTE_RUNTIME = ["svelte", "clsx", "esm-env"];

// Bundled files that are not packages.
const FONTS = [
  {
    name: "Source Serif 4",
    licence: "OFL-1.1",
    source: "https://github.com/adobe-fonts/source-serif",
  },
];

const SAFE = /^[A-Za-z0-9_.+-]+$/;

/** MPL-2.0 asks for a notice of where the source code is. The link is for
 * the exact version, built only from the crate's name and version. */
function sourceLink(name, version, licence) {
  if (!licence.includes("MPL")) return undefined;
  if (!SAFE.test(name) || !SAFE.test(version))
    throw new Error(`odd crate name: ${name} ${version}`);
  return `https://docs.rs/crate/${name}/${version}/source/`;
}

export function crates(appDir) {
  const args = ["metadata", "--locked", "--format-version", "1"];
  for (const t of TARGETS) args.push("--filter-platform", t);
  const meta = JSON.parse(
    execFileSync("cargo", args, {
      cwd: join(appDir, "src-tauri"),
      encoding: "utf8",
      maxBuffer: 256 << 20,
    }),
  );
  const packages = new Map(meta.packages.map((p) => [p.id, p]));
  const nodes = new Map(meta.resolve.nodes.map((n) => [n.id, n]));
  const seen = new Set();
  const todo = [meta.resolve.root];
  while (todo.length) {
    const id = todo.pop();
    if (seen.has(id)) continue;
    seen.add(id);
    for (const dep of nodes.get(id).deps) {
      // `kind` is null for a normal dependency, "build" or "dev" otherwise.
      if (dep.dep_kinds.some((k) => k.kind === null)) todo.push(dep.pkg);
    }
  }
  return [...seen]
    .map((id) => packages.get(id))
    .filter((p) => p.source) // our own path crates are MIT, in this repo
    .map((p) => {
      const licence = p.license ?? "see the crate";
      return {
        name: p.name,
        version: p.version,
        licence,
        source: sourceLink(p.name, p.version, licence),
      };
    })
    .sort((a, b) => a.name.localeCompare(b.name) || a.version.localeCompare(b.version));
}

/** Finds a package the way Node does: in node_modules next to the folder
 * that needs it, then further up. With pnpm, a package's own dependencies
 * sit next to its real folder. */
function findPackage(from, name) {
  for (let dir = from; ; dir = dirname(dir)) {
    const here =
      basename(dir) === "node_modules" ? join(dir, name) : join(dir, "node_modules", name);
    if (existsSync(join(here, "package.json"))) return realpathSync(here);
    if (dirname(dir) === dir) throw new Error(`could not find the npm package ${name}`);
  }
}

export function npm(appDir) {
  const app = JSON.parse(readFileSync(join(appDir, "package.json"), "utf8"));
  const read = (dir) => JSON.parse(readFileSync(join(dir, "package.json"), "utf8"));
  const svelteDir = findPackage(appDir, "svelte");
  const svelte = read(svelteDir);
  for (const name of SVELTE_RUNTIME.slice(1)) {
    if (!svelte.dependencies?.[name])
      throw new Error(`svelte no longer depends on ${name}; update SVELTE_RUNTIME`);
  }
  const seen = new Map();
  const todo = [
    ...Object.keys(app.dependencies ?? {}).map((name) => [appDir, name]),
    ...SVELTE_RUNTIME.map((name) => [svelteDir, name]),
  ];
  while (todo.length) {
    const [from, name] = todo.pop();
    if (seen.has(name)) continue;
    const dir = findPackage(from, name);
    const p = read(dir);
    seen.set(name, { name, version: p.version, licence: p.license ?? "see the package" });
    if (!SVELTE_RUNTIME.includes(name))
      todo.push(...Object.keys(p.dependencies ?? {}).map((d) => [dir, d]));
  }
  return [...seen.values()].sort((a, b) => a.name.localeCompare(b.name));
}

export function thirdParty(appDir) {
  return { crates: crates(appDir), npm: npm(appDir), fonts: FONTS };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const list = thirdParty(join(dirname(fileURLToPath(import.meta.url)), ".."));
  for (const [group, rows] of Object.entries(list)) {
    console.log(`${group} (${rows.length})`);
    for (const r of rows)
      console.log(`  ${r.name} ${r.version ?? ""}  ${r.licence}${r.source ? `  ${r.source}` : ""}`);
  }
}
