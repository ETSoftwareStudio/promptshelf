---
type: decision
status: proposed
date: 2026-10-29
projects: [prompt-library]
tags: [git, sync, conflicts, editing, pull-requests]
---

# 0006. Sync, conflicts and adding prompts

## Context
- **git is the sync** — [0003](0003-local-first-git-for-sync-and-sharing-no-server.md) makes each library a git repo. [0005](0005-several-libraries-and-a-cli-for-models.md) adds team libraries where changes go in by pull request.
- **Questions to answer** — How conflicts are handled. If uncommitted work is allowed. How a new prompt is added. One pull request per prompt or one per person. How several proposals live side by side. How the app knows a pull request was merged, and what it cleans up. If it works on hosts other than GitHub.
- **Rule** — A user never sees git conflict markers, and the app never loses an edit.

## Options
| Topic | Options | Chosen |
| --- | --- | --- |
| Commits | On every save / at checkpoints / by hand only | Checkpoints, with a manual mode |
| Pull | Merge / rebase | `rebase --autostash` |
| Conflicts | Show markers / a resolution sheet | A resolution sheet |
| Team pull requests | One per person / one per prompt | One per prompt; a team can choose batched |
| Several proposals | Switch branches in one folder / one worktree each | One worktree each |
| Merge check | Host API only / API, then plain git | API, then plain git |

## Decision

### Local work
- **Two layers** — The editor writes to disk within about a second, so nothing lives only in memory. Commits come at checkpoints: when you leave the prompt, or after 30 seconds with no typing. A commit looks like "Edit strict-pr-review".
- **Uncommitted work** — Allowed. With "Commit when I save" off, changes stay uncommitted. The sidebar shows "3 changes · Commit…".
- **Edits from outside the app** — vim, Claude Code and `pl` edits are picked up by a folder watch (FSEvents). They count the same as edits in the app.
- **A file changes on disk while you edit it** — A banner shows "Changed elsewhere · Compare". The other version never overwrites yours.

### Sync, for libraries where you commit directly (Personal)
1. Checkpoint-commit finished edits.
2. `git fetch`.
3. `git rebase --autostash` onto the remote. Uncommitted work is stashed and put back.
4. `git push`. If the remote rejects the push because someone pushed first, go back to step 2, a limited number of times.

The app never force-pushes.

- **Why conflicts are rare** — Each prompt is its own file. Pins (`.promptlib/pins.yaml`) use a merge driver set up by `pl` that keeps both sides' pins.
- **Real conflicts** — The same lines edited on two Macs, or two new prompts with the same ID. The app aborts the rebase, so the repo is never half-merged. Your local version stays in use, marked "Edited on two Macs".
- **Resolution sheet** — Both versions side by side, with four choices: this Mac's version, the other version, keep both (saved as `<id>-2`), or edit them together.

### Team libraries: the main folder stays on `main`
- **Main folder** — `~/Prompts/acme` is always on `main`. You never commit to it, so each pull is a fast-forward. The app never switches its branch.
- **Each proposal** — Its own branch `pl/<you>/<id>` and its own git worktree: a separate folder for that branch, in `~/Library/Application Support/pl/proposals/<library>/<id>`. The app creates the worktree when you start to edit a team prompt. Its first state is "Not sent".
- **No lost proposals** — Nothing switches branches, so one proposal can not hide or overwrite another. Several can be open and edited at once.

What the core runs for a new proposal:

```sh
git -C ~/Prompts/acme worktree add -b pl/you/release-checklist <worktree> origin/main
git -C <worktree> commit -am "Add release-checklist"   # at checkpoints
git -C <worktree> push -u origin pl/you/release-checklist   # on Propose…
```

Then it opens the pull request: `gh pr create` on GitHub, a push option on GitLab, AGit on Gitea, or a link.

### Pull requests: one per prompt, sent when you choose
- **Send** — Edits stay on your Mac until you click **Propose…** or run `pl propose <id>`. Nothing half-written reaches the team.
- **One per prompt** — The default. Each prompt is reviewed, merged or reverted alone, so a debate about one prompt does not block another.
- **More edits** — Edits to a prompt with an open pull request stay local until **Update Pull Request**. They then go into the same pull request as new commits.
- **Grouping** — The Propose sheet offers "New pull request" (the default) or "Add to #42". Grouped proposals share one branch and worktree.
- **Dependencies** — If a new template includes a new snippet that is not merged yet, both go in the same pull request. So `main` never points at a missing file.
- **Team override** — `pull_requests = "batched"` in the team repo's `.promptlib/team.toml`. Then each person gets one rolling pull request until it is merged.
- **When `main` moves** — Update Pull Request first rebases the proposal on the new `main`. A conflict opens the resolution sheet.

