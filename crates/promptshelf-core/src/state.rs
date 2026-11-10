//! State that stays on this machine: usage counts, last values and allowed
//! commands. It lives outside every library, so a Send never makes a commit.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config::{self, Config};
use crate::error::{Error, Result};
use crate::fsutil;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Usage {
    pub count: u64,
    /// `YYYY-MM-DD`, UTC.
    pub last_used: Option<String>,
    /// The values of the last complete fill, for `from: last`.
    pub values: BTreeMap<String, String>,
}

/// `usage.json` in the state folder, keyed by full prompt ID.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct UsageLog {
    pub prompts: BTreeMap<String, Usage>,
}

impl UsageLog {
    fn path() -> PathBuf {
        config::state_dir().join("usage.json")
    }

    /// Loads the log. A missing or broken file gives an empty log: usage is a
    /// convenience and must never block a fill.
    pub fn load() -> Self {
        std::fs::read(Self::path())
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }

    pub fn get(&self, full_id: &str) -> Option<&Usage> {
        self.prompts.get(full_id)
    }

    /// Counts one use and keeps the values for next time.
    pub fn record(full_id: &str, values: BTreeMap<String, String>) -> Result<()> {
        let mut log = Self::load();
        let entry = log.prompts.entry(full_id.to_string()).or_default();
        entry.count += 1;
        entry.last_used = Some(fsutil::today_utc());
        entry.values = values;
        let bytes = serde_json::to_vec_pretty(&log).map_err(|e| Error::Invalid(e.to_string()))?;
        fsutil::write_atomic(&Self::path(), &bytes)
    }
}

/// Shell commands you allowed for prompts in untrusted libraries, in
/// `allowed.toml` next to the config. Each entry is an exact command string, so
/// an update that changes the command needs a new allow.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Allowed {
    /// Full prompt ID → allowed commands.
    pub commands: BTreeMap<String, Vec<String>>,
}

impl Allowed {
    pub fn path() -> PathBuf {
        Config::dir().join("allowed.toml")
    }

    pub fn load() -> Result<Self> {
        let path = Self::path();
        match std::fs::read_to_string(&path) {
            Ok(text) => toml::from_str(&text).map_err(|e| Error::Config {
                path,
                message: e.message().to_string(),
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(Error::io(path, e)),
        }
    }

    pub fn save(&self) -> Result<()> {
        let text = toml::to_string_pretty(self).map_err(|e| Error::Invalid(e.to_string()))?;
        fsutil::write_atomic(&Self::path(), text.as_bytes())
    }

    pub fn allows(&self, full_id: &str, command: &str) -> bool {
        self.commands
            .get(full_id)
            .is_some_and(|c| c.iter().any(|x| x == command))
    }

    pub fn allow(&mut self, full_id: &str, command: &str) {
        let list = self.commands.entry(full_id.to_string()).or_default();
        if !list.iter().any(|x| x == command) {
            list.push(command.to_string());
        }
    }
}
