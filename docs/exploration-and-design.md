---
type: report
date: 2026-10-29
projects: [prompt-library]
---

# Exploration and design

**Result** — On 2026-10-29 the owner explored a personal and team prompt library, from the shape of the files to a full clickable prototype. Six decisions came out of it. There is no code yet. This report is the history; the decisions are the current state.

## What the library holds
- **Kinds of prompt** — Snippets, templates (with variables), context packs (partials such as `{{> about-me}}`), personas, workflows, meta-prompts.
- **Frontmatter** — `id, kind, tags, vars, includes, models, targets, source, forked_from, version`.
- **Folders** — `snippets/ templates/ context/ inbox/`.
- **Generated outputs** — Claude Code skills, MCP prompts (they show up as slash commands), launcher exports such as Raycast.
- **Capture** — A `/save-prompt` skill. Repeated prompts found in `~/.claude/history.jsonl`. An Inbox to tidy later.
- **Over time** — git versions, usage counts, notes on how each model handles a prompt, small evals to run again when new models ship.

## Sharing
- **Tiers** — An unlisted link (secret gist). A team library (a shared git repo; updates arrive as a diff). A public collection (a repo or a Claude Code plugin marketplace).
- **Before sharing** — Private includes become variables. A scan for secrets and personal details. Model notes are optional. Usage stats are off by default.
- **On import** — A plain-language summary of what the prompt can do. Warnings on lines that look like prompt injection. Keep both or replace. Follow updates or copy once.

## How the design got here
The final prototype is [`design/margin-full.html`](design/margin-full.html). The earlier `explorations/` files are not in this repo; they are listed as history.

| Step | Files | Result |
| --- | --- | --- |
| 1. First look: dark Linear/Raycast style, OKLCH | `explorations/promptlib.html`, `explorations/sharing.html` | Rejected: looked AI-generated (slate and orange template, kit chrome, grey on grey, gradient backdrop) |
| 2. Four directions: terminal, reference book, "the prompt is the form", menu bar | `explorations/directions.html` | Terminal and menu bar kept as extra surfaces, plus a regular window ([0001](decisions/0001-one-core-three-surfaces.md)) |
| 3. Filling from context and Send, in the "prompt is the form" style | `explorations/context-fill.html` | Led to [0002](decisions/0002-fill-variables-late-in-chat-early-outside.md) |
| 4. Three native Mac directions: Margin, Ledger, Outbox | `explorations/native-1.html` to `native-3.html`, `explorations/native-directions.html` | Margin chosen ([0004](decisions/0004-margin-as-the-visual-direction-for-the-mac-app.md)) |
| 5. Pins and sync settings | `explorations/margin-pins-sync.html` | Pins in the sidebar, ⌘1 to ⌘9; storage in [0003](decisions/0003-local-first-git-for-sync-and-sharing-no-server.md) |
| 6. Two open choices side by side | `explorations/margin-options.html` | 1B (values tinted by source) and 2B (preview before Send) |
| 7. Remaining screens | `explorations/margin-screens-1.html`, `explorations/margin-screens-2.html` | Menu bar fill and preview, Share, import review, Inbox, empty states |
| 8. Full prototype | `margin-full.html` | Every screen in light and dark, plus libraries, agents, proposals and the terminal app |
| 9. Re-tint for the studio colour rule (2026-11-01) | `margin-full.html` | Accent hue 268 → 255, conversation violet (300) → rose (355). See [0004](decisions/0004-margin-as-the-visual-direction-for-the-mac-app.md#consequences) |
| 10. Raycast extension (2026-11-02) | `margin-full.html` (Raycast tab) | List, Actions and Fill screens. See [0008](decisions/0008-a-raycast-extension-on-top-of-pl.md) |

### The three native directions
1. **Margin** — Three columns: floating sidebar, list, detail. The prompt reads as a sentence with highlighted values. Each value has a footnote with its source. A footnote asks you when the app could not work a value out. Prompt text in New York.
2. **Ledger** — A search field with tokens, a segmented control, a zebra-striped sortable table and an inspector. Variable tokens are coloured by source: violet for the conversation, green for the repo, amber for the clipboard, grey for you. Graphite shell.
3. **Outbox** — A Mail-style sidebar, sent history first, and a compose screen with *To* and *Prompt* lines. The prompt shows as your next message under the end of the target conversation. Amber marks what needs input.

Margin won. Ledger gave it the source colours. Outbox gave it the preview before Send.

## Questions answered on the way
Each answer is now part of a decision.

| Question | Answer | Where |
| --- | --- | --- |
| Does it need a server, and what does it cost? | No server. Zero running cost. US$99 a year only for signed builds. | [0003](decisions/0003-local-first-git-for-sync-and-sharing-no-server.md) |
| What is the Inbox for? | Updates pulled from team libraries, links people send you, and your drafts. | [0005](decisions/0005-several-libraries-and-a-cli-for-models.md) |
| Can teams and several "profiles" use it? | Yes: Personal plus team libraries, each a git repo, with namespaced IDs. | [0005](decisions/0005-several-libraries-and-a-cli-for-models.md) |
| How do models use it? | Through the `pl` CLI with `--json`. MCP over stdio later. | [0005](decisions/0005-several-libraries-and-a-cli-for-models.md) |
| How are conflicts and uncommitted work handled? | Checkpoint commits, `rebase --autostash`, a resolution sheet, no markers. | [0006](decisions/0006-sync-conflicts-and-adding-prompts.md) |
| One pull request per prompt, or one per person? | One per prompt by default, with grouping. A team can choose batched. | [0006](decisions/0006-sync-conflicts-and-adding-prompts.md) |
| Is it tied to GitHub? | No. Sync is plain git. GitLab and Gitea open pull requests with push options. | [0006](decisions/0006-sync-conflicts-and-adding-prompts.md) |
| Does a second proposal lose the first? | No. Each proposal has its own branch and worktree. | [0006](decisions/0006-sync-conflicts-and-adding-prompts.md) |
| How does the app know a pull request was merged? | Host API first, then plain git checks. Then it cleans up `pl/*` branches and worktrees. | [0006](decisions/0006-sync-conflicts-and-adding-prompts.md) |
| If the library stays on `main`, how are pull requests made and seen? | From the proposal worktrees. A "Your proposals" view shows their state. | [0006](decisions/0006-sync-conflicts-and-adding-prompts.md) |

## Open
Open on 2026-10-29. Each one is now settled in a later decision (0007 to 0010) or in the build:
- **Name** — "Prompt Library" and `pl` are working names.
- **Core language** — Go or Rust. Also: the Mac app links the core, or calls `pl`.
- **Inbox rule** — Check team updates by what a prompt can do, not by its text ([0006](decisions/0006-sync-conflicts-and-adding-prompts.md#consequences)).
- **Send targets** — T3 deep link or API, Ghostty AppleScript text input, the claude.ai `?q=` prefill URL.
- **Going public** — Open source under MIT ([0007](decisions/0007-open-source-mit.md)). Confirm the licence and add `CONTRIBUTING.md` first.
- **Validation** — The owner has not yet given feedback on `margin-full.html`. The build plan comes after that.
