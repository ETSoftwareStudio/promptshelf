// Fails when an npm package has a licence that is not permissive.
// Usage: pnpm licenses list --json | node scripts/licences.mjs
//        pnpm licenses list --prod --json | node scripts/licences.mjs --prod
//
// The app ships only its production packages. Build tools may also use
// lightningcss (MPL-2.0, Vite's CSS minifier); its code never ships, only
// the CSS it writes.

import { readFileSync } from "node:fs";

const ALLOWED = new Set([
  "MIT",
  "Apache-2.0",
  "Apache-2.0 OR MIT",
  "MIT OR Apache-2.0",
  "BSD-2-Clause",
  "BSD-3-Clause",
  "ISC",
  "0BSD",
  "Zlib",
  "Unicode-3.0",
  "CC0-1.0",
  "BlueOak-1.0.0",
]);
const BUILD_ONLY = new Map([
  ["lightningcss", "MPL-2.0"],
  ["lightningcss-darwin-arm64", "MPL-2.0"],
  ["lightningcss-darwin-x64", "MPL-2.0"],
  ["lightningcss-linux-x64-gnu", "MPL-2.0"],
  ["lightningcss-linux-x64-musl", "MPL-2.0"],
  ["lightningcss-linux-arm64-gnu", "MPL-2.0"],
  ["lightningcss-win32-x64-msvc", "MPL-2.0"],
  ["lightningcss-win32-arm64-msvc", "MPL-2.0"],
]);

const prod = process.argv.includes("--prod");
const list = JSON.parse(readFileSync(0, "utf8"));
const bad = [];
for (const [licence, packages] of Object.entries(list)) {
  for (const p of packages) {
    if (ALLOWED.has(licence) || (!prod && BUILD_ONLY.get(p.name) === licence)) continue;
    bad.push(`${p.name}@${p.versions?.join(",") ?? "?"}: ${licence}`);
  }
}
if (bad.length) {
  console.error(`Licences that are not allowed:\n  ${bad.join("\n  ")}`);
  process.exit(1);
}
console.log(`licences ok (${Object.values(list).flat().length} packages)`);
