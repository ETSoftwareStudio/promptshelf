import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { candidates, findShelf } from "../src/lib/binary";
import { childPath } from "../src/lib/run";

describe("candidates", () => {
  it("looks in the usual folders by default", () => {
    assert.deepEqual(candidates(undefined, "/Users/me"), [
      "/opt/homebrew/bin/shelf",
      "/usr/local/bin/shelf",
      "/Users/me/.cargo/bin/shelf",
    ]);
  });

  it("uses only the preference when it is set", () => {
    assert.deepEqual(candidates(" ~/bin/shelf ", "/Users/me"), ["/Users/me/bin/shelf"]);
    assert.equal(candidates("", "/Users/me").length, 3);
  });
});

describe("findShelf", () => {
  it("returns the first executable candidate", () => {
    const found = findShelf(undefined, "/h", (p) => p === "/usr/local/bin/shelf" || p === "/h/.cargo/bin/shelf");
    assert.equal(found, "/usr/local/bin/shelf");
  });

  it("returns undefined when there is none", () => {
    assert.equal(
      findShelf(undefined, "/h", () => false),
      undefined,
    );
  });
});

describe("childPath", () => {
  it("puts the binary's folder and Homebrew first, without duplicates", () => {
    assert.equal(
      childPath("/h/.cargo/bin/shelf", "/usr/bin:/bin:/usr/local/bin"),
      "/h/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin",
    );
  });
});
