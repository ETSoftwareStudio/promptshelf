# PromptShelf desktop app

The desktop app for macOS, Windows and Linux: a library window, a quick panel behind a tray icon and a global shortcut, and Settings. It is built with Tauri 2 and Svelte 5. The Rust side (`src-tauri`) links `promptshelf-core`, so it reads the same libraries, pins and usage as the `shelf` command.

## What it does

- **Library window** — Three columns: the sidebar (pins with ⌘1 to ⌘9, prompts, kinds, libraries, sync), the list (search as you type) and the detail. The detail shows the prompt as a sentence. Each value is tinted by its source and has a numbered footnote that says where it came from. The footnotes of open blanks are the fill form.
- **New prompt** — **+** in the list or ⌘N (Ctrl+N on Windows and Linux) adds an "Untitled" row at the top of the list and a form in the detail: title, kind, library and the text. The library is the one in view in the sidebar, else Personal. Nothing is written until Save (⌘S). Esc or Discard drops the new prompt; with text typed, the app asks first. It also asks when you pick another prompt or another view.
- **Edit** — The pencil button opens the whole Markdown file. ⌘S saves it only when it still parses as a prompt.
- **Send** — ⌘↩ opens a preview of the message as it will arrive. Copy puts it on the clipboard. Open blanks stay as `{{name}}`, with the instruction for the model to fill them from the chat.
- **Quick panel** — Click the tray icon or press the shortcut (⌘⇧Space on a Mac, Ctrl+Shift+Space elsewhere). Pins come first, then search. Return fills a prompt, Return again previews it, and the preview offers Copy and Paste.
- **Paste** — Paste always copies first. Then it presses ⌘V or Ctrl+V in the app you came from. On macOS that needs the Accessibility permission, which macOS asks for once. On Linux it needs X11 and `xdotool`; on Wayland the panel only copies. Copy never needs a permission.
- **First run** — Choose the folder for the Personal library (the default is `~/Prompts/personal`), use a folder that has prompts, or clone a git repo into an empty folder.
- **Libraries** — In Settings → Libraries: add a library from a git URL or a folder, choose how changes go in (commit, or pull request for team libraries) and trust, and remove a library (its files stay). Each library shows its folder, remote, sync state and last error in plain words.
- **Remotes** — Connect a library that has no remote: paste the address of an empty repo, or click Create Private Repo, which uses GitHub CLI (`gh repo create --private`) when it is installed and signed in. Without git, without `gh`, or without a git name and email, the app says what to do.
- **Sync** — The sidebar syncs every library. When a file changed here and on the remote, the sync stops and keeps your version. Sync needs a look then lists the files and offers Keep both, Keep this computer's version or Use the remote's version. Nothing is force-pushed.
- **Teams** — In a team library, Edit makes a proposal on this computer; the main folder keeps the team's version. The detail shows the proposal's state with Send Proposal or Update Pull Request. Your Proposals lists them all, with Refresh; the More menu withdraws one.
- **Inbox** — Drafts that models saved: Move to Library or Delete. Team updates: the diff, Allow for a new or changed command (it stays off until you click), Keep Previous Version, Copy to Personal and Done.
- **Links** — `promptshelf://` links open the library window. See [Links](#links). Copy Link in the More menu copies a prompt's link.
- **Prompts** — The More menu renames (title and ID; pins and includes follow), deletes and copies to Personal. Files to Fix lists files that can not be read, says what is wrong, and opens them in an editor.
- **Agents** — In Settings → Agents: install the `shelf` command and the Claude Code skill, and add `shelf mcp` to Claude Desktop. The Claude Desktop config keeps its other entries, and the old file is saved as `claude_desktop_config.json.bak`.
- **About** — Settings → About shows the version and opens the GitHub repo, the release notes, a new issue, the licence and the third-party licences. About PromptShelf in the app menu opens it; the Help menu also opens the repo and a new issue.
- **Star card** — After three new prompts or fills on five days, and at least three days after the first start, a small card at the bottom of the sidebar asks for a star on GitHub. Not Now asks once more 30 days later; Star on GitHub and Don't Ask Again end it. The counts stay in `desktop-tips.json` in the state folder. Nothing is sent anywhere. `/#star` shows the card in the browser mock.
- **Settings** — The panel shortcut and the folder for `branch` and command blanks are in `desktop.toml` next to `config.toml`.

Closing a window hides it. The app keeps running in the tray; quit it from the tray menu or with ⌘Q.

## Links

Every link opens the library window and brings it to the front.

| Link                                | What it does                                              |
| ----------------------------------- | --------------------------------------------------------- |
| `promptshelf://open/<library>/<id>` | Selects the prompt.                                       |
| `promptshelf://fill/<library>/<id>` | Selects the prompt and opens the Send preview, like ⌘↩.   |
| `promptshelf://search?q=<text>`     | Shows All Prompts with that search.                       |
| `promptshelf://new?title=<text>`    | Starts a new prompt with that title. `title` is optional. |
| `promptshelf://`                    | Only opens the window.                                    |

### Rules for links

Any web page can open a link, so the app treats every link as untrusted.

- **Checks** — Library names and IDs must follow the ID rules: `a-z`, `0-9` and single hyphens. A link is at most 2048 bytes, and a search or title at most 200 characters.
- **No writes** — A link never writes a file, changes the config or syncs. Only the button you click does.
- **No commands** — A prompt that a link opens runs no shell commands (`from: "git …"`), also in a trusted library. Those blanks wait, and Run Commands runs them. The clipboard and the other named sources still fill.
- **Open sheets stay** — While a sheet, the editor or a new prompt is open, a link only shows a message.
- **Other links** — A bad link, a missing prompt or another route (`import` is kept for sharing) shows a message such as "PromptShelf can't open this link: no prompt personal/foo". Nothing else happens.

On Windows and Linux, a link sent to a running app goes to that app; it does not start a second copy. The installers register the scheme. An AppImage registers it when it starts. A development build (`vp run tauri dev`) does not register it.

The app writes each link and what it did to stderr, for example `PromptShelf: link handled: selected personal/welcome`. Search and title text are not written.

To try links in the browser mock, add `#link-open`, `#link-fill`, `#link-search`, `#link-new`, `#link-missing` or `#link-bad` to the URL. Or run `mockLink({ route: "fill", library: "personal", id: "strict-pr-review" })` in the browser console.

## The `shelf` command

The app links the CLI. When it runs under the name `shelf`, it is the CLI. Settings → Agents → Install makes `~/.local/bin/shelf` a link to the app, and adds `~/.local/bin` to `PATH` in your shell profile (`~/.zprofile` for zsh) when no profile names it.

A link, not a copy, because the app is not signed yet: macOS checks a copied program again, and the link runs the app you already opened. Move the app to Applications first; macOS runs an app from Downloads in a temporary folder, and the app refuses to link to that. On Windows, install `shelf` with cargo or npm.

## Source colours

| Colour         | Meaning                                                                                                        |
| -------------- | -------------------------------------------------------------------------------------------------------------- |
| Rose           | From the conversation. In the app these blanks stay open for the model, so they show as a rose dashed outline. |
| Green          | From the repo or a command (`branch`, `from: "git …"`).                                                        |
| Blue           | A value you gave, a default, or the value from last time.                                                      |
| Dashed outline | A blank nobody filled yet.                                                                                     |

## Develop

You need Rust, [Vite+](https://viteplus.dev) (`vp`) and, on Linux, the Tauri system packages: `libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libxdo-dev` and `libssl-dev`.

1. Install the packages: `vp install`.
2. Run the UI alone in a browser, with made-up prompts: `vp dev`, then open http://localhost:5173 (`panel.html` and `settings.html` are the other windows). Words in the URL hash show other cases, for example `/#firstrun`, `/#conflict`, `/settings.html#libraries,pusherror` or `#nogit`; `src/lib/mock-state.ts` lists them.
3. Run the real app: `vp run tauri dev`.
4. Before a pull request, run `vp check`, `vp run check`, `vp test` and `vp build`. In `src-tauri`, run `cargo clippy --all-targets -- -D warnings` and `cargo test`.

To try the app without your own libraries, set `PROMPTSHELF_CONFIG_DIR` and `PROMPTSHELF_STATE_DIR` to empty folders. The app then starts at first run.

## Layout

| Path                                       | What it is                                                       |
| ------------------------------------------ | ---------------------------------------------------------------- |
| `src-tauri/src/service.rs`, `writes.rs`    | What each command does, without Tauri, with unit tests           |
| `src-tauri/src/manage.rs`, `team.rs`       | Libraries and remotes; proposals and the Inbox                   |
| `src-tauri/src/agents.rs`                  | The skill, the `shelf` link and Claude Desktop                   |
| `src-tauri/src/commands/`                  | The Tauri commands                                               |
| `src-tauri/src/windows.rs`, `menu.rs`      | Windows, the menu bar, the tray icon                             |
| `src-tauri/src/links.rs`                   | The `promptshelf://` link parser                                 |
| `src/lib/`                                 | Typed calls, footnote numbering, the fill state, shortcuts       |
| `src/components/`                          | The sidebar, list, detail, sentence and sheets                   |
| `src/library`, `src/panel`, `src/settings` | One page per window                                              |
| `src/fonts/source-serif-4`                 | The serif for prompt text (SIL Open Font License, see `OFL.txt`) |

## The prompt serif

The serif is not final. To try another one, put its folder next to `src/fonts/source-serif-4` and change the two lines in `src/styles/prompt-font.css`.

## Licences

The app ships one npm package, `@tauri-apps/api` (Apache-2.0 or MIT). The folder picker is `tauri-plugin-dialog` (Apache-2.0 or MIT), and links use `tauri-plugin-deep-link` (Apache-2.0 or MIT). Both are called from Rust only. Settings → About → Third-party Licences lists every crate and npm package the app ships, with its licence. `scripts/third-party.mjs` makes the list during `vp build` and `vp dev`, from `src-tauri/Cargo.lock` and `node_modules`, so it follows every dependency change; nobody edits it by hand. Run `node scripts/third-party.mjs` to print it. The build fails when the list can not be made. CI checks the npm packages with `scripts/licences.mjs` and the crates with `cargo deny` and `src-tauri/deny.toml`. That file allows five MPL-2.0 crates by name, because Tauri 2 itself depends on them. It also allows `tiny-keccak` (CC0-1.0, public domain), which the deep-link plugin uses at build time on Linux.
