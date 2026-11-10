// What the sidebar selects, and what a sync did, in words.

import { KINDS, libraryLabel } from "./format";
import type { Kind, LibraryStatus, Query, SyncReport } from "./types";

/** The sidebar selection: "all", "recent", "kind:<kind>" or "lib:<name>". */
export type Nav = string;

export function navTitle(nav: Nav): string {
  if (nav === "recent") return "Recent";
  if (nav === "inbox" || nav === "kind:draft") return "Inbox";
  if (nav === "proposals") return "Your proposals";
  if (nav === "problems") return "Files to fix";
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
        : `${lib}: ${problem.hint?.message ?? problem.error}`,
    };
  }
  const pulled = reports.reduce((n, r) => n + r.pulled, 0);
  const pushed = reports.reduce((n, r) => n + r.pushed, 0);
  if (!pulled && !pushed) return { bad: false, message: null };
  return { bad: false, message: `Pulled ${pulled}, pushed ${pushed}.` };
}

/** A library's sync state in a few words, for Settings → Libraries. */
export function syncState(
  s: LibraryStatus | undefined,
  info: { git: boolean; exists: boolean },
): { text: string; bad: boolean } {
  if (!info.exists) return { text: "Folder missing", bad: true };
  if (!info.git) return { text: "Not a git repo: this computer only", bad: false };
  if (!s?.remote) return { text: "No remote: this computer only", bad: false };
  const last = s.lastSync;
  if (last?.conflicts.length) return { text: "Edited here and on the remote", bad: true };
  if (last?.error) return { text: "The last sync failed", bad: true };
  const parts: string[] = [];
  if (s.ahead) parts.push(`${s.ahead} to push`);
  if (s.behind) parts.push(`${s.behind} to pull`);
  if (s.changes) parts.push(s.changes === 1 ? "1 change" : `${s.changes} changes`);
  if (last) parts.unshift(`Synced at ${clock(last.at)}`);
  return { text: parts.join(" · ") || "Not synced yet", bad: false };
}

/** Seconds since 1970 as a local time: "9:41". */
export function clock(at: number): string {
  return new Date(at * 1000).toLocaleTimeString([], { hour: "numeric", minute: "2-digit" });
}
