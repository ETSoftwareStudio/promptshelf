import { describe, expect, test } from "vite-plus/test";
import { annotate, blankLabel, shortValue, sourceLabel, toneFor, withoutTitle } from "./footnotes";
import type { Piece } from "./types";

describe("toneFor", () => {
  test("maps each source to its colour", () => {
    expect(toneFor("conversation")).toBe("c");
    expect(toneFor("branch")).toBe("g");
    expect(toneFor("command")).toBe("g");
    expect(toneFor("given")).toBe("y");
    expect(toneFor("default")).toBe("y");
    expect(toneFor("last-used")).toBe("y");
    expect(toneFor("clipboard")).toBe("y");
  });
});

describe("annotate", () => {
  const pieces: Piece[] = [
    { type: "text", text: "Focus on " },
    { type: "open", name: "focus" },
    { type: "text", text: ". Report " },
    { type: "value", name: "findings", value: "blockers", source: "given" },
    { type: "text", text: ". Diff: " },
    { type: "value", name: "diff", value: "+1 -2", source: "command", detail: "git diff" },
    { type: "text", text: ". Again: " },
    { type: "open", name: "focus" },
  ];

  test("numbers blanks by first appearance and reuses numbers", () => {
    const { shown, notes } = annotate(pieces);
    expect(notes.map((n) => [n.n, n.name])).toEqual([
      [1, "focus"],
      [2, "findings"],
      [3, "diff"],
    ]);
    const numbers = shown.filter((s) => s.type !== "text").map((s) => ("n" in s ? s.n : 0));
    expect(numbers).toEqual([1, 2, 3, 1]);
  });

  test("colours values by source and open blanks by their from", () => {
    const { notes } = annotate(pieces, { focus: { from: "conversation", hint: "the worry" } });
    expect(notes.map((n) => n.tone)).toEqual(["c", "y", "g"]);
    expect(notes[0].open).toBe(true);
    expect(notes[2].detail).toBe("git diff");
    const plain = annotate(pieces, {});
    expect(plain.notes[0].tone).toBe("ask");
  });

  test("labels open blanks with a short hint", () => {
    const { shown } = annotate(pieces, { focus: { hint: "which findings?" } });
    const open = shown.find((s) => s.type === "open");
    expect(open && "text" in open ? open.text : "").toBe("which findings?");
  });
});

describe("shortValue", () => {
  test("keeps short values", () => {
    expect(shortValue("races in the retry backoff")).toBe("races in the retry backoff");
  });
  test("cuts long and many-line values", () => {
    expect(shortValue("diff --git a/x b/x\n+1\n-2")).toBe("diff --git a/x b/x… (3 lines)");
    const long = "a".repeat(100);
    expect(shortValue(long)).toBe(`${"a".repeat(60)}…`);
  });
});

describe("labels", () => {
  test("blankLabel falls back to the name", () => {
    expect(blankLabel("ticket_id")).toBe("ticket id");
    expect(blankLabel("x", { hint: "h".repeat(41) })).toBe("x");
  });
  test("sourceLabel shows the command", () => {
    expect(sourceLabel("command", "git log -1")).toBe("git log -1");
    expect(sourceLabel("conversation")).toBe("From the conversation");
  });
});

describe("withoutTitle", () => {
  test("drops a heading that repeats the title", () => {
    const pieces: Piece[] = [{ type: "text", text: "# Rough idea\n\nSomething." }];
    expect(withoutTitle(pieces, "Rough idea")).toEqual([{ type: "text", text: "Something." }]);
  });
  test("keeps a different heading", () => {
    const pieces: Piece[] = [{ type: "text", text: "# Other\nText" }];
    expect(withoutTitle(pieces, "Title")).toBe(pieces);
  });
});
