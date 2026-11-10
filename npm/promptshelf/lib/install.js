"use strict";

const crypto = require("node:crypto");
const fs = require("node:fs");
const https = require("node:https");
const os = require("node:os");
const path = require("node:path");
const { spawnSync } = require("node:child_process");

const { archiveName, binaryName, target } = require("./platform");

const REPO = "https://github.com/ETSoftwareStudio/promptshelf";

function cacheRoot(env = process.env, platform = process.platform) {
  if (env.PROMPTSHELF_CACHE) return env.PROMPTSHELF_CACHE;
  const home = os.homedir();
  if (platform === "win32") {
    return path.join(env.LOCALAPPDATA || path.join(home, "AppData", "Local"), "promptshelf");
  }
  if (platform === "darwin") return path.join(home, "Library", "Caches", "promptshelf");
  return path.join(env.XDG_CACHE_HOME || path.join(home, ".cache"), "promptshelf");
}

function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}

function download(url, dest, redirects = 5) {
  return new Promise((resolve, reject) => {
    https
      .get(url, { headers: { "User-Agent": "promptshelf-npm" } }, (res) => {
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          res.resume();
          if (redirects === 0) return reject(new Error(`too many redirects for ${url}`));
          return resolve(download(res.headers.location, dest, redirects - 1));
        }
        if (res.statusCode !== 200) {
          res.resume();
          return reject(new Error(`download failed (${res.statusCode}): ${url}`));
        }
        const out = fs.createWriteStream(dest);
        res.pipe(out);
        out.on("finish", () => out.close(resolve));
        out.on("error", reject);
      })
      .on("error", reject);
  });
}

/**
 * Returns the path of the `shelf` binary for this version, downloading it on
 * first use. The archive must match the SHA-256 in checksums.json.
 */
async function ensureBinary(version, checksums) {
  const triple = target(process.platform, process.arch);
  const dir = path.join(cacheRoot(), version, triple);
  const exe = path.join(dir, binaryName(process.platform));
  if (fs.existsSync(exe)) return exe;

  const asset = archiveName(version, triple);
  const expected = checksums[asset];
  if (!expected) {
    throw new Error(`no checksum for ${asset}; this package is broken, please report it`);
  }
  fs.mkdirSync(path.dirname(dir), { recursive: true });
  const tmp = fs.mkdtempSync(path.join(path.dirname(dir), ".download-"));
  try {
    const archive = path.join(tmp, asset);
    process.stderr.write(`promptshelf: downloading shelf ${version} for ${triple}\n`);
    await download(`${REPO}/releases/download/v${version}/${asset}`, archive);
    const actual = sha256(archive);
    if (actual !== expected) {
      throw new Error(`checksum mismatch for ${asset}: expected ${expected}, got ${actual}`);
    }
    // tar reads .tar.gz everywhere, and .zip on Windows 10 and later.
    const tar = spawnSync("tar", ["-xf", archive, "-C", tmp], { stdio: "inherit" });
    if (tar.status !== 0) throw new Error(`could not unpack ${asset}`);
    fs.rmSync(archive);
    fs.rmSync(dir, { recursive: true, force: true });
    fs.renameSync(tmp, dir);
  } finally {
    fs.rmSync(tmp, { recursive: true, force: true });
  }
  return exe;
}

module.exports = { cacheRoot, ensureBinary, sha256 };
