//! `config.toml`: the list of libraries and the rules for agents.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Overrides the config folder. Tests and portable setups use it.
pub const CONFIG_DIR_ENV: &str = "PROMPTSHELF_CONFIG_DIR";
/// Overrides the folder for per-machine state (usage counts, last values).
pub const STATE_DIR_ENV: &str = "PROMPTSHELF_STATE_DIR";

/// How your changes go into a library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Writes {
    /// Commit straight to the library's branch.
    #[default]
    Commit,
    /// Each change becomes a proposal (a pull request).
    PullRequest,
}

/// One `[[library]]` entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LibraryConfig {
    pub name: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote: Option<String>,
    #[serde(default)]
    pub writes: Writes,
    /// The library for pins, drafts and new prompts. Defaults to the library named
    /// "personal", else the first library.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub personal: bool,
    /// Run shell resolvers (`from: "git …"`) without asking. True for your own
    /// libraries; false for libraries other people write.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trusted: Option<bool>,
    /// The git host type when the core can not tell it from the remote URL:
    /// "github", "gitlab", "gitea" or "forgejo".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forge: Option<String>,
    /// The host's API base URL, for self-hosted forges.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api: Option<String>,
}

impl LibraryConfig {
    /// The library folder, with `~` expanded.
    pub fn root(&self) -> PathBuf {
        expand_tilde(&self.path)
    }

    /// True when shell resolvers in this library can run without an allow step.
    pub fn is_trusted(&self) -> bool {
        self.trusted.unwrap_or(self.writes == Writes::Commit)
    }
}

/// What a caller with no terminal (a model) can do.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AgentRules {
    /// Let agents write outside the Inbox: pins, new prompts, sync pushes.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub allow_writes: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default, rename = "library")]
    pub libraries: Vec<LibraryConfig>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub agents: AgentRules,
}

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    *value == T::default()
}

impl Config {
    /// The folder that holds `config.toml`.
    pub fn dir() -> PathBuf {
        if let Some(dir) = std::env::var_os(CONFIG_DIR_ENV) {
            return PathBuf::from(dir);
        }
        if cfg!(windows) {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("promptshelf")
        } else {
            // ~/.config on macOS too, so dotfile repos can carry it.
            std::env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .filter(|p| p.is_absolute())
                .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
                .unwrap_or_else(|| PathBuf::from("."))
                .join("promptshelf")
        }
    }

    pub fn path() -> PathBuf {
        Self::dir().join("config.toml")
    }

    /// Loads `config.toml`. Returns [`Error::NoConfig`] when it does not exist.
    pub fn load() -> Result<Self> {
        Self::load_from(&Self::path())
    }

    pub fn load_from(path: &Path) -> Result<Self> {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Err(Error::NoConfig),
            Err(e) => return Err(Error::io(path, e)),
        };
        let config: Config = toml::from_str(&text).map_err(|e| Error::Config {
            path: path.to_path_buf(),
            message: e.message().to_string(),
        })?;
        config.validate(path)?;
        Ok(config)
    }

    fn validate(&self, path: &Path) -> Result<()> {
        let mut seen = std::collections::BTreeSet::new();
        for lib in &self.libraries {
            if !crate::prompt::is_valid_id(&lib.name) {
                return Err(Error::Config {
                    path: path.to_path_buf(),
                    message: format!("library name \"{}\" must use only a-z, 0-9 and -", lib.name),
                });
            }
            if !seen.insert(lib.name.as_str()) {
                return Err(Error::Config {
                    path: path.to_path_buf(),
                    message: format!("two libraries are named \"{}\"", lib.name),
                });
            }
        }
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        self.save_to(&Self::path())
    }

    pub fn save_to(&self, path: &Path) -> Result<()> {
        let text = toml::to_string_pretty(self).map_err(|e| Error::Config {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
        }
        crate::fsutil::write_atomic(path, text.as_bytes())
    }

    /// The Personal library: the one marked `personal = true`, else the one named
    /// "personal", else the first.
    pub fn personal(&self) -> Option<&LibraryConfig> {
        self.libraries
            .iter()
            .find(|l| l.personal)
            .or_else(|| self.libraries.iter().find(|l| l.name == "personal"))
            .or_else(|| self.libraries.first())
    }

    pub fn library(&self, name: &str) -> Result<&LibraryConfig> {
        self.libraries
            .iter()
            .find(|l| l.name == name)
            .ok_or_else(|| Error::UnknownLibrary(name.to_string()))
    }
}

/// The folder for state that stays on this machine and never goes into git.
pub fn state_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os(STATE_DIR_ENV) {
        return PathBuf::from(dir);
    }
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("promptshelf")
}

/// Expands a leading `~` or `~/` to the home folder.
pub fn expand_tilde(path: &str) -> PathBuf {
    if path == "~" {
        return dirs::home_dir().unwrap_or_else(|| PathBuf::from(path));
    }
    if let Some(rest) = path.strip_prefix("~/").or_else(|| path.strip_prefix("~\\")) {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

#[cfg(test)]
#[path = "tests/config.rs"]
mod tests;
