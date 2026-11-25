// The prompt text in CodeMirror. Each `{{name}}` shows as a chip and each
// `{{> id}}` as a snippet chip; the file text stays exactly what you typed.
// Typing `{{` opens a menu of blanks and snippets.

import {
  autocompletion,
  type Completion,
  type CompletionContext,
  type CompletionResult,
  completionKeymap,
} from "@codemirror/autocomplete";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { EditorState, StateEffect, StateField } from "@codemirror/state";
import {
  Decoration,
  type DecorationSet,
  EditorView,
  keymap,
  placeholder,
  WidgetType,
} from "@codemirror/view";
import { blankNames, blankType, isVarName, tokens } from "./promptdoc";
import type { VarSpec } from "./types";

/** The colour class of a blank: c (rose) for the conversation, g (green) for
 *  the branch or a command, y (blue) for everything else. */
export function blankTone(spec: VarSpec | undefined): "c" | "g" | "y" {
  const from = spec?.from?.trim();
  if (!from) return "y";
  if (from === "conversation") return "c";
  if (["clipboard", "selection", "last"].includes(from)) return "y";
  return "g";
}

/** Where an Automatic blank gets its value, as a key for its icon. */
export function sourceKey(from: string): string {
  const f = from.trim();
  return ["conversation", "clipboard", "selection", "branch", "last"].includes(f) ? f : "command";
}

const SVG = (d: string, w = 12) =>
  `<svg width="${w}" height="${w}" viewBox="0 0 16 16" aria-hidden="true"><g fill="none" stroke="currentColor" stroke-width="1.5">${d}</g></svg>`;

/** Small icons for chips and the blank card, 16 × 16 grid. */
export const ICONS: Record<string, string> = {
  text: SVG('<path d="M3 13l1-3.5 7-7 2.5 2.5-7 7zM9.5 4l2.5 2.5"/>'),
  choice: SVG('<circle cx="3.5" cy="4.5" r="1.2"/><circle cx="3.5" cy="11.5" r="1.2"/><path d="M7 4.5h6.5M7 11.5h6.5"/>'),
  auto: SVG('<path d="M9 1.5L3.5 9H8l-1 5.5L12.5 7H8z"/>'),
  conversation: SVG('<path d="M2.5 3.5h11v7h-6l-3 2.5v-2.5h-2z"/>'),
  clipboard: SVG('<path d="M5.5 3H3.5v11h9V3h-2M5.5 2h5v2.5h-5z"/>'),
  selection: SVG('<path d="M6 2.5h4M6 13.5h4M8 2.5v11"/>'),
  branch: SVG('<circle cx="4.5" cy="3.5" r="1.5"/><circle cx="4.5" cy="12.5" r="1.5"/><circle cx="11.5" cy="5" r="1.5"/><path d="M4.5 5v6M11.5 6.5c0 3-7 2-7 4.5"/>'),
  last: SVG('<path d="M3 8a5 5 0 105-5H5M5 1l-2 2 2 2"/>'),
  command: SVG('<path d="M3 4.5l3.5 3.5L3 11.5M8 12h5"/>'),
  snippet: SVG('<path d="M4 2.5v6.5h8M9.5 6.5L12 9l-2.5 2.5"/>'),
  caret: '<svg width="9" height="9" viewBox="0 0 10 10" aria-hidden="true"><path fill="currentColor" d="M1.5 3.5h7L5 7.5z"/></svg>',
};

/** The icon of a blank: its source when it is Automatic, else its type. */
export function blankIcon(spec: VarSpec | undefined): string {
  const type = blankType(spec);
  return type === "auto" ? ICONS[sourceKey(spec!.from!)] : ICONS[type];
}

const esc = (s: string) =>
  s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]!);

class Chip extends WidgetType {
  constructor(
    readonly kind: "blank" | "include",
    readonly name: string,
    readonly spec: VarSpec | undefined,
    readonly active: boolean,
  ) {
    super();
  }

  eq(o: Chip): boolean {
    return (
      o.kind === this.kind &&
      o.name === this.name &&
      o.active === this.active &&
      JSON.stringify(o.spec) === JSON.stringify(this.spec)
    );
  }

  toDOM(): HTMLElement {
    const el = document.createElement("span");
    if (this.kind === "include") {
      el.className = "blank inc";
      el.dataset.include = this.name;
      el.title = `Snippet: the text of ${this.name} goes here`;
      el.innerHTML = `${ICONS.snippet}<span>Snippet <b>${esc(this.name)}</b></span>`;
      return el;
    }
    el.className = `blank ${blankTone(this.spec)}${this.active ? " on" : ""}`;
    el.dataset.blank = this.name;
    el.title = "Click to set what fills this blank";
    let html = `${blankIcon(this.spec)}<span>${esc(this.name)}</span>`;
    if (blankType(this.spec) === "choice") {
      if (this.spec?.default) html += `<span class="dv">${esc(this.spec.default)}</span>`;
      html += ICONS.caret;
    }
    el.innerHTML = html;
    return el;
  }

  ignoreEvent(): boolean {
    return false;
  }
}

const setVars = StateEffect.define<Record<string, VarSpec>>();
const setActive = StateEffect.define<string | null>();

const varsField = StateField.define<Record<string, VarSpec>>({
  create: () => ({}),
  update(value, tr) {
    for (const e of tr.effects) if (e.is(setVars)) return e.value;
    return value;
  },
});

const activeField = StateField.define<string | null>({
  create: () => null,
  update(value, tr) {
    for (const e of tr.effects) if (e.is(setActive)) return e.value;
    return value;
  },
});

