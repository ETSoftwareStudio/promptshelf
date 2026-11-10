---
type: decision
status: accepted
date: 2026-10-29
projects: [prompt-library]
tags: [design, macos, ui]
---

# 0004. Margin as the visual direction for the Mac app

## Context
- **Native app** — The owner chose a native Mac app, in light and dark mode.
- **Method** — The [impeccable](https://impeccable.style) rules for tools: standard native navigation and controls. Only the palette, the type, the density and one distinctive feature change.
- **First attempt rejected** — A dark Linear/Raycast style looked AI-generated: a slate and orange template, component-kit chrome, grey-on-grey hierarchy, a gradient backdrop.
- **Evidence** — Every direction and option is in [brand/mockups/](../design/). The [exploration report](../exploration-and-design.md) has the history.

## Options
| Direction | Layout | Distinctive feature | Result |
| --- | --- | --- | --- |
| **Margin** | Three columns: sidebar, list, detail | The prompt reads as a sentence. Each value has a footnote with its source. | Chosen |
| **Ledger** | Search with tokens, sortable table, inspector | Variable tokens coloured by source | Rejected; gave the source colours |
| **Outbox** | Mail-style sidebar, sent history first | The prompt previewed as your next message in the target chat | Rejected; gave the preview before Send |

Two follow-up choices, shown side by side in `margin-options.html`:
- **1B** — The highlight itself is the source colour, and an unresolved value is a dashed outline. Chosen over 1A: one highlighter colour, with only the footnote giving the source.
- **2B** — A preview sheet before Send, which you can turn off for each chat. Chosen over 2A: pick the target from the menu, and ⌘↩ sends at once.

## Decision
Margin, in light and dark mode.

- **Prompt as a sentence** — The prompt reads as prose, with the values filled in. Each value has a numbered footnote that says where it came from.
- **Source colours** — Rose: from the conversation. Green: from the repo or files. A dashed blank: a value the app could not work out. Footnote numbers are chips in the same colour. The colours mean the same thing on every surface, the terminal app too.
- **Preview before Send** — The sheet shows the end of the target conversation, the message as it will arrive (with attachments such as the diff) and "Submit after pasting". ⌘↩ opens the preview. ⌘↩ again sends. "Don't preview for this chat" turns it off for that chat.
- **Pins** — At the top of the sidebar, ⌘1 to ⌘9.
- **Type** — SF Pro for the interface. New York for prompt text. SF Mono only for commands and paths.
- **Diffs** — Red strikethrough for removals, a blue tint for additions. Never green, so additions do not look like "from the repo".

Tokens are in [brand/README.md](../design/README.md#palette).

## Consequences
- **Changed by [0009](0009-cross-platform-desktop-app-with-tauri-2-and-a-rust-core.md) (2026-11-09)** — The app is Tauri 2 on macOS, Windows and Linux. "Native controls only" and the SF Pro / New York type no longer hold. The layout, the sentence with footnotes, the source colours and the preview before Send stay.
- **Re-tinted for the studio rule (2026-11-01)** — The studio's design rule bans hue 265–330 with chroma above 0.08. The first palette broke it: the accent was ink blue at hue 268 and the conversation colour was violet at hue 300. A colour check flagged 12 values. The accent is now blue at hue 255, and the conversation colour is rose at hue 355. Cyan (hue 215) was rejected because it looked like the blue diff insertions. Rose is far from both green (repo) and blue (accent, insertions). Footnote numbers still give the source to people who can not tell the colours apart.
- **Screens done** — Library with pins, Send preview, menu bar quick fill and preview, Share, import review, Inbox, empty states, first run, Settings (Libraries, Agents), team Proposals and the Propose sheet, a Claude Code session, and the terminal app.
- **Validation** — The owner has the full prototype (`margin-full.html`) to validate. No feedback recorded yet.
