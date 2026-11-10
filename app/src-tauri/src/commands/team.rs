//! Commands for changing prompts (delete, rename, move, copy, broken files),
//! team proposals and the Inbox.

use std::path::PathBuf;

use promptshelf_core::write::Copied;
use promptshelf_core::{Error, Kind, Prompt, edit, fill, inbox, state as core_state, write};
use serde::Serialize;
use tauri::{AppHandle, State};

use super::{AppState, CmdResult, blocking, changed, msg};
use crate::team;
use crate::views::{InboxView, ProposalView};

#[tauri::command]
pub async fn delete_prompt(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> CmdResult<()> {
    let shelf = state.shelf().map_err(msg)?;
    let result = edit::delete(&shelf, &id).map(|_| ()).map_err(msg);
    changed(&app, &state);
    result
}

/// Gives a prompt a new title and ID. Returns the new full ID.
#[tauri::command]
pub async fn rename_prompt(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    title: String,
    new_id: String,
) -> CmdResult<String> {
    let shelf = state.shelf().map_err(msg)?;
    let result = edit::rename(&shelf, &id, &title, &new_id).and_then(|r| {
        core_state::rename_prompt(&r.old, &r.new)?;
        Ok(r.new)
    });
    changed(&app, &state);
    result.map_err(msg)
}

/// Moves a draft into a library. A team library gets it as a proposal.
#[tauri::command]
pub async fn move_prompt(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    library: String,
    kind: Kind,
) -> CmdResult<String> {
    let shelf = state.shelf().map_err(msg)?;
    let result = blocking(move || edit::move_to(&shelf, &id, &library, kind).map_err(msg)).await;
    changed(&app, &state);
    result
}

#[tauri::command]
pub async fn copy_prompt(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> CmdResult<Copied> {
    let shelf = state.shelf().map_err(msg)?;
    let result = write::copy_prompt(&shelf, &id).map_err(msg);
    changed(&app, &state);
    result
}

/// A file that could not be read as a prompt, for the raw editor.
fn library_file(state: &AppState, path: &str) -> CmdResult<PathBuf> {
    let shelf = state.shelf().map_err(msg)?;
    let path = PathBuf::from(path);
    if !edit::in_a_library(&shelf, &path) || path.extension().is_none_or(|e| e != "md") {
        return Err(format!("{} is not a prompt file", path.display()));
    }
    Ok(path)
}

#[tauri::command]
pub async fn read_file(state: State<'_, AppState>, path: String) -> CmdResult<String> {
    let path = library_file(&state, &path)?;
    std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))
}

/// Saves a broken file after a fix. Like the editor, it saves only text that
/// reads as a prompt.
#[tauri::command]
pub async fn save_file(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    text: String,
) -> CmdResult<()> {
    let path = library_file(&state, &path)?;
    Prompt::parse(&text, &path, "library", Kind::Template).map_err(|e| match e {
        Error::Frontmatter { message, .. } => format!("The file still has an error: {message}"),
        e => msg(e),
    })?;
    promptshelf_core::fsutil::write_atomic(&path, text.as_bytes()).map_err(msg)?;
    changed(&app, &state);
    Ok(())
}

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
