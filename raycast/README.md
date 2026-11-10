# PromptShelf for Raycast

Search, fill and save the prompts in your PromptShelf libraries from Raycast.

The extension is a thin client of the `shelf` command. It runs `shelf … --json` and reads the result. It never bundles or downloads `shelf`.

## Requirements

- macOS and Raycast.
- The `shelf` command. Install it with one of:
  ```sh
  brew install etsoftwarestudio/tap/promptshelf
  cargo install promptshelf
  ```
- A PromptShelf config. Run `shelf init` once in a terminal.
- Raycast Pro, only for the AI tools.

The extension looks for `shelf` in `/opt/homebrew/bin`, `/usr/local/bin` and `~/.cargo/bin`. To use another path, set **shelf Command** in the extension preferences.

## Commands

| Command                  | What it does                                                                                                                                                                                        |
| ------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Search Prompts           | Lists your prompts, pins first, with a library filter. The detail shows the filled prompt: values in bold with a footnote number, open blanks as a bold question, and each blank as a coloured tag. |
| Fill Prompt              | Fills one prompt by its ID and pastes it. The Fill form opens only when a blank needs you. Quicklinks use this command.                                                                             |
| Save Selection as Prompt | Reads the selected text and saves it as a draft in your Personal library's Inbox.                                                                                                                   |

### Actions (⌘K) in Search Prompts

| Action                                                  | Shortcut |
| ------------------------------------------------------- | -------- |
| Fill and Paste into the front app                       | ↵        |
| Copy Filled Prompt                                      | ⌘↵       |
| Pin / Unpin                                             | ⌘⇧P      |
| Open in Editor                                          | ⌘O       |
| Show in Finder                                          | ⌘⇧F      |
| Create Quicklink (to Fill Prompt with this prompt's ID) | ⌘⇧L      |
| Copy Prompt ID                                          | ⌘⇧C      |
| Reload Prompts                                          | ⌘R       |

In the Fill form, ⌘↵ pastes into the app that was in front before Raycast. ⌘⇧↵ copies.

### Tag colours

| Colour | Meaning                                                              |
| ------ | -------------------------------------------------------------------- |
| Rose   | A value from the conversation                                        |
| Green  | A value from the repo: the git branch or a shell command             |
| Grey   | A value you type ("ask"), a default, or the value you used last time |

The colours come from the PromptShelf palette in OKLCH. `src/lib/colors.ts` converts them to hex, because Raycast tags take hex.

## AI tools (Raycast Pro)

| Tool             | What it does                                                                                 | Confirmation |
| ---------------- | -------------------------------------------------------------------------------------------- | ------------ |
| `search-prompts` | Finds prompts by words.                                                                      | No           |
| `get-prompt`     | Reads one prompt and its blanks.                                                             | No           |
| `fill-prompt`    | Fills the blanks. Open blanks come back in `missing`, so the model fills them from the chat. | No           |
| `save-draft`     | Saves a draft in the Inbox.                                                                  | Yes          |

The AI tools never pass `--user-action`. So `shelf` applies its rules for models: read, fill, and save drafts in the Inbox only. This guards against mistakes; it is not a security boundary.

## Preferences

| Preference        | Default                                                                             |
| ----------------- | ----------------------------------------------------------------------------------- |
| shelf Command     | Empty: look in the usual folders.                                                   |
| Repository Folder | Your home folder. `branch` and shell blanks (such as `git diff --staged`) run here. |

## Develop

Run these commands in this folder:

```sh
npm ci
npm run dev          # opens the extension in Raycast
npm run build        # ray build
npm run typecheck    # tsc; ray build does not check types
npm run lint         # ESLint (Raycast config) and Prettier
npm test             # node:test, with tsx
npm run licences     # fails on a dependency licence that is not permissive
```

The tests read the CLI's JSON snapshots in `crates/promptshelf/tests/snapshots/`. A change to the JSON contract makes them fail.

To also run the client against a real `shelf`:

```sh
cargo build -p promptshelf
PROMPTSHELF_TEST_BIN=$PWD/../target/debug/shelf npm test
```

## Before you publish to the Raycast Store

1. Set `author` in `package.json` to your Raycast username.
2. Run `npm run lint:store`. It runs the full `ray lint`, which checks the author on raycast.com.
3. Run `npm run publish`.

`npm run lint` does not run `ray lint`, because `ray lint` fails on an author that is not a Raycast user, and it has no option to skip that check.
