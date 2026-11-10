import { expect, test } from "vite-plus/test";
import { navQuery, navTitle, syncState, syncSummary } from "./nav";

test("navTitle", () => {
  expect(navTitle("all")).toBe("All Prompts");
  expect(navTitle("kind:draft")).toBe("Inbox");
  expect(navTitle("kind:snippet")).toBe("Snippets");
  expect(navTitle("lib:acme-eng")).toBe("Acme eng");
});

test("navQuery", () => {
  expect(navQuery("recent", "x")).toEqual({ text: "x", view: "recent", kind: null, library: null });
  expect(navQuery("kind:context", "")).toMatchObject({ kind: "context", view: "all" });
  expect(navQuery("lib:personal", "")).toMatchObject({ library: "personal" });
});

test("syncSummary", () => {
  const ok = { library: "personal", pulled: 0, pushed: 0, changes: 0 };
  expect(syncSummary([ok])).toEqual({ bad: false, message: null });
  expect(syncSummary([{ ...ok, pulled: 2, pushed: 1 }]).message).toBe("Pulled 2, pushed 1.");
  const conflict = syncSummary([ok, { ...ok, library: "acme-eng", conflicts: ["templates/a.md"] }]);
  expect(conflict.bad).toBe(true);
  expect(conflict.message).toContain("Acme eng: templates/a.md");
  expect(syncSummary([{ ...ok, error: "git push failed" }]).message).toBe(
    "Personal: git push failed",
  );
});

test("syncSummary uses the plain words of a hint", () => {
  const r = {
    library: "personal",
    pulled: 0,
    pushed: 0,
    changes: 0,
    error: "Permission denied (publickey)",
    hint: { code: "auth", message: "The host did not accept your login." },
  };
  expect(syncSummary([r]).message).toBe("Personal: The host did not accept your login.");
});

test("syncState", () => {
  const info = { git: true, exists: true };
  const s = { name: "personal", remote: "git@x:y.git", ahead: 0, behind: 0, changes: 0 };
  expect(syncState(s, { git: true, exists: false })).toEqual({ text: "Folder missing", bad: true });
  expect(syncState(s, { git: false, exists: true }).bad).toBe(false);
  expect(syncState({ ...s, remote: null }, info).text).toBe("No remote: this computer only");
  expect(syncState(s, info).text).toBe("Not synced yet");
  expect(syncState({ ...s, ahead: 2, changes: 1 }, info).text).toBe("2 to push · 1 change");
  const failed = { ...s, lastSync: { at: 0, error: "boom", conflicts: [] } };
  expect(syncState(failed, info)).toEqual({ text: "The last sync failed", bad: true });
  const conflict = { ...s, lastSync: { at: 0, conflicts: ["a.md"] } };
  expect(syncState(conflict, info).text).toBe("Edited here and on the remote");
});

test("navTitle of the new views", () => {
  expect(navTitle("inbox")).toBe("Inbox");
  expect(navTitle("proposals")).toBe("Your proposals");
  expect(navTitle("problems")).toBe("Files to fix");
});
