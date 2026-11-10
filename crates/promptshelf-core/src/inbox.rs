//! The Inbox: updates pulled from team libraries. Text changes apply on pull and
//! show here for information. What a prompt can do is checked, not its text: a
//! new or changed shell command stays off until you allow it (`shelf allow`).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config;
use crate::error::{Error, Result};
use crate::fsutil;
use crate::git;
use crate::prompt::{FOLDERS, Kind, Prompt};
use crate::resolve;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Update {
    pub library: String,
    pub id: String,
    /// The file inside the library.
    pub path: String,
    pub change: Change,
    /// The commit before and after the pull, for "Keep previous version".
    pub from: String,
    pub to: String,
    pub date: String,
    /// Shell commands this update adds or changes. They stay off until allowed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub new_commands: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Change {
    Added,
    Edited,
    Removed,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Store {
    updates: Vec<Update>,
}

fn path() -> PathBuf {
    config::state_dir().join("inbox.json")
}

pub fn load() -> Vec<Update> {
    std::fs::read(path())
        .ok()
        .and_then(|b| serde_json::from_slice::<Store>(&b).ok())
        .map(|s| s.updates)
        .unwrap_or_default()
}

fn save(updates: Vec<Update>) -> Result<()> {
    let bytes =
        serde_json::to_vec_pretty(&Store { updates }).map_err(|e| Error::Invalid(e.to_string()))?;
    fsutil::write_atomic(&path(), &bytes)
}

/// Clears updates: one prompt (`library/id`), or all with `None`.
pub fn clear(full_id: Option<&str>) -> Result<usize> {
    let all = load();
    let before = all.len();
    let kept: Vec<Update> = all
        .into_iter()
        .filter(|u| full_id.is_some_and(|f| format!("{}/{}", u.library, u.id) != f))
        .collect();
    let removed = before - kept.len();
    save(kept)?;
    Ok(removed)
}

fn kind_of(path: &str) -> Option<Kind> {
    let folder = path.split('/').next()?;
    FOLDERS.iter().find(|(f, _)| *f == folder).map(|(_, k)| *k)
}

fn commands_at(root: &Path, rev: &str, path: &str, library: &str) -> BTreeSet<String> {
    let Ok(text) = git::run(root, &["show", &format!("{rev}:{path}")]) else {
        return BTreeSet::new();
    };
    let kind = kind_of(path).unwrap_or(Kind::Template);
    Prompt::parse(&text, Path::new(path), library, kind)
        .map(|p| {
            p.vars
                .values()
                .filter_map(|v| v.from.clone())
                .filter(|f| resolve::is_command(f))
                .collect()
        })
        .unwrap_or_default()
}

/// Records the prompts a pull changed, from commit `from` to `to`.
pub fn record(root: &Path, library: &str, from: &str, to: &str) -> Result<usize> {
    if from == to {
        return Ok(0);
    }
    let diff = git::run(root, &["diff", "--name-status", "--no-renames", from, to])?;
    let mut updates = load();
    let mut added = 0;
    for line in diff.lines() {
        let Some((status, file)) = line.split_once('\t') else {
            continue;
        };
        if !file.ends_with(".md") || kind_of(file).is_none() {
            continue;
        }
        let change = match status {
            "A" => Change::Added,
            "D" => Change::Removed,
            _ => Change::Edited,
        };
        let id = Path::new(file)
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let before = commands_at(root, from, file, library);
        let after = commands_at(root, to, file, library);
        // One entry per prompt: a newer pull replaces an older one, but keeps
        // the oldest "from" so Keep previous version goes back far enough.
        let first_from = updates
            .iter()
            .find(|u| u.library == library && u.path == file)
            .map(|u| u.from.clone())
            .unwrap_or_else(|| from.to_string());
        updates.retain(|u| !(u.library == library && u.path == file));
        updates.push(Update {
            library: library.to_string(),
            id,
            path: file.to_string(),
            change,
            from: first_from,
            to: to.to_string(),
            date: fsutil::today_utc(),
            new_commands: after.difference(&before).cloned().collect(),
        });
        added += 1;
    }
    save(updates)?;
    Ok(added)
}

/// The text of a prompt before an update, for "Keep previous version".
pub fn previous_text(root: &Path, update: &Update) -> Result<String> {
    git::run(root, &["show", &format!("{}:{}", update.from, update.path)])
}

/// "Keep previous version": copies the text an update replaced into Personal.
/// `id` is `library/id` or a bare ID.
pub fn keep_previous(shelf: &crate::Shelf, id: &str) -> Result<crate::write::Copied> {
    let update = load()
        .into_iter()
        .find(|u| format!("{}/{}", u.library, u.id) == id || u.id == id)
        .ok_or_else(|| Error::NotFound(format!("no Inbox update for {id}")))?;
    if update.change == Change::Added {
        return Err(Error::Invalid(format!(
            "{id} is new; it has no previous version"
        )));
    }
    let lib = shelf.library(&update.library)?;
    let text = previous_text(&lib.root, &update)?;
    let source = format!(
        "{}/{}@{}",
        update.library,
        update.id,
        &update.from[..update.from.len().min(7)]
    );
    crate::write::copy_to_personal(shelf, &text, &source, &update.id)
}

/// What an update changed in the file, as a unified diff.
pub fn diff(root: &Path, update: &Update) -> Result<String> {
    git::run(
        root,
        &[
            "diff",
            "--no-color",
            "--no-ext-diff",
            &update.from,
            &update.to,
            "--",
            &update.path,
        ],
    )
}
