# PromptShelf

A local-first library to save, template, reuse and share prompts. Prompts are Markdown files in a folder you own, synced with git. The `shelf` command finds and fills them, for you and for coding agents.

**Status:** not released yet. The CLI, the terminal app, the desktop app, the MCP server and the Raycast extension all work from source.

## What it has
- **`shelf` CLI** — Search, show and fill prompts. Every command has `--json` output for agents.
- **Terminal app** — Run `shelf` with no arguments.
- **Desktop app** — macOS, Windows and Linux, with a tray panel and a global shortcut. Everything works from the app: set up, add team libraries, sync, propose changes and connect agents, with no terminal.
- **Agents** — A Claude Code skill, and `shelf mcp` for apps with no shell.
- **Sync and sharing** — Plain git. No server, no account, no telemetry.

## Set up with an AI agent
Copy one of these prompts into your coding agent (Claude Code, Codex, Cursor and others). The agent follows the linked steps. It asks you before the steps that need you, such as creating your library.

Install PromptShelf:
```text
Install PromptShelf for me. Follow the steps in
https://raw.githubusercontent.com/ETSoftwareStudio/promptshelf/main/docs/agents/install.md
```

Connect it to an app over MCP (Claude Desktop, Cursor, VS Code and others):
```text
Connect PromptShelf to my apps over MCP. Follow the steps in
https://raw.githubusercontent.com/ETSoftwareStudio/promptshelf/main/docs/agents/mcp.md
```

Agents that read [`llms.txt`](llms.txt) get a short summary and links to both.

## Install
Not released yet. Until then, build it from source:

```sh
cargo install --locked --git https://github.com/ETSoftwareStudio/promptshelf promptshelf
```

After the first release:

<!-- x-release-please-start-version -->
```sh
brew install etsoftwarestudio/tap/promptshelf   # macOS and Linux
cargo install promptshelf --version 0.0.0       # any system with Rust
npx promptshelf@0.0.0 --help                    # try it without installing
```
<!-- x-release-please-end -->

Each install puts two commands on your PATH: `shelf`, and `promptshelf` for systems where another tool owns `shelf`.

## Use
```sh
shelf init                          # create ~/Prompts/personal
shelf new "Strict PR review"        # write a prompt in $EDITOR
shelf                               # the terminal app: search, fill, copy
shelf fill strict-pr-review --var focus="error handling"
shelf skill install                 # teach Claude Code the commands
```

Every command takes `--json`. Agents with no terminal can read, fill and save drafts; other writes need a person.

### Apps with no shell (MCP)
`shelf mcp` serves the Model Context Protocol on stdio. Templates show up as prompts, and four tools search, show, fill and save drafts. For Claude Desktop, add this to `claude_desktop_config.json`:

```json
{ "mcpServers": { "promptshelf": { "command": "shelf", "args": ["mcp"] } } }
```

It runs only while the app runs: no daemon and no network port.

Use the full path to `shelf` (`command -v shelf`): apps started from the Dock often do not see your terminal's PATH. [docs/agents/mcp.md](docs/agents/mcp.md) has the settings for Claude Code, Cursor, VS Code and Codex.

## Build
```sh
cargo build
cargo test
```

The workspace has two crates:

| Crate | What it is |
| --- | --- |
| `crates/promptshelf-core` | The library: prompt files, templates, libraries, sync |
| `crates/promptshelf` | The `shelf` command (also installed as `promptshelf`) |

The desktop app is in `app/`, outside the workspace. See [app/README.md](app/README.md).

The Raycast extension is in [`raycast/`](raycast/README.md). It calls `shelf --json`.

## Licence
MIT. See [LICENSE](LICENSE).
