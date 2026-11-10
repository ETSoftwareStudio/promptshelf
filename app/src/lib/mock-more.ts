// The browser mock's commands for libraries, remotes, sync conflicts,
// proposals, the Inbox, agents and prompt edits.

import { PROMPTS } from "./mock-data";
import { changed, lib, report, state, status, wait } from "./mock-state";
import type { InboxView, LibraryInfo, NewLibrary, Summary } from "./types";

function asSummary(p: (typeof PROMPTS)[number]): Summary {
  return {
    fullId: `${p.library}/${p.id}`,
    id: p.id,
    library: p.library,
    kind: p.kind,
    title: p.title,
    description: p.description,
    tags: p.tags,
    uses: p.uses,
  };
}

const DIFF = `diff --git a/templates/commit-message.md b/templates/commit-message.md
--- a/templates/commit-message.md
+++ b/templates/commit-message.md
@@ -1,6 +1,6 @@
 ---
 title: Commit message, conventional
 vars:
-  diff: { from: "git diff" }
+  diff: { from: "git diff --staged" }
 ---
-Write a commit message for this change:
+Write a Conventional Commits message for this change:
`;

/** The new commands of the mock. `undefined` means "not mine". */
export async function callMore(cmd: string, a: Record<string, unknown>): Promise<unknown> {
  const s = (k: string) => a[k] as string;
  switch (cmd) {
    case "pick_folder":
      await wait(200);
      return "~/Documents/Prompts";
    case "init_personal":
      await wait(400);
      state.configured = true;
      lib("personal").path = s("path") || "~/Prompts/personal";
      lib("personal").remote = s("url") || null;
      changed();
      return lib("personal").path;
    case "environment":
      return state.env;
    case "set_git_identity":
      state.env.git = { ...state.env.git, name: s("name"), email: s("email") };
      return state.env;
    case "library_status":
      return state.libraries.map(status);
    case "add_library": {
      await wait(500);
      const n = a.library as NewLibrary;
      if (!/^[a-z0-9-]+$/.test(n.name))
        throw new Error(`library name "${n.name}" must use only a-z, 0-9 and -`);
      if (state.libraries.some((l) => l.name === n.name))
        throw new Error(`a library named "${n.name}" exists`);
      const url = /:\/\/|@.*:|\.git$/.test(n.source);
      state.libraries.push({
        name: n.name,
        path: url ? n.path || `~/Prompts/${n.name}` : n.source,
        remote: url ? n.source : null,
        writes: n.writes ?? (url ? "pull-request" : "commit"),
        personal: false,
        trusted: n.trusted ?? !url,
        git: url,
        exists: true,
        count: 0,
      });
      changed();
      return n.name;
    }
    case "update_library": {
      const l = lib(s("name"));
      if (a.writes) l.writes = a.writes as LibraryInfo["writes"];
      if (a.trusted !== null && a.trusted !== undefined) l.trusted = a.trusted as boolean;
      changed();
      return undefined;
    }
    case "remove_library": {
      const l = lib(s("name"));
      state.libraries = state.libraries.filter((x) => x !== l);
      changed();
      return l.path;
    }
    case "connect_remote":
    case "create_github_repo": {
      await wait(600);
      const l = lib(s("name"));
      l.remote =
        cmd === "connect_remote" ? s("url") : `git@example.com:sam-example/${s("repo")}.git`;
      l.git = true;
      changed();
      return report(l.name);
    }
    case "resolve_conflict":
      await wait(500);
      state.conflicts = [];
      if (a.how === "both") {
        const p = PROMPTS.find((x) => x.id === "strict-pr-review")!;
        PROMPTS.push({ ...p, id: "strict-pr-review-2", title: `${p.title} (this computer)` });
      }
      changed();
      return report(s("library"));
    case "agents_status":
      return state.agents;
    case "install_skill":
      state.agents.skillInstalled = state.agents.skillCurrent = true;
      return state.agents.skillPath;
    case "install_command":
      state.agents.command = "~/.local/bin/shelf";
      return { path: "~/.local/bin/shelf", profile: "~/.zprofile" };
    case "add_to_claude_desktop":
      if (!state.agents.command) throw new Error("Install the shelf command first.");
      state.agents.claudeCommand = state.agents.command;
      return {
        path: state.agents.claudeConfig,
        backup: `${state.agents.claudeConfig}.bak`,
        changed: true,
      };
    case "list_proposals":
      if (a.refresh) await wait(500);
      return state.proposals;
    case "proposal_text":
      return {
        title: `Edit ${s("id").split("/")[1]}`,
        body: `Proposed with PromptShelf: \`${s("id")}\`.`,
      };
    case "propose": {
      await wait(600);
      const p = state.proposals.find((x) => x.fullId === s("id"));
      if (!p) throw new Error(`no proposal for ${s("id")}`);
      p.status = "sent";
      p.unsent = false;
      p.number ??= 43;
      p.url ??= `https://example.com/acme/prompts/pull/${p.number}`;
      changed();
      return p;
    }
    case "withdraw_proposal":
      state.proposals = state.proposals.filter((x) => x.fullId !== s("id"));
      changed();
      return undefined;
    case "get_inbox":
      return {
        drafts: PROMPTS.filter((p) => p.kind === "draft").map(asSummary),
        updates: state.updates,
      } satisfies InboxView;
    case "update_diff":
      return DIFF;
    case "allow_commands": {
      const u = state.updates.find((x) => x.fullId === s("id"));
      if (u) u.allowed = [...(u.new_commands ?? [])];
      changed();
      return u?.allowed ?? [];
    }
    case "keep_previous":
    case "copy_prompt": {
      const id = s("id").split("/").pop()!;
      const p = PROMPTS.find((x) => `${x.library}/${x.id}` === s("id"))!;
      PROMPTS.push({ ...p, library: "personal", id: `${id}-2`, uses: 0 });
      changed();
      return {
        id: `personal/${id}-2`,
        path: `~/Prompts/personal/templates/${id}-2.md`,
        forked_from: s("id"),
      };
    }
    case "dismiss_update":
      state.updates = state.updates.filter((x) => x.fullId !== s("id"));
      changed();
      return undefined;
    case "delete_prompt": {
      const i = PROMPTS.findIndex((x) => `${x.library}/${x.id}` === s("id"));
      if (PROMPTS[i]?.library === "acme-eng")
        throw new Error("acme-eng is a team library: changes go in as pull requests.");
      PROMPTS.splice(i, 1);
      changed();
      return undefined;
    }
    case "rename_prompt": {
      const p = PROMPTS.find((x) => `${x.library}/${x.id}` === s("id"))!;
      p.title = s("title");
      p.id = s("newId");
      changed();
      return `${p.library}/${p.id}`;
    }
    case "move_prompt": {
      const p = PROMPTS.find((x) => `${x.library}/${x.id}` === s("id"))!;
      p.library = s("library");
      p.kind = a.kind as typeof p.kind;
      if (lib(p.library).writes === "pull-request") {
        state.proposals.push({
          fullId: `${p.library}/${p.id}`,
          library: p.library,
          id: p.id,
          title: p.title,
          status: "not-sent",
          branch: `shelf/you/${p.id}`,
          unsent: true,
        });
      }
      changed();
      return `${p.library}/${p.id}`;
    }
    case "read_file":
      return "---\ntitle: Broken\nvars: [\n---\nA prompt with a typo.\n";
    case "save_file":
      if (String(a.text).includes("vars: ["))
        throw new Error("The file still has an error: did not find expected node content");
      changed();
      return undefined;
    case "open_url":
      return undefined;
  }
  return undefined;
}
