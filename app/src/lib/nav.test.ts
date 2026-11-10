import { expect, test } from "vite-plus/test";
import { navQuery, navTitle, syncSummary } from "./nav";

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
