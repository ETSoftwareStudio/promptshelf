//! Connecting agents: the Claude Code skill, `shelf mcp` in Claude Desktop,
//! and the `shelf` command on the PATH.

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{Map, Value, json};

use crate::error::{Error, Result};
use crate::fsutil;

/// The Claude Code skill that teaches models the `shelf` commands.
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

/// Where the skill goes under `base`: your home folder, or a project.
pub fn skill_path(base: &Path) -> PathBuf {
    base.join(".claude")
        .join("skills")
        .join("promptshelf")
        .join("SKILL.md")
}

/// Writes the skill under `base` and returns its file.
pub fn install_skill(base: &Path) -> Result<PathBuf> {
    let path = skill_path(base);
    fsutil::write_atomic(&path, SKILL_MD.as_bytes())?;
    Ok(path)
}

/// True when the skill under `base` is this version's text.
pub fn skill_current(base: &Path) -> bool {
    std::fs::read_to_string(skill_path(base)).is_ok_and(|t| t == SKILL_MD)
}

/// Claude Desktop's config file on this system.
pub fn claude_desktop_config() -> Option<PathBuf> {
    let home = crate::config::home_dir()?;
    Some(
        if cfg!(target_os = "macos") {
            home.join("Library/Application Support/Claude")
        } else if cfg!(windows) {
            PathBuf::from(std::env::var_os("APPDATA")?).join("Claude")
        } else {
            home.join(".config/Claude")
        }
        .join("claude_desktop_config.json"),
    )
}

/// The name of PromptShelf's entry under `mcpServers`.
pub const MCP_SERVER: &str = "promptshelf";

/// What [`add_mcp_server`] did.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct McpSetup {
    pub path: PathBuf,
    /// The copy of the file before the change.
    pub backup: Option<PathBuf>,
    /// False when the entry was already there.
    pub changed: bool,
}

fn read_config(path: &Path) -> Result<Map<String, Value>> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) if t.trim().is_empty() => return Ok(Map::new()),
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Map::new()),
        Err(e) => return Err(Error::io(path, e)),
    };
    match serde_json::from_str(&text) {
        Ok(Value::Object(map)) => Ok(map),
        _ => Err(Error::Config {
            path: path.to_path_buf(),
            message: "it is not a JSON object; fix or remove it, then try again".into(),
        }),
    }
}

/// The command of PromptShelf's entry in the Claude Desktop config, if any.
pub fn mcp_server_command(path: &Path) -> Option<String> {
    let map = read_config(path).ok()?;
    map.get("mcpServers")?
        .get(MCP_SERVER)?
        .get("command")?
        .as_str()
        .map(str::to_string)
}

/// Adds `<command> mcp` to Claude Desktop's config as the "promptshelf"
/// server. Every other entry stays. A changed file is backed up first, to
/// `claude_desktop_config.json.bak`.
pub fn add_mcp_server(path: &Path, command: &str) -> Result<McpSetup> {
    let mut map = read_config(path)?;
    let entry = json!({ "command": command, "args": ["mcp"] });
    let servers = map
        .entry("mcpServers")
        .or_insert_with(|| Value::Object(Map::new()));
    let Value::Object(servers) = servers else {
        return Err(Error::Config {
            path: path.to_path_buf(),
            message: "\"mcpServers\" is not a JSON object".into(),
        });
    };
    if servers.get(MCP_SERVER) == Some(&entry) {
        return Ok(McpSetup {
            path: path.to_path_buf(),
            backup: None,
            changed: false,
        });
    }
    servers.insert(MCP_SERVER.into(), entry);
    let backup = if path.exists() {
        let backup = path.with_extension("json.bak");
        std::fs::copy(path, &backup).map_err(|e| Error::io(&backup, e))?;
        Some(backup)
    } else {
        None
    };
    let mut text = serde_json::to_string_pretty(&Value::Object(map))
        .map_err(|e| Error::Invalid(e.to_string()))?;
    text.push('\n');
    fsutil::write_atomic(path, text.as_bytes())?;
    Ok(McpSetup {
        path: path.to_path_buf(),
        backup,
        changed: true,
    })
}

/// The folder for the `shelf` command: `~/.local/bin`.
pub fn command_dir() -> Option<PathBuf> {
    crate::config::home_dir().map(|h| h.join(".local").join("bin"))
}

/// Links `<dir>/shelf` to `target`, a program that runs as `shelf` under that
/// name. An old link is replaced; a real file is never touched.
#[cfg(unix)]
pub fn install_command(target: &Path, dir: &Path) -> Result<PathBuf> {
    let link = dir.join("shelf");
    std::fs::create_dir_all(dir).map_err(|e| Error::io(dir, e))?;
    match std::fs::symlink_metadata(&link) {
        Ok(m) if m.file_type().is_symlink() => {
            std::fs::remove_file(&link).map_err(|e| Error::io(&link, e))?;
        }
        Ok(_) => {
            return Err(Error::Invalid(format!(
                "{} exists and is not PromptShelf's link; remove it first",
                link.display()
            )));
        }
        Err(_) => {}
    }
    std::os::unix::fs::symlink(target, &link).map_err(|e| Error::io(&link, e))?;
    Ok(link)
}

/// The shell profile that sets `PATH` for `shell` (from `$SHELL`).
pub fn profile_for(home: &Path, shell: &str) -> PathBuf {
    match Path::new(shell).file_name().and_then(|n| n.to_str()) {
        Some("zsh") => home.join(".zprofile"),
        Some("bash") if cfg!(target_os = "macos") => home.join(".bash_profile"),
        Some("fish") => home.join(".config/fish/conf.d/promptshelf.fish"),
        _ => home.join(".profile"),
    }
}

/// Adds `~/.local/bin` to `PATH` in the shell profile, unless a profile names
/// it already. Returns the file it changed.
pub fn ensure_on_path(home: &Path, shell: &str) -> Result<Option<PathBuf>> {
    let profile = profile_for(home, shell);
    let fish = profile.extension().is_some_and(|e| e == "fish");
    let others = [
        ".zshrc",
        ".bashrc",
        ".bash_profile",
        ".profile",
        ".zprofile",
    ];
    let named = std::iter::once(profile.clone())
        .chain(others.iter().map(|f| home.join(f)))
        .any(|f| std::fs::read_to_string(f).is_ok_and(|t| t.contains(".local/bin")));
    if named {
        return Ok(None);
    }
    let mut text = std::fs::read_to_string(&profile).unwrap_or_default();
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    text.push_str("\n# Added by PromptShelf, for the shelf command.\n");
    text.push_str(if fish {
        "fish_add_path $HOME/.local/bin\n"
    } else {
        "export PATH=\"$HOME/.local/bin:$PATH\"\n"
    });
    fsutil::write_atomic(&profile, text.as_bytes())?;
    Ok(Some(profile))
}

#[cfg(test)]
#[path = "tests/agents.rs"]
mod tests;
