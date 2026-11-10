// Writes the release version and archive checksums into the npm package.
// Usage: node npm/scripts/stamp.mjs <version> <checksums.txt>
import { readFileSync, writeFileSync } from "node:fs";

const [version, sumsFile] = process.argv.slice(2);
if (!version || !sumsFile) {
  console.error("usage: stamp.mjs <version> <checksums.txt>");
  process.exit(2);
}
const sums = {};
for (const line of readFileSync(sumsFile, "utf8").split("\n")) {
  const [hash, name] = line.trim().split(/\s+/);
  if (hash && name) sums[name.replace(/^\*/, "")] = hash;
}
if (Object.keys(sums).length === 0) {
  console.error(`no checksums in ${sumsFile}`);
  process.exit(1);
}
const pkgPath = new URL("../promptshelf/package.json", import.meta.url);
const pkg = JSON.parse(readFileSync(pkgPath, "utf8"));
if (pkg.version !== version) {
  console.error(`package.json has ${pkg.version}, release is ${version}`);
  process.exit(1);
}
writeFileSync(
  new URL("../promptshelf/checksums.json", import.meta.url),
  JSON.stringify(sums, null, 2) + "\n",
);
console.log(`stamped ${Object.keys(sums).length} checksums for ${version}`);
