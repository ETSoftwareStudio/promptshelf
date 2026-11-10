//! The Tauri commands the UI calls. Each one loads state, calls `service` or
//! `writes` and turns errors into plain messages. Writes, Settings and
//! windows are in `commands/more.rs`.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use promptshelf_core::pins::{self, Pins};
use promptshelf_core::resolve::{self, Options};
use promptshelf_core::state::UsageLog;
use promptshelf_core::template::Value;
use promptshelf_core::{Config, Error, Shelf};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::paste;
use crate::service::{self, Detail, FillView, Overview, Query, Summary};
use crate::settings::Settings;
use crate::windows;

/// State shared by every window.
#[derive(Default)]
pub struct AppState {
    /// The loaded shelf. `None` until first use, and again after a write, so
    /// the next command reads the files again.
    shelf: Mutex<Option<Arc<Shelf>>>,
    pub settings: Mutex<Settings>,
    /// Why the shortcut could not be registered, for Settings.
    pub hotkey_error: Mutex<Option<String>>,
    /// The last sync of each library in this run, for the sidebar and Settings.
    pub last_sync: Mutex<service::LastSyncs>,
}

pub(crate) type CmdResult<T> = Result<T, String>;

/// An error in words for the person: git's failures get plain words first.
pub(crate) fn msg(e: Error) -> String {
    let text = e.to_string();
    match (&e, promptshelf_core::tools::explain(&text)) {
        (Error::Git(_), Some(hint)) => format!("{} ({text})", hint.message),
        _ => text,
    }
}

impl AppState {
    pub fn new(settings: Settings) -> Self {
        AppState {
            settings: Mutex::new(settings),
            ..Default::default()
        }
    }

    pub(crate) fn shelf(&self) -> promptshelf_core::Result<Arc<Shelf>> {
        let mut slot = self.shelf.lock().expect("shelf lock");
        if let Some(shelf) = slot.as_ref() {
            return Ok(shelf.clone());
        }
        let shelf = Arc::new(Shelf::load(Config::load()?));
        *slot = Some(shelf.clone());
        Ok(shelf)
    }

    /// Forgets the loaded shelf, so the next command reads the files again.
    pub fn invalidate(&self) {
        *self.shelf.lock().expect("shelf lock") = None;
    }

    fn options(&self) -> Options {
        Options {
            cwd: self.settings.lock().expect("settings lock").cwd(),
            run_commands: true,
        }
    }
}

fn load_pins(shelf: &Shelf) -> Pins {
    pins::load(shelf).unwrap_or_default()
}

/// After a write: forgets the loaded shelf and tells every window with a
/// "shelf-changed" event, so they read the files again.
pub(crate) fn changed(app: &AppHandle, state: &AppState) {
    state.invalidate();
    let _ = app.emit("shelf-changed", ());
}

/// Runs blocking work (resolvers, git) off the main thread.
pub(crate) async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> CmdResult<T> + Send + 'static,
) -> CmdResult<T> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn overview(state: State<'_, AppState>) -> CmdResult<Overview> {
    match state.shelf() {
        Ok(shelf) => {
            let last = state.last_sync.lock().expect("sync lock").clone();
            Ok(service::overview(&shelf, &load_pins(&shelf), &last))
        }
        Err(Error::NoConfig) => Ok(service::empty_overview()),
        Err(e) => Err(msg(e)),
    }
}

/// Reads the files again on the next command. The UI calls it when a window
/// gets focus, so edits made elsewhere show up.
#[tauri::command]
pub async fn reload(state: State<'_, AppState>) -> CmdResult<()> {
    state.invalidate();
    Ok(())
}

#[tauri::command]
pub async fn list_prompts(state: State<'_, AppState>, query: Query) -> CmdResult<Vec<Summary>> {
    let shelf = state.shelf().map_err(msg)?;
    service::list(&shelf, &load_pins(&shelf), &UsageLog::load(), &query).map_err(msg)
}

#[tauri::command]
pub async fn get_prompt(state: State<'_, AppState>, id: String) -> CmdResult<Detail> {
    let shelf = state.shelf().map_err(msg)?;
    service::detail(&shelf, &load_pins(&shelf), &UsageLog::load(), &id).map_err(msg)
}

/// Fills a prompt with its resolvers. They can run shell commands, so this
/// runs off the main thread.
#[tauri::command]
pub async fn fill_prompt(
    state: State<'_, AppState>,
    id: String,
    given: BTreeMap<String, String>,
) -> CmdResult<FillView> {
    let shelf = state.shelf().map_err(msg)?;
    let opts = state.options();
    blocking(move || service::fill(&shelf, &id, &given, &opts).map_err(msg)).await
}

/// Renders values the UI already has. No resolver runs.
#[tauri::command]
pub async fn render_prompt(
    state: State<'_, AppState>,
    id: String,
    values: BTreeMap<String, Value>,
) -> CmdResult<FillView> {
    let shelf = state.shelf().map_err(msg)?;
    service::render(&shelf, &id, &values).map_err(msg)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendResult {
    /// The text that was sent, so the UI can copy it itself when the system
    /// clipboard tool is missing.
    pub text: String,
    pub copied: bool,
    pub pasted: bool,
    /// What went wrong, in words for the person.
    pub message: Option<String>,
}

/// Send: copies the filled prompt and records the use. With `paste`, it then
/// hides the panel and presses the paste keys in the app that had focus.
#[tauri::command]
pub async fn send_prompt(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    values: BTreeMap<String, Value>,
    paste: bool,
) -> CmdResult<SendResult> {
    let shelf = state.shelf().map_err(msg)?;
    let text = service::prepare_send(&shelf, &id, &values, true).map_err(msg)?;
    let copy = {
        let text = text.clone();
        blocking(move || Ok(resolve::set_clipboard(&text))).await?
    };
    if copy.is_ok() {
        crate::tips::Tips::update(|t, now| t.filled_prompt(now));
    }
    let mut result = SendResult {
        text,
        copied: copy.is_ok(),
        pasted: false,
        message: copy.err(),
    };
    if paste && result.copied {
        windows::hide_for_paste(&app);
        let pasted = blocking(|| {
            // Give the system time to bring the previous app to the front.
            std::thread::sleep(Duration::from_millis(250));
            Ok(paste::press_paste())
        })
        .await?;
        match pasted {
            Ok(()) => result.pasted = true,
            Err(e) => result.message = Some(e),
        }
    }
    Ok(result)
}

mod about;
mod agents;
mod manage;
mod more;
mod team;
pub use about::*;
pub use agents::*;
pub use manage::*;
pub use more::*;
pub use team::*;
