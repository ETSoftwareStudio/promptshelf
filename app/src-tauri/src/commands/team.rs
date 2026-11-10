//! Commands for team proposals and the Inbox.

use promptshelf_core::write::Copied;
use promptshelf_core::{fill, inbox};
use serde::Serialize;
use tauri::{AppHandle, State};

use super::{AppState, CmdResult, blocking, changed, msg};
use crate::team;
use crate::views::{InboxView, ProposalView};

#[tauri::command]
pub async fn list_proposals(
    state: State<'_, AppState>,
    refresh: bool,
) -> CmdResult<Vec<ProposalView>> {
    let shelf = state.shelf().map_err(msg)?;
    let out = blocking(move || Ok(team::proposals(&shelf, refresh))).await;
    if refresh {
        state.invalidate();
    }
    out
}

#[derive(Debug, Serialize)]
pub struct ProposalText {
    pub title: String,
    pub body: String,
}

#[tauri::command]
pub async fn proposal_text(state: State<'_, AppState>, id: String) -> CmdResult<ProposalText> {
    let shelf = state.shelf().map_err(msg)?;
    let (title, body) = team::proposal_text(&shelf, &id).map_err(msg)?;
    Ok(ProposalText { title, body })
}

/// Sends a proposal: pushes its branch and opens the pull request the first
/// time, or updates it.
#[tauri::command]
pub async fn propose(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    title: String,
    body: String,
) -> CmdResult<ProposalView> {
    let shelf = state.shelf().map_err(msg)?;
    let result = blocking(move || team::propose(&shelf, &id, &title, &body).map_err(msg)).await;
    changed(&app, &state);
    result
}

#[tauri::command]
pub async fn withdraw_proposal(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> CmdResult<()> {
    let shelf = state.shelf().map_err(msg)?;
    let result = blocking(move || team::withdraw(&shelf, &id).map_err(msg)).await;
    changed(&app, &state);
    result
}

#[tauri::command]
pub async fn get_inbox(state: State<'_, AppState>) -> CmdResult<InboxView> {
    let shelf = state.shelf().map_err(msg)?;
    team::inbox(&shelf).map_err(msg)
}

#[tauri::command]
pub async fn update_diff(state: State<'_, AppState>, id: String) -> CmdResult<String> {
    let shelf = state.shelf().map_err(msg)?;
    team::update_diff(&shelf, &id).map_err(msg)
}

/// Allows a prompt's shell commands. Only the person's click calls this.
#[tauri::command]
pub async fn allow_commands(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> CmdResult<Vec<String>> {
    let shelf = state.shelf().map_err(msg)?;
    let p = shelf.resolve(&id).map_err(msg)?;
    let result = fill::allow(&shelf, p)
        .map(|c| c.into_iter().map(|(_, cmd)| cmd).collect())
        .map_err(msg);
    changed(&app, &state);
    result
}

#[tauri::command]
pub async fn keep_previous(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> CmdResult<Copied> {
    let shelf = state.shelf().map_err(msg)?;
    let result = inbox::keep_previous(&shelf, &id).map_err(msg);
    changed(&app, &state);
    result
}

/// Clears one team update from the Inbox.
#[tauri::command]
pub async fn dismiss_update(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> CmdResult<()> {
    let result = inbox::clear(Some(&id)).map(|_| ()).map_err(msg);
    changed(&app, &state);
    result
}
