//! Commands for libraries, git and sync conflicts.

use promptshelf_core::config::{Writes, expand_tilde};
use promptshelf_core::sync::{self as core_sync, Resolution};
use promptshelf_core::tools;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use super::more::record_syncs;
use super::{AppState, CmdResult, blocking, changed, msg};
use crate::manage::{self, NewLibrary};
use crate::views::{Environment, LibraryStatus, SyncView};

/// The system's folder picker. `start` is the folder it opens in, or the
/// nearest folder above it that exists.
#[tauri::command]
pub async fn pick_folder(
    app: AppHandle,
    title: String,
    start: Option<String>,
) -> CmdResult<Option<String>> {
    let mut dialog = app.dialog().file().set_title(title);
    if let Some(start) = start {
        let mut dir = expand_tilde(&start);
        while !dir.is_dir() && dir.pop() {}
        if dir.is_dir() {
            dialog = dialog.set_directory(dir);
        }
    }
    let (tx, rx) = std::sync::mpsc::channel();
    dialog.pick_folder(move |picked| {
        let _ = tx.send(picked);
    });
    let picked = blocking(move || rx.recv().map_err(|e| e.to_string())).await?;
    Ok(picked
        .and_then(|p| p.into_path().ok())
        .map(|p| p.display().to_string()))
}

#[tauri::command]
pub async fn environment() -> CmdResult<Environment> {
    blocking(|| Ok(manage::environment())).await
}

#[tauri::command]
pub async fn set_git_identity(name: String, email: String) -> CmdResult<Environment> {
    blocking(move || {
        tools::set_git_identity(&name, &email).map_err(msg)?;
        Ok(manage::environment())
    })
    .await
}

#[tauri::command]
pub async fn library_status(state: State<'_, AppState>) -> CmdResult<Vec<LibraryStatus>> {
    let shelf = state.shelf().map_err(msg)?;
    let last = state.last_sync.lock().expect("sync lock").clone();
    blocking(move || Ok(manage::library_status(&shelf, &last))).await
}

#[tauri::command]
pub async fn add_library(
    app: AppHandle,
    state: State<'_, AppState>,
    library: NewLibrary,
) -> CmdResult<String> {
    let result = blocking(move || manage::add_library(&library).map_err(msg)).await;
    changed(&app, &state);
    result.map(|l| l.name)
}

#[tauri::command]
pub async fn update_library(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
    writes: Option<Writes>,
    trusted: Option<bool>,
) -> CmdResult<()> {
    let result = manage::update_library(&name, writes, trusted).map_err(msg);
    changed(&app, &state);
    result
}

/// Removes a library from the list. Its folder and files stay.
#[tauri::command]
pub async fn remove_library(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
) -> CmdResult<String> {
    let result = manage::remove_library(&name).map_err(msg);
    state.last_sync.lock().expect("sync lock").remove(&name);
    changed(&app, &state);
    result
}

/// Syncs one library after its remote changed, so a failed push shows now.
async fn sync_one(state: &AppState, name: String) -> CmdResult<SyncView> {
    state.invalidate();
    let shelf = state.shelf().map_err(msg)?;
    let reports = blocking(move || {
        crate::writes::sync(&shelf, Some(&name), &manage::merge_driver()).map_err(msg)
    })
    .await?;
    state.invalidate();
    record_syncs(state, reports)
        .pop()
        .ok_or_else(|| "the sync did not run".into())
}

#[tauri::command]
pub async fn connect_remote(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
    url: String,
) -> CmdResult<SyncView> {
    let lib = name.clone();
    blocking(move || manage::connect_remote(&lib, &url).map_err(msg)).await?;
    let result = sync_one(&state, name).await;
    changed(&app, &state);
    result
}

/// Creates a private GitHub repo with `gh`, connects it and pushes. It runs
/// only when the person clicks the button.
#[tauri::command]
pub async fn create_github_repo(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
    repo: String,
) -> CmdResult<SyncView> {
    let lib = name.clone();
    blocking(move || manage::create_github_repo(&lib, &repo).map_err(msg)).await?;
    let result = sync_one(&state, name).await;
    changed(&app, &state);
    result
}

#[tauri::command]
pub async fn resolve_conflict(
    app: AppHandle,
    state: State<'_, AppState>,
    library: String,
    how: Resolution,
) -> CmdResult<SyncView> {
    let shelf = state.shelf().map_err(msg)?;
    let report = blocking(move || {
        let lib = shelf.library(&library).map_err(msg)?;
        let personal = shelf.personal().is_some_and(|p| p.name() == lib.name());
        let opts = core_sync::Options {
            commit: true,
            push: true,
            merge_driver: &manage::merge_driver(),
        };
        Ok(core_sync::resolve(lib, personal, how, &opts))
    })
    .await?;
    changed(&app, &state);
    record_syncs(&state, vec![report])
        .pop()
        .ok_or_else(|| "nothing was resolved".into())
}
