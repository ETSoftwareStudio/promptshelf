// The rows of the Inbox, proposals and files-to-fix lists.

import { libraryLabel, proposalLabel, shortDay } from "./format";
import type { Diagnostic, Group, InboxView, ProposalView } from "./types";

/** The key of a team update's row; drafts use their full ID. */
export const updateKey = (fullId: string) => `update:${fullId}`;

export function inboxGroups(inbox: InboxView | null): Group[] {
  if (!inbox) return [];
  return [
    {
      label: "Team updates",
      rows: inbox.updates.map((u) => {
        const blocked = (u.new_commands ?? []).some((c) => !u.allowed.includes(c));
        const what = u.change === "added" ? "New" : u.change === "removed" ? "Removed" : "Changed";
        return {
          key: updateKey(u.fullId),
          title: u.title,
          sub: `${what} in ${libraryLabel(u.library)}`,
          chip: blocked ? "New command" : undefined,
          tone: blocked ? ("warn" as const) : undefined,
          when: shortDay(u.date),
        };
      }),
    },
    {
      label: "Drafts",
      rows: inbox.drafts.map((d) => ({
        key: d.fullId,
        title: d.title,
        sub: d.description || "A draft in Personal",
      })),
    },
  ];
}

export function proposalGroups(proposals: ProposalView[]): Group[] {
  return [
    {
      label: "Proposals",
      rows: proposals.map((p) => {
        const l = proposalLabel(p);
        return {
          key: p.fullId,
          title: p.title,
          sub: `${l.text} · ${libraryLabel(p.library)}`,
          chip: l.chip,
          tone: l.tone,
        };
      }),
    },
  ];
}

export function problemGroups(diagnostics: Diagnostic[]): Group[] {
  return [
    {
      label: "Files",
      rows: diagnostics.map((d) => ({
        key: d.path,
        title: d.file,
        sub: d.message,
        chip: d.library ? libraryLabel(d.library) : undefined,
      })),
    },
  ];
}
