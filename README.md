# PromptShelf

A local-first library to save, template, reuse and share prompts. Prompts are Markdown files in a folder you own, synced with git. The `shelf` command finds and fills them, for you and for coding agents.

**Status:** early development. Nothing works yet.

## Planned
- **`shelf` CLI** — Search, show and fill prompts. Every command has `--json` output for agents.
- **Terminal app** — Run `shelf` with no arguments.
- **Desktop app** — macOS, Windows and Linux, with a tray panel for quick access.
- **Sync and sharing** — Plain git. No server, no account, no telemetry.

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

## Licence
MIT. See [LICENSE](LICENSE).
