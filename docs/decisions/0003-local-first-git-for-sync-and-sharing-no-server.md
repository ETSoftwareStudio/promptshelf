---
type: decision
status: accepted
date: 2026-10-29
projects: [prompt-library]
tags: [storage, sync, sharing, local-first]
---

# 0003. Local-first, git for sync and sharing, no server

## Context
- **Owner's rule (2026-10-29)** — No central server or API. Keep data local. If sync needs a remote, the user connects a git repo, local or remote.
- **What git gives** — History, diffs to review updates, merges, sync between Macs, and team sharing with the host's permissions.

## Options
1. **Our own sync server**: easy discovery and view counts / running cost, accounts, security work, the owner rejected it.
2. **iCloud Drive only**: no setup / no history, last write wins, no teams.
3. **A folder plus git**: history, review, teams, any host / conflicts need care ([0006](0006-sync-conflicts-and-adding-prompts.md)).

## Decision
Option 3, with iCloud Drive and "Off" as simpler sync choices.

- **Library** — A folder on the Mac, `~/Prompts` by default. It can be a git repo.
- **Sync** — git push and pull to a remote the user connects: a local path, GitHub, GitLab or self-hosted. The app calls `git` and uses the user's SSH key or credential helper, so it needs no OAuth app.
- **Other sync choices** — Off (this Mac only), or an iCloud Drive folder (no history; the last write wins).
- **Sharing, with no server** — A secret gist or GitLab snippet (needs `gh` or a token). A team repo. A public repo or plugin marketplace. An exported `.md` file or AirDrop. A `promptlib://import#<encoded prompt>` link that carries the whole prompt.
- **Pins** — One ordered list in `.promptlib/pins.yaml`, so pins sync between Macs. Pins are not in prompt frontmatter. So you can pin a team prompt you can not edit, and shared prompts do not carry your pins.
- **Usage counts and history** — On this Mac only, git-ignored, so a Send never makes a commit.
- **Summaries, with no hosted model of ours** — Apple's on-device Foundation Models by default. `claude -p` on the user's own plan as an option. Late binding inside a chat costs nothing extra.

## Consequences
- **Running cost** — Zero. Optional: the Apple Developer Program (US$99 a year) for signed builds, and a private remote (free on GitHub).
- **Not possible without a server** — Search across users (except through a public repo or GitHub topic), and view counts on shared links.
- **Conflicts** — Handled in [0006](0006-sync-conflicts-and-adding-prompts.md). The app never shows conflict markers.
- **Team updates** — A team library is a git repo you pull. Updates show in the Inbox ([0005](0005-several-libraries-and-a-cli-for-models.md)).
