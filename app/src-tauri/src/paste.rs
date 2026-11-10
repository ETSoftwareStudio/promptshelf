//! Paste into the front app: copy first, always, then press ⌘V or Ctrl+V for
//! you where the system allows it. Copy never needs a permission; only the
//! key press does (Accessibility on macOS).

use std::process::{Command, Stdio};

/// Why a paste can not happen here, or `None` when it can.
pub fn unsupported_reason() -> Option<String> {
    if cfg!(target_os = "macos") || cfg!(windows) {
        return None;
    }
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        return Some("Wayland does not let apps press keys for you. Paste with Ctrl+V.".into());
    }
    if std::env::var_os("DISPLAY").is_none() {
        return Some("No display to paste into.".into());
    }
    if !on_path("xdotool") {
        return Some("Install xdotool to paste for you, or paste with Ctrl+V.".into());
    }
    None
}

fn on_path(program: &str) -> bool {
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|d| d.join(program).is_file()))
        .unwrap_or(false)
}

/// Presses the paste keys in the front app.
pub fn press_paste() -> Result<(), String> {
    if let Some(reason) = unsupported_reason() {
        return Err(reason);
    }
    let (program, args): (&str, &[&str]) = if cfg!(target_os = "macos") {
        (
            "osascript",
            &[
                "-e",
                "tell application \"System Events\" to keystroke \"v\" using command down",
            ],
        )
    } else if cfg!(windows) {
        (
            "powershell",
            &[
                "-NoProfile",
                "-Command",
                "Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait('^v')",
            ],
        )
    } else {
        ("xdotool", &["key", "--clearmodifiers", "ctrl+v"])
    };
    let out = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("could not run {program}: {e}"))?;
    if out.status.success() {
        return Ok(());
    }
    if cfg!(target_os = "macos") {
        return Err("Copied. To paste for you, allow PromptShelf in System Settings → Privacy & Security → Accessibility.".into());
    }
    let msg = String::from_utf8_lossy(&out.stderr);
    Err(format!(
        "Copied, but the paste failed: {}",
        msg.lines().next().unwrap_or("").trim()
    ))
}
