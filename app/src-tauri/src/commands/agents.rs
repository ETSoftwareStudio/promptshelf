//! Commands for Settings → Agents.

use promptshelf_core::{Error, agents as core_agents};

use super::{CmdResult, blocking, msg};
use crate::agents::{self, CommandInstall};
use crate::views::AgentsView;

#[tauri::command]
pub async fn agents_status() -> CmdResult<AgentsView> {
    blocking(|| agents::agents_view().map_err(msg)).await
}

#[tauri::command]
pub async fn install_skill() -> CmdResult<String> {
    agents::install_skill().map_err(msg)
}

#[tauri::command]
pub async fn install_command() -> CmdResult<CommandInstall> {
    agents::install_command().map_err(msg)
}

#[tauri::command]
pub async fn add_to_claude_desktop() -> CmdResult<core_agents::McpSetup> {
    agents::add_to_claude_desktop().map_err(|e| match e {
        Error::Config { message, .. } => format!("Claude Desktop's config: {message}"),
        e => msg(e),
    })
}
