---
type: decision
status: proposed
date: 2026-11-02
projects: [prompt-library]
tags: [raycast, integration, surfaces]
---

# 0008. A Raycast extension on top of pl

## Context
- **Owner's request (2026-11-02)** — Prompt Library also works in Raycast.
- **Why** — Many Mac users open everything from Raycast. For them, Raycast replaces the menu bar app as the fast way in.
- **Raycast's own features** — Raycast Snippets and AI Commands store text, but they have no `from:` resolvers, no libraries and no git sync.
- **Store rules** (checked 2026-11-02, [Prepare an extension for the Store](https://developers.raycast.com/basics/prepare-an-extension-for-store)) — The `license` field must be MIT. Every published extension is open source in [raycast/extensions](https://github.com/raycast/extensions). Calling known binaries is fine. Do not bundle opaque or heavy binaries. If the extension downloads a binary, it must come from a trusted place with a hash check.

## Options
1. **Export prompts as Raycast Snippets**: no code / a static copy: no resolvers, no sync, no teams.
2. **Script Commands**: a few shell scripts / no list, detail or form views, not in the Store.
3. **An extension with its own logic in TypeScript**: no `pl` install needed / a second copy of the core, against [0001](0001-one-core-three-surfaces.md).
4. **An extension that calls `pl --json`**: full Raycast UI, the same behaviour as every other surface / the user installs `pl` first.

## Decision
Option 4. A Raycast extension, "Prompt Library", is one more thin client of the `pl` core.

### Commands
- **Search Prompts** — A list: pins first, then matches, with a library dropdown. The detail shows the filled prompt and its blanks as coloured tags: rose for the conversation, green for the repo, dashed for "ask". ⌘1 to ⌘9 paste the pinned prompts.
- **Fill** — A form that opens only when a blank needs you. Values the core found are filled in, with their source under the field. ⌘↵ pastes into the app that was in front before Raycast (`Clipboard.paste`).
- **Save Selection as Prompt** — Reads the selection (`getSelectedText`) and runs `pl save`. The result is a draft in Personal's Inbox.

### Actions (⌘K)
| Action | Shortcut | What it does |
| --- | --- | --- |
| Fill and Paste | ↵ | Pastes at once if every blank is filled; otherwise opens Fill. |
| Copy Filled Prompt | ⌘↵ | Copies instead of pasting. |
| Send to Claude Code… | ⌘S | Hands off to `pl send`, which uses the targets in [0002](0002-fill-variables-late-in-chat-early-outside.md). |
| Pin / Unpin | ⌘⇧P | Changes `.promptlib/pins.yaml`, shared with every surface. |
| Open in Prompt Library | ⌘O | Opens the prompt in the Mac app. |
| Create Quicklink | ⌘⇧L | Makes a Raycast Quicklink to `raycast://extensions/<owner>/prompt-library/fill?arguments=…`, so one prompt gets its own alias. |

### AI tools (Raycast Pro, macOS)
- **Read tools** — `search-prompts`, `get-prompt`, `fill-prompt`. No confirmation. In Raycast AI, `@prompt-library` finds a prompt and fills its blanks from the chat, which is late binding ([0002](0002-fill-variables-late-in-chat-early-outside.md)).
- **Write tool** — `save-draft`, with a `Tool.Confirmation`. It writes only to the Inbox, the same limit as any model ([0005](0005-several-libraries-and-a-cli-for-models.md)).

### How it calls the core
- **Binary** — `execFile` of `pl … --json`. A preference holds the path; the default finds `/opt/homebrew/bin/pl` or `/usr/local/bin/pl`. The extension never downloads or bundles `pl`.
- **No `pl`** — An empty view: "Install Prompt Library", with the Homebrew command to copy.
- **Context** — Raycast can not see the conversation. `pl fill --context front` resolves values from the front app's repo and its newest transcript, the same as the menu bar app.
- **Agent rules** — `pl` sees no TTY, so the model rules apply. UI actions that need a write (Pin) pass `--user-action`. The AI tools never pass it. This is a guardrail against mistakes, not a security boundary.
- **Colours** — Raycast tags take hex values. The build converts the OKLCH tokens from [brand/README.md](../design/README.md#palette) to hex.

## Consequences
- **Licence** — MIT, as the Store requires. The whole project is MIT too ([0007](0007-open-source-mit.md)), so code moves freely between the core and the extension.
- **Code** — A `raycast/` folder in the public repo for development. `npm run publish` opens the pull request to raycast/extensions, and Raycast reviews it.
- **Platforms** — `platforms: ["macOS"]` in `package.json` while `pl` is macOS-first.
- **No inline tints** — Raycast renders the prompt as markdown, so values are bold with footnote numbers. The tags carry the source colours.
- **Mockup** — The Raycast tab in `margin-full.html`: the list, the Actions menu and the Fill form.
- **To check** — If a Quicklink can have its own hotkey. Raycast asks for confirmation the first time a deeplink runs a command.
