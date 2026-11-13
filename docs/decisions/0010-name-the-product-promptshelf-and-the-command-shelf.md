---
type: decision
status: accepted
date: 2026-11-09
projects: [prompt-library]
tags: [naming, cli, distribution]
---

# 0010. Name the product PromptShelf and the command shelf

## Context
- **Working names** — "Prompt Library" and `pl` were placeholders (open item #54). The owner found "Prompt Library" too general: many App Store apps use it, and nobody can own it in search.
- **Must be fixed before v0.1** — The build plan releases to Homebrew, crates.io and npm at v0.1. A rename after a public release costs more.
- **Owner's choice (2026-11-09)** — "Shelf" as the idea, `shelf` as the command, `promptshelf` as the repo and package name.

## Options
| Option | For | Against |
| --- | --- | --- |
| **Proem, Refrain, Spiel, Cantrip** | Distinctive words, mostly free on crates.io | The owner rejected all of them |
| **Jig, Shelfmark, Carrel, Satchel, Promptfile, pmpt, Plume** | Different styles | The owner picked "shelf" from this list. `shelfmark` is a 4.1k-star GitHub project |
| **Shelf everywhere** | One short word | `shelf` is taken on npm (an abandoned 2022 package) and on crates.io |
| **`shelfmd` as the package** | Free on every registry | Reads as a general Markdown notes tool, like Obsidian, not as a prompt tool |
| **PromptShelf, with the command `shelf`** | Free on npm, crates.io and Homebrew. Says what it is. A short command | Other products used the name (see Consequences) |

## Decision
| Thing | Name |
| --- | --- |
| Product | PromptShelf |
| Command | `shelf`, plus `promptshelf` as a long alias |
| Repo | `ETSoftwareStudio/promptshelf` |
| Crates | `promptshelf-core` (library) and `promptshelf` (the binaries) |
| npm | `promptshelf-cli` (`npx promptshelf-cli`) |
| Homebrew | `promptshelf` |

Each install puts both `shelf` and `promptshelf` on the PATH, the same way `ripgrep` installs `rg`.

### Working names in older docs
Decisions 0001 to 0009 and the report use the working names. Read them with this table.

| Working name | Now |
| --- | --- |
| Prompt Library | PromptShelf |
| `pl …` | `shelf …` |
| `pl-core` | `promptshelf-core` |
| `~/.config/pl/config.toml` | `~/.config/promptshelf/config.toml` |
| `.promptlib/` in a library (pins, team settings) | `.promptshelf/` |
| `promptlib://import#…` | `promptshelf://import#…` |
| Proposal branches `pl/<you>/<id>` | `shelf/<you>/<id>` |
| `~/Library/Application Support/pl/` | `~/Library/Application Support/PromptShelf/` |
| Claude Code skill `prompt-library` | `promptshelf` |


## Consequences
- **Other users of the name (checked 2026-11-09)** — PromptShelf.ai (Talavera Solutions) returns 404. `promptshelf.io` (registered 2026-10-06) returns error 530. `promptshelf.com` (registered 2023) is a parked page. An iOS app "PromptShelf: AI Prompt Library" (July 2026) is live with 1 rating. A Chrome extension called PromptShelf exists. The owner accepted this risk. No trademark filing showed up in a web search; the USPTO database itself was not searched.
- **Command clash** — [maytees/shelf](https://github.com/maytees/shelf) (2 stars, last push 2025-09-02) also installs a `shelf` command. The `promptshelf` alias still works if both are installed.
- **Domain** — No domain yet. `promptshelf.com` may be for sale.
- **npm name** — npm rejected `promptshelf` at v0.1.0, because it is too similar to `prompt-shelf`. The npm package is `promptshelf-cli`. Its folder stays `npm/promptshelf`.
- **npx** — `promptshelf` is a Rust binary. The npm package is a small wrapper that installs the binary for each system, as Biome does. This is part of the v0.1 work.
- **Closes** — Open item #54.
