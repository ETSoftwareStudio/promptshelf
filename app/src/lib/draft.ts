// A new prompt before its first save. It lives only in the window: the file
// is written once, when you click Save.

import type { Kind } from "./types";

export interface Draft {
  title: string;
  kind: Kind;
  library: string;
  body: string;
  /** The title the draft started with, from a `promptshelf://new` link. */
  start: string;
}

/** The kinds the form offers, in order. Template is the default. */
export const DRAFT_KINDS: { kind: Kind; label: string }[] = [
  { kind: "template", label: "Template" },
  { kind: "snippet", label: "Snippet" },
  { kind: "context", label: "Context" },
  { kind: "draft", label: "Draft (Inbox)" },
];

/** The library in view in the sidebar, else Personal, else the first one. */
export function draftLibrary(
  nav: string,
  libraries: { name: string }[],
  personal: string | null | undefined,
): string {
  const inView = nav.startsWith("lib:") ? nav.slice(4) : null;
  if (inView && libraries.some((l) => l.name === inView)) return inView;
  if (personal && libraries.some((l) => l.name === personal)) return personal;
  return libraries[0]?.name ?? personal ?? "";
}

export function newDraft(
  nav: string,
  libraries: { name: string }[],
  personal: string | null | undefined,
  title = "",
): Draft {
  return {
    title,
    kind: "template",
    library: draftLibrary(nav, libraries, personal),
    body: "",
    start: title,
  };
}

/** True when you typed something that Discard would lose. The kind and the
 *  library are not counted: they are quick to choose again. */
export function typed(d: Draft): boolean {
  return d.title.trim() !== d.start.trim() || d.body.trim() !== "";
}

/** The text of the draft's row in the list. */
export function rowTitle(d: Draft): string {
  return d.title.trim() || "Untitled";
}
