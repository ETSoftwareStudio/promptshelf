---
type: decision
status: proposed
date: 2026-10-29
projects: [prompt-library]
tags: [templating, context, integration]
---

# 0002. Fill variables late in chat, early outside

## Context
- **Typing values is slow** — A template with five variables is slower than writing the prompt again. Most values are already in the conversation, the repo or the clipboard.
- **Two places** — Inside a chat, the model sees the conversation. Outside a chat (menu bar, window, terminal), the app must find the values itself.
- **Transcripts on disk** — Claude Code and T3 write session transcripts to `~/.claude/projects/<project>/<session>.jsonl` (checked 2026-10-29). claude.ai and ChatGPT conversations can not be read locally.

## Options
1. **Always ask the user**: simple / slow, and the reason the library goes unused.
2. **Late binding only**: the model fills blanks / does not work for apps the core can not read.
3. **Late binding in chat, early binding outside**: works everywhere / two code paths.

## Decision
Option 3.

- **Late binding (inside a chat)** — A prompt pulled into a chat (skill, MCP prompt, or Send with blanks open) carries an instruction: fill unset variables from the conversation, and ask only about what is unclear. The app reads nothing.
- **Early binding (outside a chat)** — Each variable declares where its value comes from (`from:`). The resolver suggests a value with its source. You confirm before Send.

```yaml
vars:
  focus:    { from: conversation, hint: "what the user is worried about" }
  diff:     { from: "git diff --staged" }
  ticket:   { from: branch, pattern: "[A-Z]+-\\d+" }
  audience: { default: developers }
```

- **Sources** — Claude Code and T3 transcripts, the repo of the front terminal, the selection and the clipboard (Accessibility API), defaults, last used.
- **Send targets** — Paste into the front app, with optional submit. `tmux send-keys` or terminal scripting into a running Claude Code. A new `claude "…"` session in the repo. An MCP pull from inside the chat.
- **Default target** — Send goes to the session the context came from.

## Consequences
- **Summaries** — The app reads transcripts locally and summarises them with a small model ([0003](0003-local-first-git-for-sync-and-sharing-no-server.md)). It never does this for shared prompts.
- **Shell resolvers** — `from: "git …"` in an imported prompt stays off until you allow it, during import review.
- **Checks still to do** — A T3 deep link or API to send text into a thread. Ghostty AppleScript text input. The claude.ai `?q=` prefill URL.
- **Design** — Each value shows its source by colour and footnote ([0004](0004-margin-as-the-visual-direction-for-the-mac-app.md)). An unresolved value is a dashed blank.
