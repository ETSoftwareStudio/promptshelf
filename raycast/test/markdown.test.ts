import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { blankStates } from "../src/lib/blanks";
import { parseFill, type FillResult } from "../src/lib/contract";
import { escapeInline, parseTemplate, renderFilled, superscript } from "../src/lib/markdown";
import { snapshot } from "./snapshots";

describe("parseTemplate", () => {
  it("splits text, blanks and includes", () => {
    assert.deepEqual(parseTemplate("Hi {{ name }}.\n{{> house-rules}}"), [
      { kind: "text", text: "Hi " },
      { kind: "var", name: "name" },
      { kind: "text", text: ".\n" },
      { kind: "include", name: "house-rules" },
    ]);
  });

  it("keeps braces that are not a blank as text", () => {
    assert.deepEqual(parseTemplate("a {{not a blank}} b {{"), [{ kind: "text", text: "a {{not a blank}} b {{" }]);
  });
});

describe("superscript", () => {
  it("writes footnote numbers", () => {
    assert.equal(superscript(1), "¹");
    assert.equal(superscript(12), "¹²");
  });
});

describe("escapeInline", () => {
  it("escapes Markdown marks", () => {
    assert.equal(escapeInline("*a* _b_ `c` [d]"), "\\*a\\* \\_b\\_ \\`c\\` \\[d\\]");
  });
});

const BODY = "Review this. Focus on {{focus}}.\nReport {{findings}}.\n\n{{> house-rules}}\n\nThe diff:\n{{diff}}\n";
const RULES = "Name the file and line.";

function fill(values: FillResult["values"], missing: FillResult["missing"], text: string): FillResult {
  return { id: "acme/review", text, complete: missing.length === 0, values, missing };
}

const BLANKS = [
  { name: "focus", from: "conversation", hint: "what you worry about" },
  { name: "findings", choices: ["Blockers only", "Everything"], default: "Blockers only" },
  { name: "diff", from: "git diff --staged" },
];

describe("renderFilled", () => {
  it("bolds values, numbers them and asks for open blanks", () => {
    const text = `Review this. Focus on races in the retry.\nReport Blockers only.\n\n${RULES}\n\nThe diff:\n{{diff}}\n`;
    const f = fill(
      [
        { name: "focus", value: "races in the retry", source: "given" },
        { name: "findings", value: "Blockers only", source: "default" },
      ],
      [{ name: "diff", from: "git diff --staged" }],
      text,
    );
    assert.equal(
      renderFilled(BODY, text, blankStates(BLANKS, f)),
      "Review this. Focus on **races in the retry**¹.\nReport **Blockers only**².\n\n" +
        `${RULES}\n\nThe diff:\n**diff?**³\n`,
    );
  });

  it("puts a value with several lines in a code block", () => {
    const diff = "+a\n-b\n";
    const text = `Review this. Focus on x.\nReport Everything.\n\n${RULES}\n\nThe diff:\n${diff}\n`;
    const f = fill(
      [
        { name: "focus", value: "x", source: "given" },
        { name: "findings", value: "Everything", source: "given" },
        { name: "diff", value: diff, source: "command", detail: "git diff --staged" },
      ],
      [],
      text,
    );
    assert.ok(renderFilled(BODY, text, blankStates(BLANKS, f)).includes("The diff:\n³\n```\n+a\n-b\n```\n\n"));
  });

  it("keeps spaces outside the bold marks and escapes the value", () => {
    const text = "Say  *hi* !";
    const f = fill([{ name: "word", value: " *hi* ", source: "given" }], [], text);
    assert.equal(renderFilled("Say {{word}}!", text, blankStates([{ name: "word" }], f)), "Say  **\\*hi\\***¹ !");
  });

  it("returns the plain text when the template and the text do not line up", () => {
    const f = fill([{ name: "x", value: "1", source: "given" }], [], "something else");
    assert.equal(renderFilled("{{x}} items", "something else", blankStates([{ name: "x" }], f)), "something else");
  });

  it("renders the open fill of the CLI snapshot", () => {
    const open = parseFill(snapshot("fill_open"));
    const body = "Explain {{topic}} to {{audience}}. Start with what it does, then how it works.\n";
    const text = open.text.split("\n")[0] + "\n";
    const md = renderFilled(body, text, blankStates([{ name: "topic" }, { name: "audience" }], open));
    assert.equal(
      md,
      "Explain **what to explain?**¹ to **a developer new to this codebase**². Start with what it does, then how it works.\n",
    );
  });
});
