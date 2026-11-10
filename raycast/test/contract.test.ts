import assert from "node:assert/strict";
import { describe, it } from "node:test";

import {
  ContractError,
  parseEnvelope,
  parseError,
  parseFill,
  parseLibraries,
  parsePromptList,
  parseSaved,
  parseShow,
  pinsFirst,
} from "../src/lib/contract";
import { snapshot } from "./snapshots";

describe("parseEnvelope", () => {
  it("rejects text that is not JSON", () => {
    assert.throws(() => parseEnvelope("shelf: error"), ContractError);
  });

  it("rejects another schema version", () => {
    assert.throws(() => parseEnvelope('{"schema": 2}'), /schema 2/);
  });

  it("rejects a list", () => {
    assert.throws(() => parseEnvelope("[]"), ContractError);
  });
});

describe("the CLI snapshots", () => {
  it("parses list", () => {
    const prompts = parsePromptList(snapshot("list"));
    assert.equal(prompts.length, 5);
    const p = prompts[1]!;
    assert.equal(p.id, "examples/strict-pr-review");
    assert.equal(p.library, "examples");
    assert.equal(p.kind, "template");
    assert.equal(p.title, "Strict PR review");
    assert.deepEqual(p.blanks, ["focus", "findings", "diff"]);
    assert.deepEqual(p.tags, ["review", "code"]);
    assert.equal(p.pinned, undefined);
  });

  it("parses search with its score", () => {
    const [hit] = parsePromptList(snapshot("search"));
    assert.equal(hit?.id, "examples/explain-like-a-colleague");
    assert.equal(hit?.score, 8);
  });

  it("parses show with blanks in order", () => {
    const p = parseShow(snapshot("show"));
    assert.match(p.body, /\{\{focus\}\}/);
    assert.deepEqual(
      p.blanks.map((b) => b.name),
      ["focus", "findings", "diff"],
    );
    assert.deepEqual(p.blanks[1], {
      name: "findings",
      choices: ["Blockers only", "Everything"],
      default: "Blockers only",
      from: undefined,
      hint: undefined,
      pattern: undefined,
    });
  });

  it("parses a complete fill", () => {
    const f = parseFill(snapshot("fill_complete"));
    assert.equal(f.complete, true);
    assert.deepEqual(f.missing, []);
    assert.deepEqual(
      f.values.map((v) => [v.name, v.source]),
      [
        ["topic", "given"],
        ["audience", "default"],
      ],
    );
    assert.equal(f.instruction, undefined);
  });

  it("parses a fill with open blanks", () => {
    const f = parseFill(snapshot("fill_open"));
    assert.equal(f.complete, false);
    assert.equal(f.missing[0]?.name, "topic");
    assert.equal(f.missing[0]?.from, "conversation");
    assert.equal(f.missing[0]?.hint, "what to explain");
    assert.match(f.text, /\{\{topic\}\}/);
    assert.match(f.instruction ?? "", /conversation/);
  });

  it("parses the error of a missing prompt", () => {
    assert.deepEqual(parseError(snapshot("not_found")), {
      code: "not_found",
      message: 'no prompt with the ID "nope"',
      matches: undefined,
    });
  });
});

describe("other commands", () => {
  it("parses libs list", () => {
    const libs = parseLibraries(
      JSON.stringify({
        schema: 1,
        libraries: [
          { name: "personal", path: "/p", personal: true, trusted: true, prompts: 3, remote: null, writes: "commit" },
          {
            name: "acme",
            path: "/a",
            personal: false,
            trusted: false,
            prompts: 9,
            remote: "x",
            writes: "pull-request",
          },
        ],
      }),
    );
    assert.deepEqual(
      libs.map((l) => [l.name, l.personal, l.prompts]),
      [
        ["personal", true, 3],
        ["acme", false, 9],
      ],
    );
  });

  it("parses save", () => {
    const saved = parseSaved('{"schema":1,"id":"personal/idea","kind":"draft","path":"/p/inbox/idea.md"}');
    assert.deepEqual(saved, { id: "personal/idea", kind: "draft", path: "/p/inbox/idea.md" });
  });

  it("reads the matches of an ambiguous ID", () => {
    const body = parseError('{"schema":1,"error":{"code":"ambiguous","message":"two","matches":["a/x","b/x"]}}');
    assert.deepEqual(body?.matches, ["a/x", "b/x"]);
  });

  it("returns no error for output without one", () => {
    assert.equal(parseError("not json"), undefined);
    assert.equal(parseError('{"schema":1}'), undefined);
  });

  it("names the field that breaks the contract", () => {
    assert.throws(() => parsePromptList('{"schema":1,"prompts":[{"id":1}]}'), /"id" is not a string/);
  });
});

describe("pinsFirst", () => {
  it("orders pins by position and keeps the rest in order", () => {
    const { pinned, rest } = pinsFirst([{ id: "a" }, { id: "b", pinned: 2 }, { id: "c" }, { id: "d", pinned: 1 }]);
    assert.deepEqual(
      pinned.map((p) => p.id),
      ["d", "b"],
    );
    assert.deepEqual(
      rest.map((p) => p.id),
      ["a", "c"],
    );
  });
});
