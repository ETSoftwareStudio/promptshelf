//! The native menu bar and the tray icon with its menu.

use tauri::image::Image;
use tauri::menu::{
    AboutMetadata, Menu, MenuBuilder, MenuEvent, MenuItemBuilder, PredefinedMenuItem,
    SubmenuBuilder,
};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Runtime};

use crate::windows;

/// Menu item IDs. The UI gets the ones it handles as a "menu" event.
pub const NEW_PROMPT: &str = "new-prompt";
pub const SYNC: &str = "sync";
pub const FIND: &str = "find";
const SETTINGS: &str = "settings";
const LIBRARY: &str = "library";
const PANEL: &str = "panel";
const QUIT: &str = "quit";

pub fn app_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let settings = MenuItemBuilder::with_id(SETTINGS, "Settings…")
        .accelerator("CmdOrCtrl+,")
        .build(app)?;
    let about = PredefinedMenuItem::about(
        app,
        Some("About PromptShelf"),
        Some(AboutMetadata {
            name: Some("PromptShelf".into()),
            version: Some(env!("CARGO_PKG_VERSION").into()),
            license: Some("MIT".into()),
            ..Default::default()
        }),
    )?;
    let quit = MenuItemBuilder::with_id(QUIT, "Quit PromptShelf")
        .accelerator("CmdOrCtrl+Q")
        .build(app)?;
    let new_prompt = MenuItemBuilder::with_id(NEW_PROMPT, "New Prompt")
        .accelerator("CmdOrCtrl+N")
        .build(app)?;
    let find = MenuItemBuilder::with_id(FIND, "Find")
        .accelerator("CmdOrCtrl+F")
        .build(app)?;
    let sync = MenuItemBuilder::with_id(SYNC, "Sync Now").build(app)?;

    let mut menu = MenuBuilder::new(app);
    if cfg!(target_os = "macos") {
        let app_sub = SubmenuBuilder::new(app, "PromptShelf")
            .item(&about)
            .separator()
            .item(&settings)
            .separator()
            .services()
            .separator()
            .hide()
            .hide_others()
            .show_all()
            .separator()
            .item(&quit)
            .build()?;
        menu = menu.item(&app_sub);
    }
    let mut file = SubmenuBuilder::new(app, "File")
        .item(&new_prompt)
        .item(&sync)
        .separator()
        .close_window();
    if !cfg!(target_os = "macos") {
        file = file.separator().item(&settings).separator().item(&quit);
    }
    let edit = SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .separator()
        .item(&find)
        .build()?;
    let window = SubmenuBuilder::new(app, "Window")
        .minimize()
        .item(
            &MenuItemBuilder::with_id(LIBRARY, "Library")
                .accelerator("CmdOrCtrl+0")
                .build(app)?,
        )
        .item(&MenuItemBuilder::with_id(PANEL, "Quick Panel").build(app)?)
        .build()?;
    if !cfg!(target_os = "macos") {
        let help = SubmenuBuilder::new(app, "Help").item(&about).build()?;
        return menu
            .item(&file.build()?)
            .item(&edit)
            .item(&window)
            .item(&help)
            .build();
    }
    menu.item(&file.build()?).item(&edit).item(&window).build()
}

/// Handles a click on the menu bar or the tray menu.
pub fn on_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        SETTINGS => windows::show_settings(app),
        LIBRARY => windows::show_library(app),
        PANEL => windows::show_panel(app, None),
        QUIT => app.exit(0),
        id @ (NEW_PROMPT | FIND) => {
            windows::show_library(app);
            let _ = app.emit_to(windows::LIBRARY, "menu", id);
        }
        SYNC => {
            let _ = app.emit_to(windows::LIBRARY, "menu", SYNC);
        }
        _ => {}
    }
}

/// The tray icon. A left click opens the panel; the menu has the rest. Linux
/// trays only show the menu, so the menu opens the panel too.
pub fn tray(app: &AppHandle) -> tauri::Result<()> {
    let menu = MenuBuilder::new(app)
        .item(&MenuItemBuilder::with_id(PANEL, "Quick Panel").build(app)?)
        .item(&MenuItemBuilder::with_id(LIBRARY, "Open Library").build(app)?)
        .item(&MenuItemBuilder::with_id(SYNC, "Sync Now").build(app)?)
        .item(&MenuItemBuilder::with_id(SETTINGS, "Settings…").build(app)?)
        .separator()
        .item(&MenuItemBuilder::with_id(QUIT, "Quit PromptShelf").build(app)?)
        .build()?;
    let icon = Image::from_bytes(include_bytes!("../icons/tray.png"))?;
    TrayIconBuilder::with_id("main")
        .icon(icon)
        .icon_as_template(true)
        .tooltip("PromptShelf")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(on_menu_event)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            {
                windows::toggle_panel(tray.app_handle(), Some(rect));
            }
        })
        .build(app)?;
    Ok(())
}
