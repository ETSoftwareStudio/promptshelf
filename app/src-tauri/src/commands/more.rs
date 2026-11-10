//! Commands that write (pins, edits, new prompts, sync, first run), Settings
//! and windows.

use promptshelf_core::sync::Report;
use promptshelf_core::{Config, Kind};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use super::{AppState, CmdResult, blocking, msg};
use crate::paste;
use crate::settings::Settings;
use crate::windows;
use crate::writes;

#[tauri::command]
pub async fn pin_prompt(
    state: State<'_, AppState>,
    id: String,
    at: Option<usize>,
) -> CmdResult<Vec<String>> {
    let shelf = state.shelf().map_err(msg)?;
    writes::pin(&shelf, &id, at).map(|p| p.pins).map_err(msg)
}

#[tauri::command]
pub async fn unpin_prompt(state: State<'_, AppState>, id: String) -> CmdResult<Vec<String>> {
    let shelf = state.shelf().map_err(msg)?;
    writes::unpin(&shelf, &id).map(|p| p.pins).map_err(msg)
}

/// Saves the Markdown editor's text. Returns the prompt's full ID afterwards.
#[tauri::command]
pub async fn save_prompt_file(
    state: State<'_, AppState>,
    id: String,
    text: String,
) -> CmdResult<String> {
    let shelf = state.shelf().map_err(msg)?;
    let result = writes::save_file(&shelf, &id, &text).map_err(msg);
    state.invalidate();
    result
}

#[tauri::command]
pub async fn new_prompt(
    state: State<'_, AppState>,
    title: String,
    kind: Option<Kind>,
    library: Option<String>,
) -> CmdResult<String> {
    let shelf = state.shelf().map_err(msg)?;
    let result = writes::new_prompt(
        &shelf,
        &title,
        kind.unwrap_or(Kind::Template),
        library.as_deref(),
    )
    .map_err(msg);
    state.invalidate();
    result
}

#[tauri::command]
pub async fn sync_libraries(
    state: State<'_, AppState>,
    library: Option<String>,
) -> CmdResult<Vec<Report>> {
    let shelf = state.shelf().map_err(msg)?;
    let reports = blocking(move || {
        writes::sync(&shelf, library.as_deref(), &writes::merge_driver()).map_err(msg)
    })
    .await;
    state.invalidate();
    reports
}

/// First run: creates a Personal library, like `shelf init`.
#[tauri::command]
pub async fn init_personal(state: State<'_, AppState>) -> CmdResult<String> {
    let result = blocking(|| writes::init("~/Prompts/personal").map_err(msg)).await;
    state.invalidate();
    result
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    pub settings: Settings,
    pub hotkey_error: Option<String>,
    /// Why Paste can not press the keys here, when it can not.
    pub paste_note: Option<String>,
    pub config_path: String,
    pub settings_path: String,
}

fn settings_view(state: &AppState) -> SettingsView {
    SettingsView {
        settings: state.settings.lock().expect("settings lock").clone(),
        hotkey_error: state.hotkey_error.lock().expect("hotkey lock").clone(),
        paste_note: paste::unsupported_reason(),
        config_path: Config::path().display().to_string(),
        settings_path: Settings::path().display().to_string(),
    }
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> CmdResult<SettingsView> {
    Ok(settings_view(&state))
}

/// Saves new settings. A shortcut that can not be registered is not saved,
/// and the old one stays.
#[tauri::command]
pub async fn set_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> CmdResult<SettingsView> {
    let old = state.settings.lock().expect("settings lock").clone();
    if settings.hotkey != old.hotkey {
        if let Err(e) = crate::apply_hotkey(&app, &settings.hotkey) {
            let _ = crate::apply_hotkey(&app, &old.hotkey);
            return Err(e);
        }
        *state.hotkey_error.lock().expect("hotkey lock") = None;
    }
    settings.save().map_err(msg)?;
    *state.settings.lock().expect("settings lock") = settings;
    Ok(settings_view(&state))
}

/// Opens one of the app's windows: "library", "settings" or "panel".
#[tauri::command]
pub async fn open_window(app: AppHandle, name: String) -> CmdResult<()> {
    match name.as_str() {
        "library" => windows::show_library(&app),
        "settings" => windows::show_settings(&app),
        "panel" => windows::show_panel(&app, None),
        other => return Err(format!("no window named {other}")),
    }
    Ok(())
}

#[tauri::command]
pub async fn hide_panel(app: AppHandle) -> CmdResult<()> {
    if let Some(w) = app.get_webview_window(windows::PANEL) {
        w.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}
