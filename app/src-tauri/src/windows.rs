//! The three windows: the library, the quick panel and Settings. All three are
//! made at start-up and hidden, never closed, so they open at once.

use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, Rect, WebviewWindow};

pub const LIBRARY: &str = "library";
pub const PANEL: &str = "panel";
pub const SETTINGS: &str = "settings";

fn bring_up(w: &WebviewWindow) {
    let _ = w.unminimize();
    let _ = w.show();
    let _ = w.set_focus();
}

pub fn show_library(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(LIBRARY) {
        bring_up(&w);
    }
}

pub fn show_settings(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(SETTINGS) {
        let _ = w.center();
        bring_up(&w);
        let _ = w.emit("settings-opened", ());
    }
}

/// Shows the panel under the tray icon, or near the top of the screen with
/// the mouse when the shortcut opened it.
pub fn show_panel(app: &AppHandle, tray: Option<Rect>) {
    let Some(w) = app.get_webview_window(PANEL) else {
        return;
    };
    if let Some(pos) = panel_position(app, &w, tray) {
        let _ = w.set_position(pos);
    }
    bring_up(&w);
    let _ = w.emit("panel-opened", ());
}

pub fn toggle_panel(app: &AppHandle, tray: Option<Rect>) {
    let visible = app
        .get_webview_window(PANEL)
        .is_some_and(|w| w.is_visible().unwrap_or(false));
    if visible {
        hide_panel(app);
    } else {
        show_panel(app, tray);
    }
}

pub fn hide_panel(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(PANEL) {
        let _ = w.hide();
    }
}

/// Before a paste: hide the panel, and on macOS hide the app, so the app you
/// came from is in front again.
pub fn hide_for_paste(app: &AppHandle) {
    hide_panel(app);
    #[cfg(target_os = "macos")]
    {
        let _ = app.hide();
    }
}

fn panel_position(
    app: &AppHandle,
    w: &WebviewWindow,
    tray: Option<Rect>,
) -> Option<PhysicalPosition<i32>> {
    let scale = w.scale_factor().unwrap_or(1.0);
    let size = w.outer_size().ok()?;
    let (width, height) = (f64::from(size.width), f64::from(size.height));
    let (anchor_x, anchor_y, tray_height) = match tray {
        Some(rect) => {
            let p = rect.position.to_physical::<f64>(scale);
            let s = rect.size.to_physical::<f64>(scale);
            (p.x + s.width / 2.0, p.y, s.height)
        }
        None => {
            let c = app.cursor_position().ok()?;
            (c.x, c.y, 0.0)
        }
    };
    let monitor = app
        .monitor_from_point(anchor_x, anchor_y)
        .ok()
        .flatten()
        .or_else(|| app.primary_monitor().ok().flatten())?;
    let area = monitor.work_area();
    let (left, top) = (f64::from(area.position.x), f64::from(area.position.y));
    let (right, bottom) = (
        left + f64::from(area.size.width),
        top + f64::from(area.size.height),
    );
    let gap = 6.0 * scale;
    let x = (anchor_x - width / 2.0).clamp(left + gap, (right - width - gap).max(left));
    let y = if tray.is_none() {
        // Opened with the shortcut: a fifth of the way down, like a launcher.
        top + (bottom - top) / 5.0
    } else if anchor_y > (top + bottom) / 2.0 {
        // The tray is at the bottom (Windows): open above it.
        anchor_y - height - gap
    } else {
        anchor_y + tray_height + gap
    };
    let y = y.clamp(top, (bottom - height).max(top));
    Some(PhysicalPosition::new(x.round() as i32, y.round() as i32))
}
