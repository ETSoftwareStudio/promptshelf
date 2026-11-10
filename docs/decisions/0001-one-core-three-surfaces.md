---
type: decision
status: accepted
date: 2026-10-29
projects: [prompt-library]
tags: [architecture, ui]
---

# 0001. One core, three surfaces

## Context
- **Fast retrieval** — Finding and using a prompt must take a second or two. A full window is too slow for that.
- **Room to edit** — Editing, testing templates, sorting the Inbox, sharing and import review need a full window.
- **Same behaviour everywhere** — Templates, variables, sync and checks must act the same on every surface.

## Options
1. **GUI only**: one app to build / too slow for quick retrieval.
2. **Terminal app and menu bar only**: fast / no room for editing and review, and the owner wants a regular window.
3. **Each surface with its own logic**: each surface is free / the behaviour drifts apart.
4. **One core, thin surfaces**: same behaviour everywhere / the core needs a stable interface.

## Decision
Option 4. Three surfaces on one core:

1. **Terminal app** — Opens with `pl` (lazygit style). Keyboard only. It also runs inside T3 and the Claude Code terminal.
2. **Menu bar app** — Native. Pins, recent prompts, quick fill, "Save selection as prompt".
3. **Library window** — Browse, edit, test templates with real values, the Inbox, history, sharing, import review.

The core owns the library files, templates and includes, the variable resolver ([0002](0002-fill-variables-late-in-chat-early-outside.md)), the targets for Send, usage counts and the share and import checks. Models use the same core through the CLI ([0005](0005-several-libraries-and-a-cli-for-models.md)).

## Consequences
- **Interface** — The core is the `pl` binary with `--json` output. The desktop app links the `pl-core` crate; external clients call `pl --json` ([0009](0009-cross-platform-desktop-app-with-tauri-2-and-a-rust-core.md)).
- **Generated outputs** — Claude Code skills and, later, `pl mcp` over stdio come from the same core.
- **Look** — The menu bar app follows native macOS conventions. The window follows [0004](0004-margin-as-the-visual-direction-for-the-mac-app.md).