### Where proposals show up
- **Team library list** — An "All / Your proposals" switch. Each proposal shows its state: Not sent, "#39 Waiting for review", "#42 Changes requested".
- **Detail view** — A banner with the reviewer's latest request (read through the host's API). "Compare with main". "Update Pull Request" and "Withdraw".
- **Propose sheet** — Title, description, "New pull request" or "Add to #N", then the same checks as Share.
- **Your version** — On your Mac, a prompt you proposed fills and sends your version. `pl fill --main` uses the team's version. Teammates keep the team's version until the merge.
- **CLI** — `pl propose <id> [--into N]`, `pl proposals`.
- **Mockup** — The Proposals tab and the Propose sheet in `margin-full.html`.

### Knowing a proposal was merged
Checked on every sync (every 5 minutes, or Sync Now), in this order:

1. **Host API** — If a login exists (`gh`, `glab`, `tea` or a token): the pull request state, open, merged or closed. This answer is final.
2. **Plain git** — `git merge-base --is-ancestor` finds merge commits and fast-forwards. A patch-id compare of the whole proposal, or `git diff --quiet` on its files, finds squash and rebase merges.
3. **No proof** — The state is "Sent · status unknown", with a link to the host. If the remote branch is gone: "Closed or merged with edits · check on host".

### After a merge, and pruning
- **No branch switch** — The main folder is already on `main`. `fetch` plus `merge --ff-only` brings the change in. The overlay goes away because `main` now has your text.
- **Clean-up** — `git worktree remove` and `git branch -D pl/<you>/<id>`. This is safe because the content check confirmed the change is in `main`. The app also deletes the remote branch if it is yours and the host did not delete it.
- **Edits after the last send** — Never deleted. They become a new "Not sent" proposal on the new `main`.
- **Closed without merge** — The local version is marked "Not merged", with Copy to Personal or Discard. The worktree stays until you choose.
- **Regular pruning** — `git fetch --prune` on every sync. `git worktree prune` for stale records. Only `pl/*` branches are deleted, and only after a merge or Discard. `git maintenance register` on each library for background gc. History is never squashed or rewritten.
- **Never pruned automatically** — Drafts in the Inbox, and closed proposals that were not merged. These get an Inbox reminder after 30 days.

### Any git host, not only GitHub
- **Sync** — Plain git with the user's SSH key or credential helper: GitHub, GitLab, Gitea/Forgejo, Bitbucket, a bare repo on a NAS, another folder.
- **Host-specific parts** — Only three: open a pull request ("merge request" on GitLab), read its state, and secret snippets for sharing. A small adapter in the core does these. The core picks it from the remote URL, or the library sets `forge = "gitlab"` and `api = "https://git.company.com"`.
- **Open a pull request with no token** (docs checked 2026-10-29):
  - **GitLab 11.10+** — `git push -o merge_request.create -o merge_request.target=main` ([push options](https://docs.gitlab.com/ee/user/project/push_options.html)).
  - **Gitea 1.13+ and Forgejo** — AGit: `git push origin HEAD:refs/for/main/<topic> -o title=… -o description=…`. More pushes to the same topic update the same pull request ([Gitea](https://docs.gitea.com/usage/agit), [Forgejo](https://forgejo.org/docs/v16.0/user/agit-support/)).
  - **GitHub** — Needs the API, through `gh` or a token. Without either, the app pushes the branch and shows GitHub's "create pull request" link.
- **Read state** — Needs the API (`gh`, `glab`, `tea`, or a token in the macOS Keychain). Without one, the state shows "Sent" with a link.
- **Sharing** — "Secret snippet" is a GitHub gist or a GitLab snippet. Gitea has no gists, so the Share sheet hides that choice there. Links and files need no host.
- **Team settings** — `.promptlib/team.toml` lives in the repo, so it works on every host.

### Adding a prompt
- **In the app** — ⌘N. Pick the library (default: the "Save new prompts to" setting) and the kind. The title sets the ID, a slug unique in the library. Write the sentence, select text, choose "Make Blank" and pick where its value comes from. The file is `templates/<id>.md`. It is committed at the next checkpoint and pushed on sync. In a team library it becomes a proposal.
- **Fixed ID** — The ID stays when the title changes, because pins, slash commands and models use it. "Rename ID…" changes it everywhere.
- **Quick capture** — "Save selection as prompt", "Turn this chat into a prompt" and `pl save` from a model make a draft in Personal's `inbox/`. "Move to Library" turns it into a prompt.
- **Terminal** — `pl new` opens `$EDITOR` with a new file. A person in a terminal can run `pl save --lib personal` to save straight to the library.
- **A file added by hand** — Works. Missing frontmatter gets defaults; the ID comes from the file name.

## Consequences
- **Team updates in the Inbox (decided 2026-11-09, owner)** — Check what a prompt can do, not its text. Text changes apply on pull and show in the Inbox for information. A new or changed command or file read stays off until you allow it. "Keep previous version" copies the old text to Personal. This replaces a per-prompt "Accept Update" step, which git can not do anyway: a fast-forward can not hold back one file.
- **Host logins** — Pull request state needs `gh`, `glab`, `tea` or a token. Without one, the app still syncs but shows less.
- **Disk** — One worktree per open proposal. The repos are small text, so the cost is low.
