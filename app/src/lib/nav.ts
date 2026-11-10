// What the sidebar selects, and what a sync did, in words.

import { KINDS, libraryLabel } from "./format";
import type { Kind, Query, SyncReport } from "./types";

/** The sidebar selection: "all", "recent", "kind:<kind>" or "lib:<name>". */
export type Nav = string;

export function navTitle(nav: Nav): string {
  if (nav === "recent") return "Recent";
  if (nav === "kind:draft") return "Inbox";
  if (nav.startsWith("kind:"))
    return KINDS.find((k) => `kind:${k.kind}` === nav)?.label ?? "Prompts";
  if (nav.startsWith("lib:")) return libraryLabel(nav.slice(4));
  return "All Prompts";
}

export function navQuery(nav: Nav, text: string): Query {
  return {
    text,
    view: nav === "recent" ? "recent" : "all",
    kind: nav.startsWith("kind:") ? (nav.slice(5) as Kind) : null,
    library: nav.startsWith("lib:") ? nav.slice(4) : null,
  };
}

/** A sync in words: a message when something needs a look or moved, and
 *  whether it is a problem. */
export function syncSummary(reports: SyncReport[]): { message: string | null; bad: boolean } {
  const problem = reports.find((r) => r.error || r.conflicts?.length);
  if (problem) {
    const lib = libraryLabel(problem.library);
    return {
      bad: true,
      message: problem.conflicts?.length
        ? `${lib}: ${problem.conflicts.join(", ")} changed here and on the remote. Your version stays.`
        : `${lib}: ${problem.error}`,
    };
  }
  const pulled = reports.reduce((n, r) => n + r.pulled, 0);
  const pushed = reports.reduce((n, r) => n + r.pushed, 0);
  if (!pulled && !pushed) return { bad: false, message: null };
  return { bad: false, message: `Pulled ${pulled}, pushed ${pushed}.` };
}
