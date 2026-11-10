// Runs the client against a real `shelf` binary and the sample library.
// Set PROMPTSHELF_TEST_BIN to the binary to run it:
//   cargo build -p promptshelf
//   PROMPTSHELF_TEST_BIN=$PWD/../target/debug/shelf npm test

import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { after, before, describe, it } from "node:test";

import { createClient, ShelfCommandError } from "../src/lib/client";

const BIN = process.env.PROMPTSHELF_TEST_BIN;
const EXAMPLES = join(__dirname, "..", "..", "examples", "library");
const shelf = createClient(() => ({ shelfPath: BIN, workingFolder: tmpdir() }));

async function exitCodeOf(promise: Promise<unknown>): Promise<number | undefined> {
  try {
    await promise;
    return undefined;
  } catch (error) {
    assert.ok(error instanceof ShelfCommandError);
    return error.exitCode;
  }
}

describe("shelf client", { skip: !BIN && "set PROMPTSHELF_TEST_BIN to run" }, () => {
  let dir = "";

  before(() => {
    dir = mkdtempSync(join(tmpdir(), "promptshelf-raycast-"));
    process.env.PROMPTSHELF_CONFIG_DIR = join(dir, "config");
    process.env.PROMPTSHELF_STATE_DIR = join(dir, "state");
    const person = { ...process.env, PROMPTSHELF_AGENT: "0" };
    execFileSync(BIN!, ["init", "--path", join(dir, "personal")], { env: person });
    execFileSync(BIN!, ["libs", "add", EXAMPLES, "--name", "examples"], { env: person });
  });

  after(() => {
    if (dir) rmSync(dir, { recursive: true, force: true });
  });

  it("lists, searches and shows", async () => {
    assert.deepEqual(
      (await shelf.listLibraries()).map((l) => l.name),
      ["personal", "examples"],
    );
    assert.equal((await shelf.listPrompts("examples")).length, 5);
    const hits = await shelf.searchPrompts("-review strict");
    assert.equal(hits[0]?.id, "examples/strict-pr-review");
    const show = await shelf.showPrompt("strict-pr-review");
    assert.deepEqual(
      show.blanks.map((b) => b.name),
      ["focus", "findings", "diff"],
    );
  });

  it("fills with open blanks, then with every value", async () => {
    const open = await shelf.fillPrompt("explain-like-a-colleague", { run: false });
    assert.equal(open.complete, false);
    const done = await shelf.fillPrompt("explain-like-a-colleague", { values: { topic: "the sync loop" } });
    assert.equal(done.complete, true);
    assert.match(done.text, /Explain the sync loop/);
  });

  it("maps a bad choice and a missing prompt to exit codes", async () => {
    assert.equal(await exitCodeOf(shelf.fillPrompt("strict-pr-review", { values: { findings: "Some" } })), 2);
    assert.equal(await exitCodeOf(shelf.showPrompt("nope")), 4);
  });

  it("pins as a person and saves a draft from stdin", async () => {
    await shelf.setPinned("strict-pr-review", true);
    const pinned = (await shelf.listPrompts()).find((p) => p.id === "examples/strict-pr-review");
    assert.equal(pinned?.pinned, 1);
    await shelf.setPinned("strict-pr-review", false);
    const saved = await shelf.saveDraft("-Rough idea", "Draft text with {{blank}}.\n");
    assert.equal(saved.id, "personal/rough-idea");
    assert.equal(saved.kind, "draft");
  });
});
