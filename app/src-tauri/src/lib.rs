//! The PromptShelf desktop app: a library window, a quick panel behind a tray
//! icon and a global shortcut, and Settings. The Rust side links
//! `promptshelf-core`; the UI is Svelte in `app/src`.

pub mod about;
pub mod agents;
mod commands;
pub mod manage;
mod menu;
mod paste;
pub mod service;
pub mod settings;
pub mod team;
pub mod views;
mod windows;
pub mod writes;

#[cfg(test)]
#[path = "tests/common.rs"]
mod tests_common;

use tauri::{AppHandle, Manager, WindowEvent};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use commands::AppState;
use settings::Settings;

/// Registers the panel shortcut, replacing the old one. Empty turns it off.
pub fn apply_hotkey(app: &AppHandle, hotkey: &str) -> Result<(), String> {
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    let hotkey = hotkey.trim();
    if hotkey.is_empty() {
        return Ok(());
    }
    let shortcut: Shortcut = hotkey
        .parse()
        .map_err(|e| format!("\"{hotkey}\" is not a shortcut: {e}"))?;
    gs.register(shortcut)
        .map_err(|e| format!("could not use {hotkey}: {e}"))
}

pub fn run() {
    let settings = Settings::load();
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            windows::show_library(app);
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        windows::toggle_panel(app, None);
                    }
                })
                .build(),
        )
        .manage(AppState::new(settings.clone()))
        .menu(menu::app_menu)
        .on_menu_event(menu::on_menu_event)
        .setup(move |app| {
            let handle = app.handle().clone();
            menu::tray(&handle)?;
            if let Err(e) = apply_hotkey(&handle, &settings.hotkey) {
                let state = handle.state::<AppState>();
                *state.hotkey_error.lock().expect("hotkey lock") = Some(e);
            }
            Ok(())
        })
        .on_window_event(|window, event| match (window.label(), event) {
            // Closing a window hides it; the app keeps running in the tray.
            (_, WindowEvent::CloseRequested { api, .. }) => {
                api.prevent_close();
                let _ = window.hide();
            }
            (windows::PANEL, WindowEvent::Focused(false)) => {
                let _ = window.hide();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::overview,
            commands::reload,
            commands::list_prompts,
            commands::get_prompt,
            commands::fill_prompt,
            commands::render_prompt,
            commands::send_prompt,
            commands::pin_prompt,
            commands::unpin_prompt,
            commands::save_prompt_file,
            commands::new_prompt,
            commands::sync_libraries,
            commands::init_personal,
            commands::get_settings,
            commands::set_settings,
            commands::open_window,
            commands::hide_panel,
            commands::open_url,
            commands::pick_folder,
            commands::environment,
            commands::set_git_identity,
            commands::library_status,
            commands::add_library,
            commands::update_library,
            commands::remove_library,
            commands::connect_remote,
            commands::create_github_repo,
            commands::resolve_conflict,
            commands::agents_status,
            commands::install_skill,
            commands::install_command,
            commands::add_to_claude_desktop,
            commands::delete_prompt,
            commands::rename_prompt,
            commands::move_prompt,
            commands::copy_prompt,
            commands::read_file,
            commands::save_file,
            commands::list_proposals,
            commands::proposal_text,
            commands::propose,
            commands::withdraw_proposal,
            commands::get_inbox,
            commands::update_diff,
            commands::allow_commands,
            commands::keep_previous,
            commands::dismiss_update,
            commands::about,
        ])
        .build(tauri::generate_context!())
        .expect("could not start PromptShelf");

    app.run(|app, event| {
        // macOS: a click on the Dock icon opens the library again.
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Reopen { .. } = event {
            windows::show_library(app);
        }
        let _ = (app, event);
    });
}