function chipsFor(state: EditorState): DecorationSet {
  const vars = state.field(varsField);
  const active = state.field(activeField);
  const ranges = tokens(state.doc.toString()).map((t) =>
    Decoration.replace({
      widget: new Chip(t.kind, t.name, t.kind === "blank" ? vars[t.name] : undefined, t.name === active),
    }).range(t.from, t.to),
  );
  return Decoration.set(ranges);
}

const chips = StateField.define<DecorationSet>({
  create: chipsFor,
  update(deco, tr) {
    const redraw = tr.docChanged || tr.effects.some((e) => e.is(setVars) || e.is(setActive));
    return redraw ? chipsFor(tr.state) : deco;
  },
  provide: (f) => [EditorView.decorations.from(f), EditorView.atomicRanges.of((v) => v.state.field(f))],
});

/** Words for a blank in the menu. */
export function describe(spec: VarSpec | undefined): string {
  const type = blankType(spec);
  if (type === "auto") return `from ${spec!.from}`;
  if (type === "choice") return spec!.choices!.join(" · ");
  return spec?.hint ?? "Text";
}

export interface BlankEditorOptions {
  doc: string;
  vars: Record<string, VarSpec>;
  /** Snippet IDs for the menu. */
  snippets: () => string[];
  placeholder?: string;
  onchange: (doc: string) => void;
  /** A click on a blank chip. */
  onblank: (name: string, chip: HTMLElement) => void;
  /** A new blank from the menu. */
  onnew?: (name: string) => void;
}

export interface BlankEditor {
  view: EditorView;
  setVars(vars: Record<string, VarSpec>): void;
  setActive(name: string | null): void;
  /** Puts `{{name}}` at the cursor, with a space before it when needed. */
  insert(text: string): void;
  chip(name: string): HTMLElement | null;
  destroy(): void;
}

function completions(opts: BlankEditorOptions, onnew: (name: string) => void) {
  return (ctx: CompletionContext): CompletionResult | null => {
    const m = ctx.matchBefore(/\{\{\s*(>\s*)?[\w-]*$/);
    if (!m) return null;
    const include = /^\{\{\s*>/.test(m.text);
    const typed = m.text.replace(/^\{\{\s*>?\s*/, "");
    const vars = ctx.state.field(varsField);
    const names = [...new Set([...blankNames(ctx.state.doc.toString()), ...Object.keys(vars)])];
    const put =
      (text: string, after?: () => void) =>
      (view: EditorView, _c: Completion, _from: number, to: number) => {
        const end = view.state.sliceDoc(to, to + 2) === "}}" ? to + 2 : to;
        view.dispatch({
          changes: { from: m.from, to: end, insert: text },
          selection: { anchor: m.from + text.length },
        });
        after?.();
      };
    const options: Completion[] = [];
    if (!include) {
      if (typed && isVarName(typed) && !names.includes(typed)) {
        options.push({
          label: typed,
          detail: "New blank",
          section: { name: "Blanks", rank: 0 },
          boost: 99,
          apply: put(`{{${typed}}}`, () => onnew(typed)),
        });
      }
      for (const n of names) {
        options.push({
          label: n,
          detail: describe(vars[n]),
          section: { name: "Blanks", rank: 0 },
          apply: put(`{{${n}}}`),
        });
      }
    }
    for (const id of opts.snippets()) {
      options.push({
        label: id,
        section: { name: "Snippets", rank: 1 },
        apply: put(`{{> ${id}}}`),
      });
    }
    return { from: m.to - typed.length, options, validFor: /^[\w-]*$/ };
  };
}

export function createBlankEditor(parent: HTMLElement, opts: BlankEditorOptions): BlankEditor {
  const onnew = (name: string) => requestAnimationFrame(() => opts.onnew?.(name));
  const view = new EditorView({
    parent,
    state: EditorState.create({
      doc: opts.doc,
      extensions: [
        varsField.init(() => opts.vars),
        activeField,
        chips,
        history(),
        EditorView.lineWrapping,
        EditorView.contentAttributes.of({ "aria-label": "Prompt text", spellcheck: "true" }),
        placeholder(opts.placeholder ?? ""),
        autocompletion({
          override: [completions(opts, onnew)],
          icons: false,
          activateOnTyping: true,
        }),
        keymap.of([...completionKeymap, ...defaultKeymap, ...historyKeymap]),
        EditorView.updateListener.of((u) => {
          if (u.docChanged) opts.onchange(u.state.doc.toString());
        }),
        EditorView.domEventHandlers({
          mousedown(e) {
            const el = (e.target as Element).closest<HTMLElement>("[data-blank]");
            if (!el) return false;
            e.preventDefault();
            opts.onblank(el.dataset.blank!, el);
            return true;
          },
        }),
      ],
    }),
  });

  return {
    view,
    setVars: (vars) => view.dispatch({ effects: setVars.of(vars) }),
    setActive: (name) => view.dispatch({ effects: setActive.of(name) }),
    insert(text) {
      const { from, to } = view.state.selection.main;
      const before = view.state.sliceDoc(Math.max(0, from - 1), from);
      const ins = before && !/\s/.test(before) ? ` ${text}` : text;
      view.dispatch({
        changes: { from, to, insert: ins },
        selection: { anchor: from + ins.length },
        scrollIntoView: true,
      });
      view.focus();
    },
    chip: (name) => view.dom.querySelector<HTMLElement>(`[data-blank="${CSS.escape(name)}"]`),
    destroy: () => view.destroy(),
  };
}
