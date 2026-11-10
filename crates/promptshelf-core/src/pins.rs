//! Pins: one ordered list of full prompt IDs in Personal's
//! `.promptshelf/pins.yaml`. Pins are not in prompt files, so you can pin a team
//! prompt you can not edit, and shared prompts do not carry your pins.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::fsutil;
use crate::shelf::Shelf;

pub const MAX_PINS: usize = 9;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Pins {
    pub pins: Vec<String>,
}

impl Pins {
    pub fn path(personal_root: &Path) -> PathBuf {
        personal_root.join(".promptshelf").join("pins.yaml")
    }

    pub fn load(personal_root: &Path) -> Result<Self> {
        Self::load_file(&Self::path(personal_root))
    }

    pub fn load_file(path: &Path) -> Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(text) if text.trim().is_empty() => Ok(Self::default()),
            Ok(text) => serde_saphyr::from_str(&text).map_err(|e| Error::Frontmatter {
                path: path.to_path_buf(),
                message: e.to_string().lines().next().unwrap_or("").to_string(),
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(Error::io(path, e)),
        }
    }

    pub fn save(&self, personal_root: &Path) -> Result<()> {
        self.save_file(&Self::path(personal_root))
    }

    pub fn save_file(&self, path: &Path) -> Result<()> {
        let mut text =
            String::from("# Pinned prompts, in order. Shared by every PromptShelf app.\npins:\n");
        for id in &self.pins {
            text.push_str(&format!("  - {id}\n"));
        }
        if self.pins.is_empty() {
            text = text.replace("pins:\n", "pins: []\n");
        }
        fsutil::write_atomic(path, text.as_bytes())
    }

    /// The 1-based position of a pin, for ⌘1 to ⌘9.
    pub fn position(&self, full_id: &str) -> Option<usize> {
        self.pins.iter().position(|p| p == full_id).map(|i| i + 1)
    }

    /// Pins a prompt at `at` (1-based), or at the end.
    pub fn pin(&mut self, full_id: &str, at: Option<usize>) -> Result<()> {
        self.pins.retain(|p| p != full_id);
        if self.pins.len() >= MAX_PINS {
            return Err(Error::Invalid(format!(
                "you can pin {MAX_PINS} prompts; unpin one first"
            )));
        }
        let index = at
            .map(|n| n.saturating_sub(1).min(self.pins.len()))
            .unwrap_or(self.pins.len());
        self.pins.insert(index, full_id.to_string());
        Ok(())
    }

    pub fn unpin(&mut self, full_id: &str) -> bool {
        let before = self.pins.len();
        self.pins.retain(|p| p != full_id);
        self.pins.len() != before
    }
}

/// Loads the pins of the shelf's Personal library.
pub fn load(shelf: &Shelf) -> Result<Pins> {
    match shelf.personal() {
        Some(p) => Pins::load(&p.root),
        None => Ok(Pins::default()),
    }
}

/// Merges two edits of the pin list against their common base: keeps every pin
/// either side added, drops pins either side removed, and keeps our order with
/// their new pins after it.
pub fn merge(base: &Pins, ours: &Pins, theirs: &Pins) -> Pins {
    let removed = |side: &Pins| -> Vec<String> {
        base.pins
            .iter()
            .filter(|p| !side.pins.contains(p))
            .cloned()
            .collect()
    };
    let gone: Vec<String> = removed(ours).into_iter().chain(removed(theirs)).collect();
    let mut out: Vec<String> = ours
        .pins
        .iter()
        .filter(|p| !gone.contains(p))
        .cloned()
        .collect();
    for p in &theirs.pins {
        if !gone.contains(p) && !out.contains(p) {
            out.push(p.clone());
        }
    }
    Pins { pins: out }
}

/// Lines for a library's `.gitattributes` and the git config that make git use
/// `shelf merge-pins` for the pin list.
pub const GITATTRIBUTES_LINE: &str = ".promptshelf/pins.yaml merge=promptshelf-pins";

#[cfg(test)]
#[path = "tests/pins.rs"]
mod tests;
