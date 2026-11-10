import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { blankStates, isCommand, sourceNote, tagText } from "../src/lib/blanks";
import type { FillResult } from "../src/lib/contract";

const blanks = [
  { name: "focus", from: "conversation", hint: "what the user is worried about" },
  { name: "findings", choices: ["Blockers only", "Everything"], default: "Blockers only" },
  { name: "diff", from: "git diff --staged" },
  { name: "branch", from: "branch" },
  { name: "note" },
];

const fill: FillResult = {
  id: "acme/review",
  text: "",
  complete: false,
  values: [
    { name: "findings", value: "Blockers only", source: "default" },
    { name: "diff", value: "+1 -1", source: "command", detail: "git diff --staged" },
  ],
  missing: [
    { name: "focus", from: "conversation", hint: "what the user is worried about" },
    { name: "branch", from: "branch", reason: "/ is not on a git branch" },
    { name: "note" },
  ],
};

describe("isCommand", () => {
  it("tells built-in sources from shell commands", () => {
    assert.equal(isCommand("branch"), false);
    assert.equal(isCommand("conversation"), false);
    assert.equal(isCommand("git diff --staged"), true);
  });
});

describe("blankStates", () => {
  const states = blankStates(blanks, fill);

  it("numbers blanks in the order of show", () => {
    assert.deepEqual(states.map(tagText), [
      "1 focus · conversation",
      "2 findings · default",
      "3 diff · repo",
      "4 branch · repo",
      "5 note · ask",
    ]);
  });

  it("gives each blank its tone", () => {
    assert.deepEqual(
      states.map((s) => s.tone),
      ["conversation", "neutral", "repo", "repo", "ask"],
    );
  });

  it("adds blanks that only the fill knows", () => {
    const extra = blankStates([], { ...fill, values: [], missing: [{ name: "x", from: "conversation" }] });
    assert.deepEqual(extra.map(tagText), ["1 x · conversation"]);
  });

  it("works without a fill", () => {
    assert.deepEqual(
      blankStates(blanks).map((s) => s.label),
      ["conversation", "ask", "repo", "repo", "ask"],
    );
  });
});

describe("sourceNote", () => {
  const states = blankStates(blanks, fill);
  const note = (i: number) => sourceNote(states[i]!, "/Users/me/code/app", "/Users/me");

  it("names the source of a value the core found", () => {
    assert.equal(note(2), "From git diff --staged in ~/code/app");
    assert.equal(note(1), "You choose · Blockers only, Everything");
  });

  it("explains an open blank", () => {
    assert.equal(note(0), "What the user is worried about · Raycast can not read the conversation");
    assert.equal(note(3), "Not filled: / is not on a git branch");
    assert.equal(note(4), "Type a value");
  });
});
