import { expect, test } from "vite-plus/test";
import { DRAFT_KINDS, draftLibrary, newDraft, rowTitle, typed } from "./draft";

const libs = [{ name: "personal" }, { name: "acme-eng" }];

test("the draft goes to the library in view, else Personal", () => {
  expect(draftLibrary("lib:acme-eng", libs, "personal")).toBe("acme-eng");
  expect(draftLibrary("all", libs, "personal")).toBe("personal");
  expect(draftLibrary("kind:snippet", libs, "personal")).toBe("personal");
  expect(draftLibrary("lib:gone", libs, "personal")).toBe("personal");
  expect(draftLibrary("all", libs, null)).toBe("personal");
  expect(draftLibrary("all", [], null)).toBe("");
});

test("a new draft is an empty template", () => {
  const d = newDraft("all", libs, "personal");
  expect(d).toEqual({ title: "", kind: "template", library: "personal", body: "", start: "" });
  expect(DRAFT_KINDS[0].kind).toBe("template");
  expect(DRAFT_KINDS.map((k) => k.label)).toEqual([
    "Template",
    "Snippet",
    "Context",
    "Draft (Inbox)",
  ]);
});

test("only typed text counts, not spaces, kind or library", () => {
  const d = newDraft("all", libs, "personal");
  expect(typed(d)).toBe(false);
  expect(typed({ ...d, title: "  ", body: "\n" })).toBe(false);
  expect(typed({ ...d, kind: "snippet", library: "acme-eng" })).toBe(false);
  expect(typed({ ...d, title: "Review" })).toBe(true);
  expect(typed({ ...d, body: "Text" })).toBe(true);
});

test("a title from a link is not typed text", () => {
  const d = newDraft("all", libs, "personal", "From a link");
  expect(d.title).toBe("From a link");
  expect(typed(d)).toBe(false);
  expect(typed({ ...d, title: "From a link, changed" })).toBe(true);
});

test("the row follows the title, else says Untitled", () => {
  const d = newDraft("all", libs, "personal");
  expect(rowTitle(d)).toBe("Untitled");
  expect(rowTitle({ ...d, title: "  Strict review " })).toBe("Strict review");
});
