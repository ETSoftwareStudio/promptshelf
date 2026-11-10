// The prompt as a sentence with numbered footnotes. Every surface uses the
// same colours: rose for a value from the conversation, green for a value
// from the repo or a command, a quiet blue for a value you gave or a default,
// and a dashed outline for a blank nobody filled yet.

import type { Piece, Source, VarSpec } from "./types";

/** The colour class of a value: c (rose), g (green) or y (you). */
export type Tone = "c" | "g" | "y";

export function toneFor(source: Source): Tone {
  switch (source) {
    case "conversation":
      return "c";
    case "branch":
    case "command":
      return "g";
    default:
      return "y";
  }
}

/** Sources the app works out for you; you can still change the value. */
export const RESOLVED: readonly Source[] = ["branch", "command", "clipboard", "selection"];

/** Words for a footnote. Commands show the command itself. */
export function sourceLabel(source: Source, detail?: string): string {
  switch (source) {
    case "conversation":
      return "From the conversation";
    case "branch":
      return "The git branch";
    case "command":
      return detail ?? "A command";
    case "clipboard":
      return "From the clipboard";
    case "selection":
      return "From the selection";
    case "last-used":
      return "Last time's value";
    case "default":
      return "The default";
    case "given":
      return "You gave it";
  }
}

export type Shown =
  | { type: "text"; text: string }
  | {
      type: "value";
      name: string;
      n: number;
      tone: Tone;
      text: string;
      source: Source;
      detail?: string;
    }
  | { type: "open"; name: string; n: number; tone: Tone | "ask"; text: string };

export interface Note {
  n: number;
  name: string;
  /** The colour of the number chip. */
  tone: Tone | "ask";
  open: boolean;
  source?: Source;
  detail?: string;
  value?: string;
}

/** A value short enough for the sentence. Long or many-line values, such as
 *  a diff, show their first line and how many lines they have. */
export function shortValue(value: string, max = 80): string {
  const lines = value.split("\n");
  const first = lines[0].trim();
  const more = lines.length > 1 ? ` (${lines.length} lines)` : "";
  if (lines.length === 1 && first.length <= max) return first;
  const cut = first.length > 60 ? `${first.slice(0, 60).trimEnd()}…` : `${first}…`;
  return cut + more;
}

/** The label of an open blank: its hint when it is short, else its name. */
export function blankLabel(name: string, spec?: VarSpec): string {
  const hint = spec?.hint?.trim();
  if (hint && hint.length <= 40) return hint;
  return name.replace(/[-_]+/g, " ");
}

/** Numbers each blank by where it first appears; a blank used twice keeps its
 *  number. Returns the pieces to show and one note per number. */
export function annotate(
  pieces: Piece[],
  specs: Record<string, VarSpec> = {},
): { shown: Shown[]; notes: Note[] } {
  const numbers = new Map<string, number>();
  const notes: Note[] = [];
  const shown: Shown[] = [];
  for (const p of pieces) {
    if (p.type === "text") {
      shown.push({ type: "text", text: p.text });
      continue;
    }
    let n = numbers.get(p.name);
    const isNew = n === undefined;
    if (n === undefined) {
      n = numbers.size + 1;
      numbers.set(p.name, n);
    }
    if (p.type === "value") {
      const tone = toneFor(p.source);
      shown.push({
        type: "value",
        name: p.name,
        n,
        tone,
        text: shortValue(p.value),
        source: p.source,
        detail: p.detail,
      });
      if (isNew) {
        notes.push({
          n,
          name: p.name,
          tone,
          open: false,
          source: p.source,
          detail: p.detail,
          value: p.value,
        });
      }
    } else {
      const spec = specs[p.name];
      // A blank the model fills from the chat keeps the conversation colour.
      const tone = spec?.from === "conversation" ? "c" : "ask";
      shown.push({ type: "open", name: p.name, n, tone, text: blankLabel(p.name, spec) });
      if (isNew) notes.push({ n, name: p.name, tone, open: true });
    }
  }
  return { shown, notes };
}

/** Drops a first-line `# Heading` that repeats the title; the window shows the
 *  title already. */
export function withoutTitle(pieces: Piece[], title: string): Piece[] {
  const first = pieces[0];
  if (!first || first.type !== "text") return pieces;
  const m = /^\s*#\s+(.+?)\s*\n+/.exec(first.text);
  if (!m || m[1].trim().toLowerCase() !== title.trim().toLowerCase()) return pieces;
  const rest = first.text.slice(m[0].length);
  return rest ? [{ type: "text", text: rest }, ...pieces.slice(1)] : pieces.slice(1);
}
