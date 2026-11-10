// The state of one prompt being filled: what the resolvers found, what you
// typed, and the sentence that shows now. The library window and the quick
// panel share it.

import { api, errorText } from "./api";
import { RESOLVED } from "./footnotes";
import type { FillView, Value } from "./types";

export class Filler {
  id = $state<string | null>(null);
  /** The resolver run. Null until it finishes. */
  base = $state<FillView | null>(null);
  /** What shows now: the resolver run with your values on top. */
  view = $state<FillView | null>(null);
  /** Values you typed or picked, by blank name. Empty means "leave it open". */
  typed = $state<Record<string, string>>({});
  resolving = $state(false);
  error = $state<string | null>(null);
  /** True when the resolvers ran with shell commands held back, as they do
   *  for a prompt that a link opened. Run Commands runs them. */
  held = $state(false);

  #load = 0;
  #render = 0;
  #timer: ReturnType<typeof setTimeout> | undefined;

  /** Shows the prompt with every blank open at once, then runs the
   *  resolvers and shows what they found. They run shell commands only
   *  with `run`. */
  async load(id: string, run = true): Promise<void> {
    const seq = ++this.#load;
    clearTimeout(this.#timer);
    this.id = id;
    this.typed = {};
    this.base = null;
    this.error = null;
    this.held = !run;
    this.resolving = true;
    try {
      const quick = await api.render(id, {});
      if (seq !== this.#load) return;
      this.view = quick;
      const base = await api.fill(id, {}, run);
      if (seq !== this.#load) return;
      this.base = base;
      if (Object.keys(this.typed).length) await this.rerender();
      else this.view = base;
    } catch (e) {
      if (seq === this.#load) this.error = errorText(e);
    } finally {
      if (seq === this.#load) this.resolving = false;
    }
  }

  /** Runs the resolvers again with shell commands, and keeps what you typed. */
  async runCommands(): Promise<void> {
    const id = this.id;
    if (!id) return;
    const seq = ++this.#load;
    this.held = false;
    this.resolving = true;
    try {
      const base = await api.fill(id, {}, true);
      if (seq !== this.#load) return;
      this.base = base;
      this.error = null;
      if (Object.keys(this.typed).length) await this.rerender();
      else this.view = base;
    } catch (e) {
      if (seq === this.#load) this.error = errorText(e);
    } finally {
      if (seq === this.#load) this.resolving = false;
    }
  }

  /** The open blanks that wait for Run Commands. */
  waiting(): string[] {
    if (!this.held || !this.base) return [];
    return this.base.missing
      .filter((m) => m.from && isCommand(m.from) && !(m.name in this.typed))
      .map((m) => m.name);
  }

  clear(): void {
    this.#load++;
    clearTimeout(this.#timer);
    this.id = null;
    this.base = null;
    this.view = null;
    this.typed = {};
    this.error = null;
    this.resolving = false;
    this.held = false;
  }

  /** Every value to send: what the resolvers found, then what you typed. */
  values(): Record<string, Value> {
    return mergeValues(this.base?.values ?? [], this.typed);
  }

  /** Sets a blank's value and renders again shortly after. */
  set(name: string, value: string): void {
    this.typed = { ...this.typed, [name]: value };
    clearTimeout(this.#timer);
    this.#timer = setTimeout(() => void this.rerender(), 60);
  }

  async rerender(): Promise<void> {
    const id = this.id;
    if (!id) return;
    const seq = ++this.#render;
    try {
      const view = await api.render(id, this.values());
      if (seq === this.#render && id === this.id) {
        this.view = view;
        this.error = null;
      }
    } catch (e) {
      if (seq === this.#render) this.error = errorText(e);
    }
  }

  /** True when the blank shows a field: you typed it, or no resolver set it. */
  editable(name: string): boolean {
    if (name in this.typed) return true;
    const found = this.base?.values.find((v) => v.name === name);
    return !found || !RESOLVED.includes(found.source);
  }

  /** The text in a blank's field. */
  current(name: string): string {
    if (name in this.typed) return this.typed[name];
    return this.base?.values.find((v) => v.name === name)?.value ?? "";
  }

  /** Why a resolver could not fill a blank, if it tried. */
  reason(name: string): string | undefined {
    if (this.waiting().includes(name)) return "Waits for Run Commands.";
    return this.base?.missing.find((m) => m.name === name)?.reason ?? undefined;
  }
}

/** Sources that are not shell commands, as in `promptshelf-core`. */
const NAMED_SOURCES = ["conversation", "branch", "clipboard", "selection", "last"];

/** True when a blank's `from` is a shell command. */
export function isCommand(from: string): boolean {
  return !NAMED_SOURCES.includes(from);
}

/** Resolved values with typed ones on top. An empty typed value removes the
 *  value, so the blank stays open. */
export function mergeValues(
  resolved: { name: string; value: string; source: Value["source"]; detail?: string }[],
  typed: Record<string, string>,
): Record<string, Value> {
  const out: Record<string, Value> = {};
  for (const f of resolved) {
    out[f.name] = f.detail
      ? { value: f.value, source: f.source, detail: f.detail }
      : { value: f.value, source: f.source };
  }
  for (const [name, value] of Object.entries(typed)) {
    if (value === "") delete out[name];
    else out[name] = { value, source: "given" };
  }
  return out;
}
