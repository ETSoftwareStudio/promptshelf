import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { Exit, isResult, problemFor } from "../src/lib/exit";

describe("isResult", () => {
  it("reads stdout for complete and open fills only", () => {
    assert.deepEqual([0, 1, 2, 3, 4, 5].filter(isResult), [Exit.Ok, Exit.BlanksOpen]);
  });
});

describe("problemFor", () => {
  it("maps each exit code to a title", () => {
    assert.equal(problemFor(1).title, "shelf failed");
    assert.equal(problemFor(2).title, "shelf did not accept the input");
    assert.equal(problemFor(3).title, "Some blanks are open");
    assert.equal(problemFor(4).title, "Prompt not found");
    assert.equal(problemFor(5).title, "shelf refused the change");
    assert.equal(problemFor(101).title, "shelf stopped with exit code 101");
  });

  it("uses the JSON error message", () => {
    assert.deepEqual(problemFor(4, { code: "not_found", message: 'no prompt with the ID "nope"' }), {
      title: "Prompt not found",
      message: 'no prompt with the ID "nope"',
    });
  });

  it("lists the matches of an ambiguous ID", () => {
    const p = problemFor(4, { code: "ambiguous", message: "x", matches: ["personal/x", "acme/x"] });
    assert.deepEqual(p, { title: "More than one prompt matches", message: "personal/x, acme/x" });
  });

  it("falls back to the last line of stderr", () => {
    assert.equal(problemFor(1, undefined, "warning\nshelf: error: no config found\n").message, "no config found");
  });
});
