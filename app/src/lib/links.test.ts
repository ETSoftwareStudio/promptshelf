import { beforeAll, expect, test, vi } from "vite-plus/test";
import { cantOpen, linkPlan } from "./links";
import type { Link } from "./types";

const ready = { configured: true, busy: false };

test("open and fill select the prompt; fill also opens the Send preview", () => {
  const open: Link = { route: "open", library: "personal", id: "welcome" };
  expect(linkPlan(open, ready)).toEqual({
    kind: "prompt",
    fullId: "personal/welcome",
    send: false,
  });
  expect(linkPlan({ ...open, route: "fill" }, ready)).toEqual({
    kind: "prompt",
    fullId: "personal/welcome",
    send: true,
  });
});

test("search and new carry their text", () => {
  expect(linkPlan({ route: "search", query: "release" }, ready)).toEqual({
    kind: "search",
    query: "release",
  });
  expect(linkPlan({ route: "new", title: null }, ready)).toEqual({ kind: "new", title: "" });
  expect(linkPlan({ route: "new", title: "Standup" }, ready)).toEqual({
    kind: "new",
    title: "Standup",
  });
});

test("a bad link is only a toast", () => {
  const plan = linkPlan({ route: "bad", reason: '"import" links are not supported yet' }, ready);
  expect(plan).toEqual({
    kind: "toast",
    text: `PromptShelf can't open this link: "import" links are not supported yet`,
  });
  expect(cantOpen("no prompt personal/foo")).toBe(
    "PromptShelf can't open this link: no prompt personal/foo",
  );
});

test("an empty link only opens the window, even at first run", () => {
  expect(linkPlan({ route: "home" }, { configured: false, busy: true })).toEqual({ kind: "none" });
});

test("a link never replaces an open sheet or edit, or skips first run", () => {
  const open: Link = { route: "open", library: "personal", id: "welcome" };
  const busy = linkPlan(open, { configured: true, busy: true });
  expect(busy.kind).toBe("toast");
  const firstRun = linkPlan({ route: "new", title: null }, { configured: false, busy: false });
  expect(firstRun.kind).toBe("toast");
});

// The browser mock, as `vp dev` uses it. It reads the URL hash and sends its
// events on the window.
const target = new EventTarget();
beforeAll(() => {
  vi.stubGlobal("location", { hash: "" });
  vi.stubGlobal("window", target);
});

test("the mock hands a simulated link to the window once", async () => {
  const mock = await import("./mock");
  const seen = vi.fn();
  target.addEventListener("mock:link", seen);
  mock.mockLink({ route: "fill", library: "personal", id: "strict-pr-review" });
  expect(seen).toHaveBeenCalledOnce();
  expect(await mock.call("take_link", {})).toEqual({
    route: "fill",
    library: "personal",
    id: "strict-pr-review",
  });
  expect(await mock.call("take_link", {})).toBeNull();
});

test("a prompt that a link opened holds its commands until Run Commands", async () => {
  const { Filler } = await import("./filler.svelte");
  const filler = new Filler();
  await filler.load("personal/strict-pr-review", false);
  expect(filler.held).toBe(true);
  expect(filler.waiting()).toEqual(["diff"]);
  expect(filler.reason("diff")).toBe("Waits for Run Commands.");

  await filler.runCommands();
  expect(filler.held).toBe(false);
  expect(filler.waiting()).toEqual([]);
  expect(filler.values().diff?.source).toBe("command");
});

test("named sources still fill when commands are held", async () => {
  const { Filler } = await import("./filler.svelte");
  const filler = new Filler();
  await filler.load("personal/explain-like-a-colleague", false);
  expect(filler.values().topic?.source).toBe("clipboard");
  expect(filler.waiting()).toEqual([]);
});
