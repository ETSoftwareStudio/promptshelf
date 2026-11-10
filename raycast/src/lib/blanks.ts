// Each blank of a prompt with its number, tone and label, for the tags,
// the footnotes and the notes under the Fill form's fields.

import type { Blank, FillResult, FilledValue, MissingBlank, Source } from "./contract";

/** The colour family of a blank: rose, green, or an outline for "ask". */
export type Tone = "conversation" | "repo" | "ask" | "neutral";

/** The built-in `from:` sources. Anything else is a shell command. */
const NAMED_SOURCES = ["conversation", "branch", "clipboard", "selection", "last"];

export function isCommand(from: string): boolean {
  return !NAMED_SOURCES.includes(from);
}

export interface BlankState {
  /** The footnote number, from 1. */
  n: number;
  name: string;
  spec: Blank;
  value?: FilledValue;
  missing?: MissingBlank;
  tone: Tone;
  label: string;
}

const SOURCE_LABELS: Record<Source, string> = {
  given: "you",
  default: "default",
  "last-used": "last used",
  branch: "repo",
  command: "repo",
  clipboard: "clipboard",
  selection: "selection",
  conversation: "conversation",
};

function toneOfValue(source: Source): Tone {
  if (source === "conversation") return "conversation";
  if (source === "branch" || source === "command") return "repo";
  return "neutral";
}

function toneOfOpen(from: string | undefined): Tone {
  if (from === "conversation") return "conversation";
  if (from === "branch" || (from !== undefined && isCommand(from))) return "repo";
  return "ask";
}

function labelOfOpen(from: string | undefined): string {
  if (from === undefined) return "ask";
  if (from === "conversation") return "conversation";
  if (from === "branch" || isCommand(from)) return "repo";
  return from;
}

/**
 * Joins the blanks of `show` (their order and specs) with a fill result.
 * Blanks that only the fill knows come last.
 */
export function blankStates(blanks: Blank[], fill?: FillResult): BlankState[] {
  const specs = [...blanks];
  for (const m of fill?.missing ?? []) if (!specs.some((b) => b.name === m.name)) specs.push(m);
  for (const v of fill?.values ?? []) if (!specs.some((b) => b.name === v.name)) specs.push({ name: v.name });
  return specs.map((spec, i) => {
    const value = fill?.values.find((v) => v.name === spec.name);
    const missing = fill?.missing.find((m) => m.name === spec.name);
    const from = spec.from ?? missing?.from;
    if (value) {
      return {
        n: i + 1,
        name: spec.name,
        spec,
        value,
        tone: toneOfValue(value.source),
        label: SOURCE_LABELS[value.source] ?? value.source,
      };
    }
    return { n: i + 1, name: spec.name, spec, missing, tone: toneOfOpen(from), label: labelOfOpen(from) };
  });
}

/** The tag text: "1 focus · conversation". */
export function tagText(state: BlankState): string {
  return `${state.n} ${state.name} · ${state.label}`;
}

function tildify(path: string, home: string): string {
  return home && path.startsWith(home) ? `~${path.slice(home.length)}` : path;
}

/** The line under a Fill form field: where the value came from, or how to fill it. */
export function sourceNote(state: BlankState, folder: string, home: string): string {
  const where = tildify(folder, home);
  const from = state.spec.from ?? state.missing?.from;
  const v = state.value;
  if (v) {
    switch (v.source) {
      case "command":
        return `From ${v.detail ?? from ?? "a command"} in ${where}`;
      case "branch":
        return `From the git branch in ${where}`;
      case "conversation":
        return "From the conversation";
      case "clipboard":
        return "From the clipboard";
      case "selection":
        return "From the selection";
      case "last-used":
        return "The value you used last time";
      case "default":
        return choicesNote(state.spec) ?? "The default value";
      case "given":
        return "Your value";
    }
  }
  const parts: string[] = [];
  const choices = choicesNote(state.spec);
  if (choices) parts.push(choices);
  else if (state.spec.hint) parts.push(capitalize(state.spec.hint));
  if (from === "conversation") parts.push("Raycast can not read the conversation");
  if (state.missing?.reason) parts.push(`Not filled: ${state.missing.reason}`);
  return parts.join(" · ") || "Type a value";
}

function choicesNote(spec: Blank): string | undefined {
  return spec.choices?.length ? `You choose · ${spec.choices.join(", ")}` : undefined;
}

function capitalize(text: string): string {
  return text.charAt(0).toUpperCase() + text.slice(1);
}
