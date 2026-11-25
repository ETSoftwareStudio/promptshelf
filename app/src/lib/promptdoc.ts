// A prompt file as the rich editor sees it: a title, tags, blank settings and
// the text. Save writes back only the fields you changed, so comments and
// fields the app does not know stay as they are.

import { Document, type YAMLMap, type YAMLSeq, parseDocument } from "yaml";
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

/** What the rich editor edits. */
export interface Fields {
  title: string;
  tags: string[];
  vars: Record<string, VarSpec>;
  body: string;
}

interface Split {
  /** The YAML between the `---` lines, or null when there is none. */
  yaml: string | null;
  /** "\n" when a blank line sits between the frontmatter and the text. */
  gap: string;
  body: string;
}

/** Splits the file the same way as the core's `split_frontmatter`. */
export function split(text: string): Split {
  const t = text.replace(/\r\n/g, "\n");
  if (!t.startsWith("---\n")) return { yaml: null, gap: "", body: t };
  const rest = t.slice(4);
  if (rest.startsWith("---\n")) return { yaml: "", ...gapped(rest.slice(4)) };
  let offset = 0;
  for (const line of rest.split(/(?<=\n)/)) {
    if (line.trimEnd() === "---") {
      return { yaml: rest.slice(0, offset), ...gapped(rest.slice(offset + line.length)) };
    }
    offset += line.length;
  }
  return { yaml: null, gap: "", body: t };
}

function gapped(body: string): { gap: string; body: string } {
  return body.startsWith("\n") ? { gap: "\n", body: body.slice(1) } : { gap: "", body };
}

function asStrings(v: unknown): string[] {
  return Array.isArray(v) ? v.filter((x) => x != null).map(String) : [];
}

function asSpec(v: unknown): VarSpec {
  if (!v || typeof v !== "object") return {};
  const o = v as Record<string, unknown>;
  const s = (k: string) => (o[k] == null ? undefined : String(o[k]));
  const spec: VarSpec = { from: s("from"), hint: s("hint"), default: s("default"), pattern: s("pattern") };
  if (Array.isArray(o.choices)) spec.choices = asStrings(o.choices);
  return cleanSpec(spec) ?? {};
}

/** Reads the fields out of a prompt file. Throws when the YAML is broken. */
export function readFields(text: string): Fields {
  const { yaml, body } = split(text);
  const data = yaml ? parseYaml(yaml) : {};
  const vars: Record<string, VarSpec> = {};
  const raw = data.vars;
  if (raw && typeof raw === "object" && !Array.isArray(raw)) {
    for (const [name, spec] of Object.entries(raw)) vars[name] = asSpec(spec);
  }
  return {
    title: data.title == null ? "" : String(data.title),
    tags: asStrings(data.tags),
    vars,
    body,
  };
}

function parseYaml(yaml: string): Record<string, unknown> {
  const doc = parseDocument(yaml);
  if (doc.errors.length) throw new Error(doc.errors[0].message.split("\n")[0]);
  const js: unknown = doc.toJS();
  return js && typeof js === "object" && !Array.isArray(js) ? (js as Record<string, unknown>) : {};
}

/** The settings to write: blanks in the text first, in text order, then
 *  blanks that are not in the text but still have settings. Empty specs are
 *  left out. */
export function varsToWrite(body: string, vars: Record<string, VarSpec>): Record<string, VarSpec> {
  const out: Record<string, VarSpec> = {};
  const used = blankNames(body);
  for (const name of [...used, ...Object.keys(vars).filter((n) => !used.includes(n))]) {
    const spec = cleanSpec(vars[name]);
    if (spec) out[name] = spec;
  }
  return out;
}

/** Block style for the map, `[a, b]` style for each list of choices. */
function varsNode(doc: Document, vars: Record<string, VarSpec>): YAMLMap {
  const node = doc.createNode(vars) as YAMLMap;
  for (const [name, spec] of Object.entries(vars)) {
    if (!spec.choices) continue;
    const choices = doc.createNode(spec.choices) as YAMLSeq;
    choices.flow = true;
    (node.get(name, true) as unknown as YAMLMap).set("choices", choices);
  }
  return node;
}

const same = (a: unknown, b: unknown) => JSON.stringify(a) === JSON.stringify(b);

/** Writes the fields into the file. A field you did not change keeps its
 *  text on disk; other frontmatter fields are kept too. */
export function writeFields(original: string, f: Fields): string {
  const { yaml, gap, body } = split(original);
  const before = yaml === null ? null : readFields(original);
  const doc = yaml ? parseDocument(yaml) : new Document({});
  let changed = false;

  const title = f.title.trim();
  if (!before || title !== before.title.trim()) {
    if (title) doc.set("title", title);
    else doc.delete("title");
    changed = true;
  }
  const tags = [...new Set(f.tags.map((t) => t.trim()).filter(Boolean))];
  if (!before || !same(tags, before.tags)) {
    if (tags.length) doc.set("tags", doc.createNode(tags, { flow: true }));
    else doc.delete("tags");
    changed = true;
  }
  const vars = varsToWrite(f.body, f.vars);
  if (!before || !same(vars, varsToWrite(before.body, before.vars))) {
    if (Object.keys(vars).length) doc.set("vars", varsNode(doc, vars));
    else doc.delete("vars");
    changed = true;
  }

  const head = changed ? doc.toString({ lineWidth: 0 }) : (yaml ?? "");
  if (yaml === null && head.trim() === "{}") return f.body;
  if (yaml === null) return `---\n${head}---\n\n${f.body}`;
  if (!changed && f.body === body) return original;
  const empty = head.trim() === "{}" || head.trim() === "";
  return `---\n${empty ? "" : head}---\n${gap}${f.body}`;
}
