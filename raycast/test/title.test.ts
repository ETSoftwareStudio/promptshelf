import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { titleFrom } from "../src/lib/title";

describe("titleFrom", () => {
  it("takes the first line without Markdown marks", () => {
    assert.equal(titleFrom("\n# Review **this** diff\nmore"), "Review this diff");
  });

  it("keeps at most eight words", () => {
    assert.equal(titleFrom("one two three four five six seven eight nine"), "one two three four five six seven eight");
  });

  it("gives an empty title for empty text", () => {
    assert.equal(titleFrom("  "), "");
  });
});
