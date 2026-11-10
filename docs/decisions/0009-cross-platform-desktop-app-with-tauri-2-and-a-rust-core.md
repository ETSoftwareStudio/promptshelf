---
type: decision
status: accepted
date: 2026-11-09
projects: [prompt-library]
tags: [architecture, cross-platform, tauri, rust, ui]
---

# 0009. Cross-platform desktop app with Tauri 2 and a Rust core

## Context
- **Owner's request (2026-11-09)** — The app also works on Windows and Linux, not only on macOS.
- **Before** — macos.md planned SwiftUI + TCA. SwiftUI runs only on Apple platforms, so Windows and Linux would each need a second UI.
- **Open item** — The core language (Go or Rust) and how the app reaches the core were still open ([0001](0001-one-core-three-surfaces.md), [0005](0005-several-libraries-and-a-cli-for-models.md)).
- **The prototype is HTML** — [`margin-full.html`](../design/margin-full.html) is HTML and CSS. The Margin features are type and colour: the prompt as a sentence, footnotes, source colours ([0004](0004-margin-as-the-visual-direction-for-the-mac-app.md)).

## Options
| Option | For | Against |
| --- | --- | --- |
| **Tauri 2 + Rust core** | One UI on three systems. The app backend is Rust, so it links the core crate directly. The prototype HTML and CSS carry over. Small binaries. | Web controls, not native ones. Some OS features differ (see Consequences). |
| **SwiftUI now, other systems later** | Best feel on macOS. | Two or three UIs to build and keep the same. |
| **Compose Multiplatform (Kotlin)** | One UI. A language the studio knows. | Ships a JVM runtime. Not native controls either. |
| **Electron** | Mature. | Larger and heavier than Tauri, with no gain for this app. |
| **Go core** (with any UI) | Fast to write. Bubble Tea for the terminal app. | A Tauri app must then call it as a sidecar process. Two languages in the backend. |

## Decision
Tauri 2 for the desktop app on macOS, Windows and Linux. Rust for the core.

- **Core** — A `pl-core` Rust crate owns the files, templates, resolvers, sync and checks, as in [0001](0001-one-core-three-surfaces.md).
- **`pl` binary** — The CLI and the terminal app (ratatui). It uses `pl-core`.
- **Desktop app** — Tauri 2. The Rust side links `pl-core` directly. The UI is TypeScript built with Vite+ (`vp`).
- **External clients** — Raycast and models use `pl --json`. That JSON output and the exit codes stay the stable contract.
- **Menu bar** — A Tauri tray icon with a panel window and a global hotkey. On Windows it is the notification-area icon.

## Consequences
- **Changes [0004](0004-margin-as-the-visual-direction-for-the-mac-app.md)** — "Native controls only" no longer holds. The UI uses web controls that follow each OS's conventions: native menus and window frames from Tauri, and vibrancy on macOS. The rest of Margin stays: the layout, the sentence, the footnotes, the source colours and the preview before Send. Watch for a "web app" look, the reason the first design was rejected.
- **Fonts** — Apple's licence does not let SF Pro and New York ship on Windows or Linux. The UI uses the font of each system. Prompt text uses one bundled serif with an open licence, the same on every system.
- **Colours** — The OKLCH tokens go into CSS custom properties. `oklch()` works in WKWebView, WebView2 and WebKitGTK. This app no longer needs the `oklch-swift` package.
- **Paste and selection** — macOS uses the Accessibility API. Windows uses `SendInput` and UI Automation. Linux X11 works. Linux Wayland blocks synthetic input and reading the selection, so Send falls back to "copy to clipboard".
- **Send targets** — `tmux send-keys` works on macOS and Linux only. A new `claude "…"` session works on each system that has `claude`.
- **Summaries** — Apple's Foundation Models exist only on macOS. Windows and Linux use `claude -p` on the user's plan, or summaries stay off.
- **Linux tray** — Uses AppIndicator. GNOME shows it only with the AppIndicator extension. The global hotkey still opens the panel.
- **Signing and distribution** — macOS: Developer ID, notarised, Homebrew tap. Windows: a code-signing certificate (a yearly cost; SmartScreen warns without one), then winget. Linux: AppImage and `.deb` first. The Mac App Store stays out (macos.md).
- **Repo** — Private at the start, public at v0.1 (owner, 2026-11-09). After it goes public, all CI jobs use GitHub-hosted runners.
- **Licences** — Tauri and ratatui use MIT or Apache-2.0, so [0007](0007-open-source-mit.md) holds. CI checks the licence of each crate and npm package.
- **Closes** — The core language question. The desktop app links the core. External clients call `pl --json`.
- **UI framework (2026-11-09, owner)** — Svelte 5, built with Vite+.
- **Still open** — The bundled serif (Source Serif 4, Newsreader or Literata; the owner is comparing them). The Windows signing option.
