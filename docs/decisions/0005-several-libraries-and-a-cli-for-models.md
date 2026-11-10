---
type: decision
status: proposed
date: 2026-10-29
projects: [prompt-library]
tags: [teams, sync, cli, agents, mcp]
---

# 0005. Several libraries, and a CLI for models

## Context
The owner asked for two things on 2026-10-29:
- **Teams** — Each person keeps a personal repo, and the team shares one too.
- **Model access** — Models use the library through a CLI rather than an MCP server, so nothing runs as a server.

## Options
1. **One library per user, sharing by copy**: simple / no team updates, copies drift.
2. **Several git-repo libraries, attached at once**: git gives permissions, review and history / IDs need namespaces.
3. **An MCP server for models**: works in apps with no shell / a long-running process.
4. **A CLI for models**: works in any agent with a shell, no setup / apps with no shell need something else.

## Decision
Options 2 and 4, with MCP over stdio later.

### Libraries
- **A library** — A git repo with the standard layout (`templates/ snippets/ context/ inbox/`). The app and CLI attach several at once: Personal, team libraries such as "Acme eng", and others (a client, an open-source collection, a plain folder).
- **Prompt IDs** — Include the library: `personal/strict-pr-review`, `acme/coroutine-audit`. A bare name looks in Personal first. If it is still ambiguous, the error lists the matches.
- **Who can edit** — Whatever the git host allows. Each library sets how your changes go in: `writes = "commit"`, or `writes = "pull-request"` (the default for team libraries).
- **Copy to Personal** — For a team prompt you can not or do not want to change for everyone. The copy keeps `forked_from`.
- **Personal state** — Pins (which can point at team prompts), drafts and settings live in Personal. Usage counts stay on each Mac.
- **Inbox** — Updates pulled from team libraries, links people send you, and your drafts.
- **Profiles** — Start with all libraries visible, labelled in the sidebar. A profile can come later: a saved set of active libraries plus a default library for new prompts.

```toml
# ~/.config/pl/config.toml
[[library]]
name = "personal"
path = "~/Prompts/personal"
remote = "git@github.com:you/prompts.git"
writes = "commit"

[[library]]
name = "acme"
path = "~/Prompts/acme"
remote = "git@github.com:acme/prompts.git"
writes = "pull-request"
```

### `pl`, the CLI and core
- **One binary** — The Mac app, the menu bar app and the terminal app all use the same core.
- **For models** — Every command takes `--json`. Exit codes are stable. `pl fill` lists the blanks it could not fill, with hints, so the model fills them from the chat ([0002](0002-fill-variables-late-in-chat-early-outside.md)).
- **Commands** — Listed in cli.md.
- **Safety for agents** — No TTY means a model runs it. Then writes go only to drafts. `sync --push`, `share`, `send`, team edits and allowing commands refuse unless the config allows them. Reading and filling are always allowed.
- **Claude Code** — A generated `prompt-library` skill teaches the commands. Pinned prompts can also become slash commands.
- **`pl mcp`** — Later. A stdio mode of the same binary, for apps with no shell (Claude desktop). The app starts it as a child process and stops it when done. No daemon and no network port, so [0003](0003-local-first-git-for-sync-and-sharing-no-server.md) still holds.

## Consequences
- **Mockups** — The sidebar lists each library with its own sync dot. Each prompt shows its ID. Settings has Libraries and Agents tabs. The terminal app labels team prompts. The "Claude Code" screen shows a model using `pl`: search, fill, ask about the missing blank, save a draft.
- **Ownership** — Pins and drafts live only in Personal, never in a team library.
- **Settled 2026-11-09** — The core is Rust. The desktop app links the core; other clients call `pl` ([0009](0009-cross-platform-desktop-app-with-tauri-2-and-a-rust-core.md)).
