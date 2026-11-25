// Blanks and includes in a prompt's text, found the way the core finds them.
// No YAML here: the fill view loads this module too.

import type { VarSpec } from "./types";

/** A `{{name}}` blank or a `{{> id}}` include in the text. */
export interface Token {
  from: number;
  to: number;
  kind: "blank" | "include";
  name: string;
}

/** The same rule as the core: a letter or `_`, then letters, digits, `_` or `-`. */
export function isVarName(s: string): boolean {
  return /^[A-Za-z_][A-Za-z0-9_-]*$/.test(s);
}

/** Finds the blanks and includes the way the core does. Other `{{ }}` text is
 *  plain text. */
export function tokens(body: string): Token[] {
  const out: Token[] = [];
  let i = 0;
  for (;;) {
    const start = body.indexOf("{{", i);
    if (start < 0) break;
    const end = body.indexOf("}}", start + 2);
    if (end < 0) break;
    const inner = body.slice(start + 2, end).trim();
    if (inner.startsWith(">")) {
      const name = inner.slice(1).trim();
      if (name && !/\s/.test(name)) out.push({ from: start, to: end + 2, kind: "include", name });
    } else if (isVarName(inner)) {
      out.push({ from: start, to: end + 2, kind: "blank", name: inner });
    }
    i = end + 2;
  }
  return out;
}

/** The blank names in the text, in the order they first appear. */
export function blankNames(body: string): string[] {
  return [...new Set(tokens(body).filter((t) => t.kind === "blank").map((t) => t.name))];
}

/** Renames every `{{old}}` in the text. Includes stay as they are. */
export function renameBlank(body: string, from: string, to: string): string {
  let out = "";
  let at = 0;
  for (const t of tokens(body)) {
    if (t.kind !== "blank" || t.name !== from) continue;
    out += `${body.slice(at, t.from)}{{${to}}}`;
    at = t.to;
  }
  return out + body.slice(at);
}

/** This many choices or more show as a menu in the fill view, not buttons. */
export const MENU_AT = 5;

export type BlankType = "text" | "choice" | "auto";

/** Automatic when a source fills it, Choice when it has choices, else Text. */
export function blankType(spec: VarSpec | undefined): BlankType {
  if (spec?.from) return "auto";
  if (spec?.choices?.length) return "choice";
  return "text";
}

/** The spec without empty fields. Null when nothing is left. */
export function cleanSpec(spec: VarSpec | undefined): VarSpec | null {
  if (!spec) return null;
  const out: VarSpec = {};
  const from = spec.from?.trim();
  if (from) out.from = from;
  if (spec.hint?.trim()) out.hint = spec.hint.trim();
  if (spec.default?.trim()) out.default = spec.default.trim();
  const choices = (spec.choices ?? []).map((c) => c.trim()).filter(Boolean);
  if (choices.length) out.choices = [...new Set(choices)];
  if (spec.pattern?.trim()) out.pattern = spec.pattern.trim();
  return Object.keys(out).length ? out : null;
}

/** Takes every `{{name}}` out of the text, and the space before it when that
 *  would leave two spaces or a space before punctuation. */
export function removeBlank(body: string, name: string): string {
  let out = "";
  let at = 0;
  for (const t of tokens(body)) {
    if (t.kind !== "blank" || t.name !== name) continue;
    let head = body.slice(at, t.from);
    const next = body.charAt(t.to);
    if (head.endsWith(" ") && (next === "" || /[\s.,;:!?)]/.test(next))) head = head.slice(0, -1);
    out += head;
    at = t.to;
  }
  return out + body.slice(at);
}

/** The first free name: `blank`, then `blank-2`, `blank-3` and so on. */
export function freeName(taken: Iterable<string>, base = "blank"): string {
  const names = new Set(taken);
  if (!names.has(base)) return base;
  for (let n = 2; ; n++) if (!names.has(`${base}-${n}`)) return `${base}-${n}`;
}
