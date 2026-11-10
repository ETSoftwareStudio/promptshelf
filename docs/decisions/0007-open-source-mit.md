---
type: decision
status: accepted
date: 2026-11-01
projects: [prompt-library]
tags: [licensing, open-source, distribution]
---

# 0007. Open source, MIT

## Context
- **Owner's plan (2026-11-01)** — The project is open source.
- **Fit** — The app has no server, no account and no running cost ([0003](0003-local-first-git-for-sync-and-sharing-no-server.md)). Nothing is hidden behind a service, so open code costs nothing to run. Users can check what the app does with their prompts and transcripts.
- **Studio rules** — A public repo picks an open-source licence, recorded as a decision.
- **Raycast** — The Raycast Store requires MIT for the extension ([0008](0008-a-raycast-extension-on-top-of-pl.md)).
- **History** — The first version of this decision (2026-11-01) chose Apache-2.0. The owner changed it to MIT on 2026-11-03, before any code or outside contribution existed.

## Options
| Option | For | Against |
| --- | --- | --- |
| **MIT** | Shortest and best known. One licence for the whole project, the Raycast extension included. Code moves freely between the core and the extension. | No explicit patent grant. |
| **Apache-2.0** | Explicit patent grant and contribution terms. The studio uses it for OBD Scanner. | Outside contributions can not move into the MIT extension. Not compatible with GPLv2-only code. |
| **MIT OR Apache-2.0** | Users pick. Common in Rust. | Two licence files to explain. |
| **GPL-3.0** | Forks must stay open. | Keeps out some contributors and companies. Not allowed in the Raycast Store. |

## Decision
MIT for everything: the core, the Mac app, the terminal app and the Raycast extension. The `LICENSE` file names ET Software Studio Inc. as the copyright holder.

## Consequences
- **Patent risk** — Accepted. MIT has no explicit patent grant. For a small local tool from one studio, the risk is low.
- **Design docs in the repo** — The decisions, the exploration report and the prototype are in `docs/`, so contributors can read them.
- **Example names** — Mockups and docs use only made-up names. The sample team library is "Acme eng" (`acme/`), since 2026-11-01.
- **CI** — GitHub-hosted runners only, `macos-*` included, after the repo is public.
- **Dependencies** — Permissive licences only (MIT, BSD, Apache-2.0, ISC), so every release can stay MIT. Keep each dependency's licence notice. No GPL code. The core calls the `git` binary, so it links no git library.
- **Distribution** — Release builds signed and notarised with the studio's Developer ID. `pl` and the app through a Homebrew tap first; homebrew-core and the cask repo later, when the project is notable enough.
- **No telemetry** — Already true by design. The README says so.
- **Contributions** — A `CONTRIBUTING.md` and issue templates at repo creation. `CONTRIBUTING.md` says that contributions come in under the MIT licence. No CLA.
