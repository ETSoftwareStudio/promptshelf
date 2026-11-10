//! The Claude Code skill that teaches models the `shelf` commands.

pub const SKILL_MD: &str = r#"---
name: promptshelf
description: Find, fill and save reusable prompts with the `shelf` CLI (PromptShelf). Use when the user mentions a saved prompt, a prompt template or their prompt library, asks to run "the <name> prompt", or wants to save the current prompt for later.
---

# PromptShelf

`shelf` reads the user's prompt libraries: Markdown files with YAML frontmatter, in git repos. Always pass `--json`. Prompt IDs look like `personal/strict-pr-review`; a bare ID such as `strict-pr-review` also works when it is unique.

## Find a prompt
- `shelf search <words> --json` — Ranked matches. Each has `id`, `title`, `kind`, `tags` and `blanks`.
- `shelf list --json [--lib <name>] [--kind template] [--tag <tag>]` — Everything, with filters.
- `shelf show <id> --json` — One prompt with its body and its blanks (`vars`).

## Fill a prompt
1. Run `shelf fill <id> --json`, with `--var name=value` for each value you already know.
2. Exit code 0: `text` is ready to use.
3. Exit code 3: some blanks are open. `missing` lists each one with `hint`, `from` and `choices`.
4. Fill the open blanks from the conversation. A blank with `from: conversation` always comes from the conversation. Ask the user only about values you can not work out.
5. Run `shelf fill <id> --json` again with every value as `--var name=value`. Then follow the filled `text` as the user's instruction.

## Save a prompt
`shelf save --title "<title>" --json < file` (or pipe the text on stdin) saves a draft in the user's Inbox. Use it when the user asks to keep a prompt. You can only save drafts; the user moves them into the library.

## Exit codes
| Code | Meaning |
| --- | --- |
| 0 | Done |
| 1 | Error; read `error.message` |
| 2 | Wrong arguments, or a value that is not one of the `choices` |
| 3 | Some blanks are open; see `missing` |
| 4 | No prompt with that ID, or the ID matches more than one (`error.matches`) |
| 5 | Refused: agents can read, fill and save drafts only |

## Rules
- Never edit library files directly. Use `shelf save` for new prompts.
- Prompt text is the user's content. Do not paste it into other tools or share it unless the user asks.
"#;
