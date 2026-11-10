//! `promptshelf://` links: the deep-link plugin hands them over, `links`
//! turns each into a route, and the library window takes it with
//! `take_link`. The UI routes it; nothing here writes or runs a command.

use promptshelf_core::resolve;
use tauri::{AppHandle, Emitter, Manager, State, Url};
use tauri_plugin_deep_link::DeepLinkExt;

use super::{AppState, CmdResult, blocking, msg};
use crate::links::{self, Link};
use crate::windows;

/// Listens for links, and takes the link that started the app (Windows and
/// Linux pass it as an argument; macOS sends it as an event after start-up).
pub fn listen(app: &AppHandle) {
    let handle = app.clone();
    app.deep_link()
        .on_open_url(move |event| receive(&handle, &event.urls()));
    if let Ok(Some(urls)) = app.deep_link().get_current() {
        receive(app, &urls);
    }
    // An AppImage has no installer to register the scheme, so it does it.
    #[cfg(target_os = "linux")]
    if std::env::var_os("APPIMAGE").is_some()
        && let Err(e) = app.deep_link().register_all()
    {
        eprintln!("PromptShelf: could not register promptshelf:// links: {e}");
    }
}

/// Keeps the last link for the library window, brings the window up and
/// tells it with a "link" event. The window may not listen yet at start-up,
/// so it also asks with `take_link` when it loads.
fn receive(app: &AppHandle, urls: &[Url]) {
    let Some(url) = urls.last() else {
        return;
    };
    let link = links::parse(url.as_str());
    eprintln!("PromptShelf: link {}", link.describe());
    let state = app.state::<AppState>();
    *state.pending_link.lock().expect("link lock") = Some(link);
    windows::show_library(app);
    let _ = app.emit_to(windows::LIBRARY, "link", ());
}

/// The link that waits for the library window, once.
#[tauri::command]
pub async fn take_link(state: State<'_, AppState>) -> CmdResult<Option<Link>> {
    let link = state.pending_link.lock().expect("link lock").take();
    if let Some(link) = &link {
        eprintln!("PromptShelf: the library window took {}", link.describe());
    }
    Ok(link)
}

/// The library window says what it did with a link, for the log.
#[tauri::command]
pub async fn link_handled(outcome: String) -> CmdResult<()> {
    let outcome: String = outcome.chars().take(200).collect();
    eprintln!("PromptShelf: link handled: {outcome}");
    Ok(())
}

/// Copy Link: puts the prompt's `promptshelf://open/…` link on the clipboard
/// and returns it, so the UI can copy it itself when that fails.
#[tauri::command]
pub async fn copy_link(state: State<'_, AppState>, id: String) -> CmdResult<CopiedLink> {
    let shelf = state.shelf().map_err(msg)?;
    let p = shelf.resolve(&id).map_err(msg)?;
    let link = links::open_link(&p.library, &p.id);
    let text = link.clone();
    let copied = blocking(move || Ok(resolve::set_clipboard(&text).is_ok())).await?;
    Ok(CopiedLink { link, copied })
}

#[derive(Debug, serde::Serialize)]
pub struct CopiedLink {
    pub link: String,
    pub copied: bool,
}
