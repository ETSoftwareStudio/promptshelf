#!/usr/bin/env node
"use strict";

// Runs the native `shelf` binary, downloading it on first use.

const { spawnSync } = require("node:child_process");
const { ensureBinary } = require("../lib/install");
const pkg = require("../package.json");
const checksums = require("../checksums.json");

ensureBinary(pkg.version, checksums)
  .then((exe) => {
    const result = spawnSync(exe, process.argv.slice(2), { stdio: "inherit" });
    if (result.error) throw result.error;
    if (result.signal) process.kill(process.pid, result.signal);
    process.exit(result.status === null ? 1 : result.status);
  })
  .catch((err) => {
    process.stderr.write(`promptshelf: ${err.message}\n`);
    process.exit(1);
  });
