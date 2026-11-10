"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const test = require("node:test");

const { archiveName, binaryName, target } = require("../lib/platform");
const { cacheRoot, sha256 } = require("../lib/install");

test("maps platforms to release targets", () => {
  assert.equal(target("darwin", "arm64"), "aarch64-apple-darwin");
  assert.equal(target("linux", "x64"), "x86_64-unknown-linux-musl");
  assert.equal(target("win32", "x64"), "x86_64-pc-windows-msvc");
  assert.throws(() => target("aix", "ppc64"), /no build/);
});

test("names archives and binaries", () => {
  assert.equal(
    archiveName("0.1.0", "aarch64-apple-darwin"),
    "promptshelf-v0.1.0-aarch64-apple-darwin.tar.gz",
  );
  assert.equal(
    archiveName("0.1.0", "x86_64-pc-windows-msvc"),
    "promptshelf-v0.1.0-x86_64-pc-windows-msvc.zip",
  );
  assert.equal(binaryName("win32"), "shelf.exe");
  assert.equal(binaryName("linux"), "shelf");
});

test("uses PROMPTSHELF_CACHE first", () => {
  assert.equal(cacheRoot({ PROMPTSHELF_CACHE: "/tmp/x" }, "linux"), "/tmp/x");
  assert.equal(
    cacheRoot({ XDG_CACHE_HOME: "/c" }, "linux"),
    path.join("/c", "promptshelf"),
  );
});

test("hashes files", () => {
  const file = path.join(fs.mkdtempSync(path.join(os.tmpdir(), "ps-")), "a");
  fs.writeFileSync(file, "abc");
  assert.equal(
    sha256(file),
    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
  );
});
