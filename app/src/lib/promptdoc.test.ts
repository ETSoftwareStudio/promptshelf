import { expect, test } from "vite-plus/test";
import {
  blankNames,
  blankType,
  freeName,
  readFields,
  removeBlank,
  renameBlank,
  split,
  tokens,
  varsToWrite,
  writeFields,
} from "./promptdoc";

const FILE = `---
title: Strict review
# a comment the app must keep
kind: template
tags: [review]
vars:
  diff:
    from: git diff --staged
owner: someone
---

Review {{language}} code. {{> house-rules}} The diff is {{diff}}. {{ not a blank }}
`;

test("tokens match the core's rules", () => {
  const t = tokens("a {{x}} {{ y-2 }} {{> rules}} {{2bad}} {{a b}} {{>}} {{");
  expect(t.map((x) => [x.kind, x.name])).toEqual([
    ["blank", "x"],
    ["blank", "y-2"],
    ["include", "rules"],
  ]);
});

test("blank names keep text order and appear once", () => {
  expect(blankNames("{{b}} {{a}} {{b}} {{> a}}")).toEqual(["b", "a"]);
});

test("rename changes blanks, not includes", () => {
  expect(renameBlank("{{a}} {{ a }} {{> a}} {{ab}}", "a", "z")).toBe("{{z}} {{z}} {{> a}} {{ab}}");
});

test("blank type comes from the spec", () => {
  expect(blankType(undefined)).toBe("text");
  expect(blankType({ choices: ["x"] })).toBe("choice");
  expect(blankType({ from: "branch", choices: ["x"] })).toBe("auto");
});

test("split skips one blank line after the frontmatter", () => {
  expect(split(FILE).gap).toBe("\n");
  expect(split(FILE).body.startsWith("Review")).toBe(true);
  expect(split("no frontmatter")).toEqual({ yaml: null, gap: "", body: "no frontmatter" });
});

test("read gives title, tags, vars and text", () => {
  const f = readFields(FILE);
  expect(f.title).toBe("Strict review");
  expect(f.tags).toEqual(["review"]);
  expect(f.vars).toEqual({ diff: { from: "git diff --staged" } });
});

test("an unchanged file is written back as it was", () => {
  expect(writeFields(FILE, readFields(FILE))).toBe(FILE);
});

test("a text change keeps the frontmatter text", () => {
  const f = readFields(FILE);
  const out = writeFields(FILE, { ...f, body: "New text {{diff}}\n" });
  expect(out).toBe(FILE.replace(/\n\nReview[\s\S]*$/, "\n\nNew text {{diff}}\n"));
});

test("new blank settings keep comments and unknown fields", () => {
  const f = readFields(FILE);
  const out = writeFields(FILE, {
    ...f,
    vars: { ...f.vars, language: { choices: ["Rust", "Go"], default: "Rust" } },
  });
  expect(out).toContain("# a comment the app must keep");
  expect(out).toContain("owner: someone");
  expect(out).toContain("choices: [Rust, Go]");
  expect(readFields(out).vars.language).toEqual({ default: "Rust", choices: ["Rust", "Go"] });
  // Blanks in the text come first, in text order.
  expect(Object.keys(readFields(out).vars)).toEqual(["language", "diff"]);
});

test("empty settings and empty tags are removed", () => {
  const f = readFields(FILE);
  const out = writeFields(FILE, { ...f, tags: [], vars: { diff: { hint: " " } } });
  expect(out).not.toContain("tags");
  expect(out).not.toContain("vars");
});

test("settings of a blank not in the text stay until removed", () => {
  expect(varsToWrite("{{a}}", { a: {}, b: { hint: "kept" } })).toEqual({ b: { hint: "kept" } });
});

test("a file without frontmatter gets one only when needed", () => {
  const plain = "Just {{x}}";
  expect(writeFields(plain, { title: "", tags: [], vars: {}, body: plain })).toBe(plain);
  expect(writeFields(plain, { title: "T", tags: [], vars: {}, body: plain })).toBe(
    "---\ntitle: T\n---\n\nJust {{x}}",
  );
});

test("broken YAML throws", () => {
  expect(() => readFields("---\ntitle: [\n---\nx")).toThrow();
});

test("remove takes the blank and one space out", () => {
  expect(removeBlank("Use {{a}} and {{a}}. Then {{b}}", "a")).toBe("Use and. Then {{b}}");
  expect(removeBlank("{{a}} first", "a")).toBe(" first");
});

test("free names count up", () => {
  expect(freeName([])).toBe("blank");
  expect(freeName(["blank", "blank-2"])).toBe("blank-3");
});
