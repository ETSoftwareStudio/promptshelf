// The fill view for text that is not saved yet. Nothing runs: values come from
// what you type in the preview and from defaults. Automatic blanks stay open
// and say where their value will come from.

import { isCommand, type FillSource } from "./filler.svelte";
import { tokens } from "./blanks";
import type { Filled, FillView, Missing, Piece, VarSpec } from "./types";

/** Words for an Automatic blank in the preview. */
export function laterText(from: string): string {
  if (isCommand(from)) return `Runs ${from} when you use the prompt.`;
  switch (from) {
    case "branch":
      return "The git branch, when you use the prompt.";
    case "clipboard":
      return "What you copied, when you use the prompt.";
    case "selection":
      return "The selected text, when you use the prompt.";
    case "last":
      return "The value from the last time you used the prompt.";
    default:
      return "";
  }
}

/** Builds the fill view from the text, the blank settings and typed values. */
export function previewView(
  body: string,
  vars: Record<string, VarSpec>,
  typed: Record<string, string>,
): FillView {
  const pieces: Piece[] = [];
  const values: Filled[] = [];
  const missing: Missing[] = [];
  const seen = new Set<string>();
  let at = 0;
  const text = (t: string) => {
    if (!t) return;
    const last = pieces[pieces.length - 1];
    if (last?.type === "text") last.text += t;
    else pieces.push({ type: "text", text: t });
  };
  for (const t of tokens(body)) {
    text(body.slice(at, t.from));
    at = t.to;
    if (t.kind === "include") {
      text(`[snippet ${t.name}]`);
      continue;
    }
    const spec = vars[t.name] ?? {};
    const given = typed[t.name];
    const fallback = spec.from ? undefined : spec.default;
    const value = given !== undefined ? given : fallback;
    if (value) {
      const source = given !== undefined ? "given" : "default";
      pieces.push({ type: "value", name: t.name, value, source });
      if (!seen.has(t.name)) values.push({ name: t.name, value, source });
    } else {
      pieces.push({ type: "open", name: t.name });
      if (!seen.has(t.name)) missing.push({ name: t.name, ...spec });
    }
    seen.add(t.name);
  }
  text(body.slice(at));
  const plain = pieces.map((p) => (p.type === "text" ? p.text : p.type === "value" ? p.value : `{{${p.name}}}`)).join("");
  return {
    fullId: "",
    pieces,
    text: plain,
    sendText: plain,
    complete: missing.length === 0,
    values,
    missing,
    specs: vars,
  };
}

export class PreviewFill implements FillSource {
  body = $state("");
  vars = $state<Record<string, VarSpec>>({});
  typed = $state<Record<string, string>>({});
  view = $derived(previewView(this.body, this.vars, this.typed));

  current(name: string): string {
    if (name in this.typed) return this.typed[name];
    const spec = this.vars[name];
    return spec?.from ? "" : (spec?.default ?? "");
  }

  editable(): boolean {
    return true;
  }

  reason(): undefined {
    return undefined;
  }

  later(name: string): string | undefined {
    const from = this.vars[name]?.from;
    return from && !(name in this.typed) ? laterText(from) || undefined : undefined;
  }

  set(name: string, value: string): void {
    this.typed = { ...this.typed, [name]: value };
  }
}
