//! Connecting agents, without Tauri: the Claude Code skill, the `shelf`
//! command, and `shelf mcp` in Claude Desktop.
//!
//! The app links the CLI. Under the name `shelf` it runs as the CLI, so the
//! `shelf` command is a link to the app in `~/.local/bin`. A copy of an
//! unsigned app would need its own Gatekeeper approval; the link runs the
//! app the person already opened.

use std::path::{Path, PathBuf};

use promptshelf_core::{Error, Result, agents, tools};

use crate::views::AgentsView;

fn home() -> Result<PathBuf> {
    promptshelf_core::config::home_dir().ok_or_else(|| Error::Invalid("no home folder".into()))
}

/// Why this copy of the app can not be the `shelf` command, if it can not.
pub fn command_note(exe: &Path) -> Option<String> {
    if cfg!(windows) {
        return Some(
            "On Windows, install shelf with cargo; see the PromptShelf README.".into(),
        );
    }
    if exe.to_string_lossy().contains("/AppTranslocation/") {
        return Some("Move PromptShelf to the Applications folder and open it again first.".into());
    }
    None
}

pub fn agents_view() -> Result<AgentsView> {
    let home = home()?;
    let claude = agents::claude_desktop_config();
    let exe = std::env::current_exe().unwrap_or_default();
    Ok(AgentsView {
        skill_path: agents::skill_path(&home).display().to_string(),
        skill_installed: agents::skill_path(&home).is_file(),
        skill_current: agents::skill_current(&home),
        command: tools::find_program("shelf").map(|p| p.display().to_string()),
        command_note: command_note(&exe),
        claude_command: claude.as_deref().and_then(agents::mcp_server_command),
        claude_config: claude.map(|p| p.display().to_string()),
    })
}

pub fn install_skill() -> Result<String> {
    Ok(agents::install_skill(&home()?)?.display().to_string())
}

/// What installing the `shelf` command did.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandInstall {
    pub path: String,
    /// The shell profile that now adds `~/.local/bin` to PATH, if one changed.
    pub profile: Option<String>,
}

/// Links `~/.local/bin/shelf` to this app, which runs as the `shelf` command
/// under that name, and puts `~/.local/bin` on PATH for new terminals.
#[cfg(unix)]
pub fn install_command() -> Result<CommandInstall> {
    let exe = std::env::current_exe().map_err(|e| Error::Invalid(e.to_string()))?;
    if let Some(note) = command_note(&exe) {
        return Err(Error::Invalid(note));
    }
    let home = home()?;
    let dir = agents::command_dir().ok_or_else(|| Error::Invalid("no home folder".into()))?;
    let link = agents::install_command(&exe, &dir)?;
    let shell = std::env::var("SHELL").unwrap_or_else(|_| {
        if cfg!(target_os = "macos") {
            "/bin/zsh".into()
        } else {
            "/bin/bash".into()
        }
    });
    let profile = agents::ensure_on_path(&home, &shell)?;
    Ok(CommandInstall {
        path: link.display().to_string(),
        profile: profile.map(|p| p.display().to_string()),
    })
}

#[cfg(not(unix))]
pub fn install_command() -> Result<CommandInstall> {
    Err(Error::Invalid(
        command_note(Path::new("")).unwrap_or_default(),
    ))
}

/// Adds `shelf mcp` to Claude Desktop, with the full path of the command.
pub fn add_to_claude_desktop() -> Result<agents::McpSetup> {
    let command = tools::find_program("shelf")
        .ok_or_else(|| Error::Invalid("Install the shelf command first.".into()))?;
    let path =
        agents::claude_desktop_config().ok_or_else(|| Error::Invalid("no home folder".into()))?;
    agents::add_mcp_server(&path, &command.display().to_string())
}

/// True when the app runs under the name `shelf`: then it is the CLI.
pub fn is_cli_name(arg0: &std::ffi::OsStr) -> bool {
    Path::new(arg0).file_stem().is_some_and(|n| n == "shelf")
}

#[cfg(test)]
#[path = "tests/agents.rs"]
mod tests;
