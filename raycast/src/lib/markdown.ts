// The filled prompt as Markdown: values in bold with a footnote number, open
// blanks as a bold question. Raycast renders Markdown, so there are no inline
// tints; the tags in the detail carry the source colours.

import type { BlankState } from "./blanks";

export type Segment =
  { kind: "text"; text: string } | { kind: "var"; name: string } | { kind: "include"; name: string };

export function isVarName(s: string): boolean {
  return /^[A-Za-z_][A-Za-z0-9_-]*$/.test(s);
}

/** Splits a template like the core does: text, `{{blank}}` and `{{> include}}`. */
export function parseTemplate(template: string): Segment[] {
  const out: Segment[] = [];
  let text = "";
  let rest = template;
  for (;;) {
    const start = rest.indexOf("{{");
    if (start < 0) break;
    const after = rest.slice(start + 2);
    const end = after.indexOf("}}");
    if (end < 0) break;
    const inner = after.slice(0, end).trim();
    let segment: Segment | undefined;
    if (inner.startsWith(">")) {
      const name = inner.slice(1).trim();
      if (name && !/\s/.test(name)) segment = { kind: "include", name };
    } else if (isVarName(inner)) {
      segment = { kind: "var", name: inner };
    }
    if (segment) {
      text += rest.slice(0, start);
      if (text) out.push({ kind: "text", text });
      text = "";
      out.push(segment);
    } else {
      text += rest.slice(0, start + 2 + end + 2);
    }
    rest = after.slice(end + 2);
  }
  text += rest;
  if (text) out.push({ kind: "text", text });
  return out;
}

const SUPERSCRIPT = ["⁰", "¹", "²", "³", "⁴", "⁵", "⁶", "⁷", "⁸", "⁹"];

export function superscript(n: number): string {
  return String(n)
    .split("")
    .map((d) => SUPERSCRIPT[Number(d)] ?? d)
    .join("");
}

/** Escapes Markdown in a value, so a value can not change the layout. */
export function escapeInline(text: string): string {
  return text.replace(/[\\`*_[\]<>#|~]/g, (c) => `\\${c}`);
}

function bold(text: string, mark: string): string {
  // `** x **` is not bold in Markdown: keep the spaces outside the stars.
  const lead = text.match(/^\s*/)?.[0] ?? "";
  const trail = text.slice(lead.length).match(/\s*$/)?.[0] ?? "";
  const core = text.slice(lead.length, text.length - trail.length);
  return core ? `${lead}**${escapeInline(core)}**${mark}${trail}` : `${text}${mark}`;
}

function codeBlock(text: string): string {
  const longest = Math.max(2, ...(text.match(/`+/g) ?? []).map((r) => r.length));
  const fence = "`".repeat(longest + 1);
  return `\n${fence}\n${text.replace(/\n$/, "")}\n${fence}\n`;
}

function renderValue(value: string, n: number): string {
  if (value.includes("\n")) return `${superscript(n)}${codeBlock(value)}`;
  return bold(value, superscript(n));
}

function renderOpen(state: BlankState): string {
  const ask = state.spec.hint ?? state.missing?.hint ?? state.name;
  return `**${escapeInline(ask)}?**${superscript(state.n)}`;
}

/**
 * Renders the filled text. It walks the template and matches each piece in the
 * filled text: text stays as it is, values and open blanks get their marks, and
 * an include is whatever lies before the next piece of text. If the two do not
 * line up, it returns the filled text without marks.
 */
export function renderFilled(body: string, text: string, states: BlankState[]): string {
  const marked = align(parseTemplate(body), text, states);
  return marked ?? text;
}

function align(segments: Segment[], text: string, states: BlankState[]): string | undefined {
  let out = "";
  let at = 0;
  for (let i = 0; i < segments.length; i++) {
    const seg = segments[i]!;
    if (seg.kind === "text") {
      if (!text.startsWith(seg.text, at)) return undefined;
      out += seg.text;
      at += seg.text.length;
    } else if (seg.kind === "var") {
      const state = states.find((s) => s.name === seg.name);
      const open = `{{${seg.name}}}`;
      if (state?.value && text.startsWith(state.value.value, at)) {
        out += renderValue(state.value.value, state.n);
        at += state.value.value.length;
      } else if (state && !state.value && text.startsWith(open, at)) {
        out += renderOpen(state);
        at += open.length;
      } else {
        return undefined;
      }
    } else {
      const next = segments[i + 1];
      let end: number;
      if (next === undefined) end = text.length;
      else if (next.kind === "text") end = text.indexOf(next.text, at);
      else return undefined;
      if (end < 0) return undefined;
      out += text.slice(at, end);
      at = end;
    }
  }
  return at === text.length ? out : undefined;
}
