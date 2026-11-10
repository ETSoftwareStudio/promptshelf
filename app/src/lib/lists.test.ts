import { expect, test } from "vite-plus/test";
import { inboxGroups, problemGroups, proposalGroups, updateKey } from "./lists";
import type { InboxView } from "./types";

test("inboxGroups marks updates with a new command", () => {
  const inbox: InboxView = {
    drafts: [
      {
        fullId: "personal/idea",
        id: "idea",
        library: "personal",
        kind: "draft",
        title: "Idea",
        tags: [],
        uses: 0,
      },
    ],
    updates: [
      {
        library: "acme-eng",
        id: "review",
        fullId: "acme-eng/review",
        title: "Review",
        path: "templates/review.md",
        change: "edited",
        from: "a",
        to: "b",
        date: "2026-11-10",
        new_commands: ["git log"],
        allowed: [],
      },
    ],
  };
  const [updates, drafts] = inboxGroups(inbox);
  expect(updates.rows[0]).toMatchObject({
    key: updateKey("acme-eng/review"),
    sub: "Changed in Acme eng",
    chip: "New command",
    tone: "warn",
  });
  expect(drafts.rows[0]).toMatchObject({ key: "personal/idea", sub: "A draft in Personal" });
  inbox.updates[0].allowed = ["git log"];
  expect(inboxGroups(inbox)[0].rows[0].chip).toBeUndefined();
  expect(inboxGroups(null)).toEqual([]);
});

test("proposalGroups and problemGroups", () => {
  const [g] = proposalGroups([
    {
      fullId: "acme-eng/a",
      library: "acme-eng",
      id: "a",
      title: "A",
      status: "not-sent",
      branch: "shelf/you/a",
      unsent: true,
    },
  ]);
  expect(g.rows[0]).toMatchObject({ chip: "Not sent", sub: "Only on this computer · Acme eng" });
  const [p] = problemGroups([
    {
      path: "/x/templates/b.md",
      message: "bad",
      library: "personal",
      file: "templates/b.md",
      help: "",
    },
  ]);
  expect(p.rows[0]).toMatchObject({
    key: "/x/templates/b.md",
    title: "templates/b.md",
    chip: "Personal",
  });
});
