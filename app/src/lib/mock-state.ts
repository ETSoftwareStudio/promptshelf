// The state of the browser mock: libraries, the last syncs, proposals, the
// Inbox and agents. Words in the URL hash pick a case to look at:
// `firstrun`, `noremote`, `conflict`, `pusherror`, `nogit`, `nogh`,
// `signedout`, `noidentity`, `problems`. Every name is made up.

import type {
  AgentsView,
  Environment,
  LastSync,
  LibraryInfo,
  LibraryStatus,
  ProposalView,
  SyncReport,
  UpdateView,
} from "./types";

const has = (word: string) => location.hash.includes(word);
export const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));
const now = () => Math.floor(Date.now() / 1000);

/** Tells the mock's windows that something changed, like the app does. */
export function changed(): void {
  window.dispatchEvent(new CustomEvent("mock:shelf-changed"));
}

export const state = {
  configured: !has("firstrun"),
  libraries: [
    {
      name: "personal",
      path: "~/Prompts/personal",
      remote: has("noremote") ? null : "git@example.com:you/prompts.git",
      writes: "commit",
      personal: true,
      trusted: true,
      git: true,
      exists: true,
      count: 0,
    },
    {
      name: "acme-eng",
      path: "~/Prompts/acme-eng",
      remote: "git@example.com:acme/prompts.git",
      writes: "pull-request",
      personal: false,
      trusted: false,
      git: true,
      exists: true,
      count: 0,
    },
  ] as LibraryInfo[],
  lastSync: {} as Record<string, LastSync>,
  /** Conflicts the next sync of Personal reports. */
  conflicts: has("conflict") ? ["templates/strict-pr-review.md"] : [],
  proposals: [
    {
      fullId: "acme-eng/coroutine-audit",
      library: "acme-eng",
      id: "coroutine-audit",
      title: "Coroutine audit",
      status: "changes-requested",
      number: 42,
      url: "https://example.com/acme/prompts/pull/42",
      branch: "shelf/you/coroutine-audit",
      unsent: true,
    },
  ] as ProposalView[],
  updates: [
    {
      library: "acme-eng",
      id: "commit-message",
      fullId: "acme-eng/commit-message",
      title: "Commit message, conventional",
      path: "templates/commit-message.md",
      change: "edited",
      from: "1a2b3c4d",
      to: "5e6f7a8b",
      date: "2026-11-10",
      new_commands: ["git diff --staged"],
      allowed: [],
    },
  ] as UpdateView[],
  env: {
    git: has("nogit")
      ? {}
      : {
          version: "git version 2.50.1",
          name: has("noidentity") ? null : "Sam Example",
          email: has("noidentity") ? null : "sam@example.com",
        },
    gh: has("nogh")
      ? { state: "missing" }
      : has("signedout")
        ? { state: "signed-out" }
        : { state: "signed-in", account: "sam-example" },
  } as Environment,
  agents: {
    skillPath: "~/.claude/skills/promptshelf/SKILL.md",
    skillInstalled: false,
    skillCurrent: false,
    command: null,
    commandNote: null,
    claudeConfig: "~/Library/Application Support/Claude/claude_desktop_config.json",
    claudeCommand: null,
  } as AgentsView,
};

export function lib(name: string): LibraryInfo {
  const l = state.libraries.find((x) => x.name === name);
  if (!l) throw new Error(`no library named "${name}"`);
  return l;
}

export function report(name: string): SyncReport {
  const l = lib(name);
  const r: SyncReport = { library: name, pulled: 0, pushed: 0, changes: 0 };
  if (!l.remote) r.skipped = "no remote";
  else if (name === "personal" && state.conflicts.length) r.conflicts = [...state.conflicts];
  else if (name === "personal" && has("pusherror")) {
    r.error =
      "git push failed: git@example.com: Permission denied (publickey). fatal: Could not read from remote repository.";
    r.hint = {
      code: "auth",
      message:
        "The host did not accept your login. Sign in to GitHub CLI (gh auth login) or set up an SSH key, then try again.",
      url: "https://docs.github.com/en/get-started/git-basics/caching-your-github-credentials-in-git",
    };
  } else if (name === "personal") {
    r.pulled = 1;
    r.pushed = 2;
    r.committed = "Edit strict-pr-review";
  } else if (state.updates.length) r.updates = state.updates.length;
  state.lastSync[name] = {
    at: now(),
    error: r.error ?? null,
    hint: r.hint ?? null,
    conflicts: r.conflicts ?? [],
  };
  return r;
}

export function sync(name?: string | null): SyncReport[] {
  return state.libraries.filter((l) => !name || l.name === name).map((l) => report(l.name));
}

export function status(l: LibraryInfo): LibraryStatus {
  return {
    name: l.name,
    remote: l.remote ?? null,
    branch: l.git ? "main" : null,
    ahead: l.name === "personal" && has("pusherror") ? 2 : 0,
    behind: 0,
    changes: 0,
    lastSync: state.lastSync[l.name] ?? null,
  };
}
