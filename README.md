# PromptShelf

A local-first library to save, template, reuse and share prompts. Prompts are Markdown files in a folder you own, synced with git. The `shelf` command finds and fills them, for you and for coding agents.

**Status:** early development. The CLI and the terminal app work; the desktop app is in progress.

## Planned
- **`shelf` CLI** — Search, show and fill prompts. Every command has `--json` output for agents.
- **Terminal app** — Run `shelf` with no arguments.
- **Desktop app** — macOS, Windows and Linux, with a tray panel for quick access.
- **Sync and sharing** — Plain git. No server, no account, no telemetry.

## Install
Not released yet. When it is:

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

## Licence
MIT. See [LICENSE](LICENSE).
