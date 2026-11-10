# Connect PromptShelf over MCP (instructions for an AI agent)

You are an AI agent. A person asked you to connect PromptShelf to an app through the Model Context Protocol (MCP). Follow these steps in order.

`shelf mcp` is an MCP server on stdio. The app starts it when it needs it and stops it when it quits. It has no daemon and no network port. It gives the app:
- every template as an MCP prompt,
- four tools: `search_prompts`, `get_prompt`, `fill_prompt` and `save_draft`.

Agents can read, fill and save drafts to the Inbox. They cannot change other prompts.

## Rules
- Tell the person which file you will change before you change it.
- Before you edit a config file, copy it to `<file>.bak`.
- Keep every other entry in the file. Add or replace only the `promptshelf` entry.
- If a step fails, stop and tell the person the exact error.

## 1. Check that `shelf` works
Run:
```sh
shelf --version
shelf check
```
- If `shelf` is not found, install it first: follow [install.md](install.md).
- If `shelf check` says there is no config, the person must create a library first (step 3 of [install.md](install.md)).

## 2. Find the full path to `shelf`
Apps started from the Dock or the Start menu often do not see the terminal's PATH. Use the full path in the config.

- macOS and Linux: `command -v shelf`
- Windows (PowerShell): `(Get-Command shelf).Source`

Below, `<shelf>` means this full path.

## 3. Add the server to the app
Ask the person which app to connect if you do not know. Then use the matching entry.

### Claude Desktop
File:
- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Windows: `%APPDATA%\Claude\claude_desktop_config.json`

Add this under `mcpServers`. Create the file with `{}` if it does not exist.
```json
{ "mcpServers": { "promptshelf": { "command": "<shelf>", "args": ["mcp"] } } }
```
If the person has the PromptShelf desktop app, Settings → Agents → Add to Claude Desktop does the same, with a backup.

### Claude Code
Claude Code can run `shelf` directly, so the skill (step 4 of [install.md](install.md)) is usually enough. To add MCP as well:
```sh
claude mcp add --scope user promptshelf -- <shelf> mcp
```

### Cursor
File: `~/.cursor/mcp.json`
```json
{ "mcpServers": { "promptshelf": { "command": "<shelf>", "args": ["mcp"] } } }
```

### VS Code
File: `.vscode/mcp.json` in the workspace, or the user `mcp.json` (Command Palette → "MCP: Open User Configuration").
```json
{ "servers": { "promptshelf": { "type": "stdio", "command": "<shelf>", "args": ["mcp"] } } }
```

### Codex
File: `~/.codex/config.toml`
```toml
[mcp_servers.promptshelf]
command = "<shelf>"
args = ["mcp"]
```

### Another app
Most apps take the same three values: the name `promptshelf`, the command `<shelf>`, and the arguments `["mcp"]` over stdio. Find the app's MCP settings in its own documentation.

## 4. Check that it works
1. Check the server by itself. This sends one request and must print a JSON reply that lists the tools:
   ```sh
   printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"check","version":"0"}}}' '{"jsonrpc":"2.0","method":"notifications/initialized"}' '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | <shelf> mcp
   ```
2. Ask the person to quit the app completely and open it again. Most apps read MCP settings only at start.
3. Ask the person to check that PromptShelf shows in the app's MCP or tools list. Then they can try: "Find my welcome prompt and fill it."

Then tell the person which file you changed, where the backup is, and how to remove the entry if they want to.
