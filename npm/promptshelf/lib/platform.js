"use strict";

// The release archives, one per platform. The names match the release workflow.
const TARGETS = {
  "darwin-arm64": "aarch64-apple-darwin",
  "darwin-x64": "x86_64-apple-darwin",
  "linux-x64": "x86_64-unknown-linux-musl",
  "linux-arm64": "aarch64-unknown-linux-musl",
  "win32-x64": "x86_64-pc-windows-msvc",
};

function target(platform, arch) {
  const t = TARGETS[`${platform}-${arch}`];
  if (!t) {
    throw new Error(
      `PromptShelf has no build for ${platform}-${arch}. ` +
        "Build it from source: cargo install promptshelf",
    );
  }
  return t;
}

function archiveName(version, triple) {
  const ext = triple.includes("windows") ? "zip" : "tar.gz";
  return `promptshelf-v${version}-${triple}.${ext}`;
}

function binaryName(platform) {
  return platform === "win32" ? "shelf.exe" : "shelf";
}

module.exports = { TARGETS, target, archiveName, binaryName };
