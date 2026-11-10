//! The desktop app's own settings, in `desktop.toml` next to `config.toml`.
//! The CLI ignores this file.

use std::path::PathBuf;

use promptshelf_core::{Config, Error, Result};
use serde::{Deserialize, Serialize};

/// The default shortcut for the quick panel. Settings can change it.
pub const DEFAULT_HOTKEY: &str = "CommandOrControl+Shift+Space";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    /// The global shortcut that opens the quick panel, in Tauri's accelerator
    /// format. Empty turns it off.
    pub hotkey: String,
    /// The folder for `branch` and shell resolvers. Empty means the home folder.
    pub work_dir: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            hotkey: DEFAULT_HOTKEY.into(),
            work_dir: String::new(),
        }
    }
}

impl Settings {
    pub fn path() -> PathBuf {
        Config::dir().join("desktop.toml")
    }

    /// Loads the settings. A missing or broken file gives the defaults: the app
    /// must always start.
    pub fn load() -> Self {
        std::fs::read_to_string(Self::path())
            .ok()
            .and_then(|t| toml::from_str(&t).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let text = toml::to_string_pretty(self).map_err(|e| Error::Invalid(e.to_string()))?;
        let path = Self::path();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| Error::io(dir, e))?;
        }
        promptshelf_core::fsutil::write_atomic(&path, text.as_bytes())
    }

    /// The folder resolvers run in.
    pub fn cwd(&self) -> PathBuf {
        let dir = self.work_dir.trim();
        if dir.is_empty() {
            promptshelf_core::config::home_dir().unwrap_or_else(|| PathBuf::from("."))
        } else {
            promptshelf_core::config::expand_tilde(dir)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_use_the_defaults() {
        let s: Settings = toml::from_str("workDir = \"~/code\"\n").unwrap();
        assert_eq!(s.hotkey, DEFAULT_HOTKEY);
        assert!(s.cwd().ends_with("code"));
    }

    #[test]
    fn empty_work_dir_is_home() {
        let s = Settings::default();
        assert_eq!(Some(s.cwd()), promptshelf_core::config::home_dir());
    }
}
