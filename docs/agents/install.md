# Install PromptShelf (instructions for an AI agent)

You are an AI coding agent. A person asked you to install PromptShelf on their computer. Follow these steps in order. Run each command yourself unless a step says the person must do it. Do not skip the checks.

PromptShelf is a local prompt library. Prompts are Markdown files in a folder the person owns. The `shelf` command finds and fills them. It has no server, no account and no telemetry.

## Rules
- Tell the person what you will run before you run it.
- Do not use `sudo`.
- Some `shelf` commands refuse to run for an agent. This is on purpose: they create or change the person's library. When a step says "the person runs", ask the person to run it in their own terminal. Do not try to get around the refusal.
- If a step fails, stop and tell the person the exact error. Do not guess a fix that changes their system.

## 1. Check for an existing install
Run:
```sh
shelf --version
```
- If it prints a version, go to step 3.
- If the command is not found, go to step 2.
- If `shelf` is a different program (the output does not say `shelf <version>`), use `promptshelf` in place of `shelf` in every step below.

## 2. Install the command
Try these in order. Stop at the first one that works.

1. Homebrew (macOS and Linux), if `brew --version` works:
   ```sh
   brew install etsoftwarestudio/tap/promptshelf
   ```
2. Cargo, if `cargo --version` works. This builds from source and takes a few minutes:
   ```sh
   cargo install --locked promptshelf
   ```

If none of these tools exist, stop. Tell the person to install one of them (Homebrew on macOS: https://brew.sh), or to download a binary from https://github.com/ETSoftwareStudio/promptshelf/releases.

Check:
```sh
shelf --version
```
It must print `shelf <version>`. If the command is not found, the install folder is not on the PATH. Tell the person which folder it is (for Cargo: `~/.cargo/bin`).

## 3. Create the library (the person runs this)
Run:
```sh
shelf check
```
- If it prints `OK`, a library exists. Go to step 4.
- If it says there is no config, ask the person to run this in their own terminal:
  ```sh
  shelf init
  ```
  It creates `~/Prompts/personal` with a first prompt called "Welcome to PromptShelf". To use another folder, they run `shelf init --path <folder>`.

  If the person uses the PromptShelf desktop app, opening it does the same thing.

Then run `shelf check` again. It must print `OK`.

## 4. Teach yourself the commands
Ask the person to run this in their terminal:
```sh
npx skills add ETSoftwareStudio/promptshelf -g
```
It asks which agents to install the skill for, then links each agent to one copy.

If the person has no Node.js:
- **Claude Code:** ask the person to run `shelf skill install`. It writes `~/.claude/skills/promptshelf/SKILL.md`. If the person tells you to do it for them, write the output of `shelf skill print` to that file.
- **Other agents:** run `shelf skill print`. With the person's OK, add its text to the file your tool reads for instructions, for example `AGENTS.md` or a rules file.

## 5. Optional: apps with no shell
If the person also uses an app that cannot run commands, such as Claude Desktop, follow [mcp.md](mcp.md) to connect it.

## 6. Check that it works
Run:
```sh
shelf list --json
```
It must show at least the `personal/welcome` prompt, unless the person used an existing library.

Then tell the person:
- what you installed and how,
- where their library is (from `shelf libs`),
- that they can say "use my <name> prompt" and you will find and fill it with `shelf`.
