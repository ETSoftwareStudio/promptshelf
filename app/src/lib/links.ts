// What the library window does with a `promptshelf://` link. The Tauri side
// parses and checks the link; this decides the steps. A link only moves the
// window: it never writes, and the prompt it opens holds its commands back.

import type { Link } from "./types";

export type LinkPlan =
  | { kind: "none" }
  | { kind: "toast"; text: string }
  | { kind: "prompt"; fullId: string; send: boolean }
  | { kind: "search"; query: string }
  | { kind: "new"; title: string };

/** The toast for a link the app does not follow. */
export function cantOpen(reason: string): string {
  return `PromptShelf can't open this link: ${reason}`;
}

export function linkPlan(
  link: Link,
  now: {
    /** False at first run, before the Personal library has a folder. */
    configured: boolean;
    /** True while a sheet or the editor is open: a link must not replace it. */
    busy: boolean;
  },
): LinkPlan {
  if (link.route === "bad") return { kind: "toast", text: cantOpen(link.reason) };
  if (link.route === "home") return { kind: "none" };
  if (!now.configured)
    return { kind: "toast", text: cantOpen("choose a folder for your Personal library first") };
  if (now.busy) return { kind: "toast", text: cantOpen("close the open sheet or edit first") };
  switch (link.route) {
    case "open":
    case "fill":
      return {
        kind: "prompt",
        fullId: `${link.library}/${link.id}`,
        send: link.route === "fill",
      };
    case "search":
      return { kind: "search", query: link.query };
    case "new":
      return { kind: "new", title: link.title ?? "" };
  }
}
