//! Changing prompts that exist: save, rename, delete and move. In a team
//! library (`writes = "pull-request"`) a save goes into a proposal, never into
//! the main folder.

use std::path::{Path, PathBuf};

use crate::config::Writes;
use crate::error::{Error, Result};
use crate::pins::Pins;
use crate::prompt::{FOLDERS, Kind, Prompt, is_valid_id};
use crate::shelf::{Library, Shelf};
use crate::write::{self, free_id};
use crate::{fsutil, proposals};

pub(crate) fn is_team(lib: &Library) -> bool {
    lib.config.writes == Writes::PullRequest
}

/// The kind the prompt's folder implies, for parsing an edit.
pub fn folder_kind(lib: &Library, p: &Prompt) -> Kind {
    let rel = p
        .path
        .strip_prefix(&lib.root)
        .or_else(|_| {
            p.path
                .strip_prefix(proposals::worktree_dir(lib.name(), &p.id))
        })
        .unwrap_or(&p.path);
    let first = rel
        .components()
        .next()
        .map(|c| c.as_os_str().to_string_lossy().into_owned());
    FOLDERS
        .iter()
        .find(|(folder, _)| first.as_deref() == Some(folder))
        .map(|(_, k)| *k)
        .unwrap_or(p.kind)
}

/// Writes a prompt's whole file. The text must parse as a prompt, so a typo in
/// the frontmatter never lands on disk. In a team library the text goes into
/// your proposal for the prompt. Returns the full ID, which changes when the
/// frontmatter changes the ID.
pub fn save_text(shelf: &Shelf, full_id: &str, text: &str) -> Result<String> {
    let p = shelf.resolve(full_id)?;
    let lib = shelf.library(&p.library)?;
    let parsed = Prompt::parse(text, &p.path, &p.library, folder_kind(lib, p))?;
    if parsed.id != p.id {
        if is_team(lib) {
            return Err(Error::Invalid(
                "the ID of a team prompt can not change in a proposal".into(),
            ));
        }
        if let Some(other) = lib.get(&parsed.id) {
            return Err(Error::Invalid(format!(
                "the ID \"{}\" is taken by {}",
                parsed.id,
                other.path.display()
            )));
        }
    }
    let path = if is_team(lib) && !p.proposal {
        let prop = proposals::start(lib, &p.id)?;
        proposals::file_in_worktree(lib, &prop)
    } else {
        p.path.clone()
    };
    fsutil::write_atomic(&path, text.as_bytes())?;
    Ok(parsed.full_id())
}

fn refuse_team(lib: &Library, what: &str) -> Result<()> {
    if is_team(lib) {
        return Err(Error::Refused(format!(
            "{} is a team library: changes go in as pull requests. {what} a team prompt on the host.",
            lib.name()
        )));
    }
    Ok(())
}

/// Deletes a prompt's file and its pin. Not in team libraries.
pub fn delete(shelf: &Shelf, full_id: &str) -> Result<PathBuf> {
    let p = shelf.resolve(full_id)?;
    refuse_team(shelf.library(&p.library)?, "Delete")?;
    std::fs::remove_file(&p.path).map_err(|e| Error::io(&p.path, e))?;
    replace_pin(shelf, &p.full_id(), None)?;
    Ok(p.path.clone())
}

/// What [`rename`] changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Renamed {
    pub old: String,
    pub new: String,
    pub path: PathBuf,
    /// Other files whose includes now name the new ID.
    pub includes: usize,
}

/// Gives a prompt a new title and, when `new_id` differs, a new ID. A new ID
/// renames the file and updates pins and includes. Not in team libraries.
/// Usage counts and allowed commands are per machine: call
/// [`crate::state::rename_prompt`] too.
pub fn rename(shelf: &Shelf, full_id: &str, title: &str, new_id: &str) -> Result<Renamed> {
    let p = shelf.resolve(full_id)?;
    let lib = shelf.library(&p.library)?;
    refuse_team(lib, "Rename")?;
    let (title, new_id) = (title.trim(), new_id.trim());
    if title.is_empty() {
        return Err(Error::Invalid("the title is empty".into()));
    }
    if !is_valid_id(new_id) {
        return Err(Error::Invalid(format!(
            "\"{new_id}\" is not an ID: use a-z, 0-9 and -"
        )));
    }
    let text = std::fs::read_to_string(&p.path).map_err(|e| Error::io(&p.path, e))?;
    let mut text = set_field(&text, "title", &write::yaml_string(title));
    let old = p.full_id();
    let new = format!("{}/{new_id}", lib.name());
    if new_id == p.id {
        fsutil::write_atomic(&p.path, text.as_bytes())?;
        return Ok(Renamed {
            old,
            new,
            path: p.path.clone(),
            includes: 0,
        });
    }
    if let Some(other) = lib.get(new_id) {
        return Err(Error::Invalid(format!(
            "the ID \"{new_id}\" is taken by {}",
            other.path.display()
        )));
    }
    text = set_field(&text, "id", new_id);
    let path = p.path.with_file_name(format!("{new_id}.md"));
    if !fsutil::create_new(&path, text.as_bytes())? {
        return Err(Error::Invalid(format!("{} exists", path.display())));
    }
    std::fs::remove_file(&p.path).map_err(|e| Error::io(&p.path, e))?;
    replace_pin(shelf, &old, Some(&new))?;
    let includes = rewrite_includes(shelf, lib.name(), &p.id, new_id)?;
    Ok(Renamed {
        old,
        new,
        path,
        includes,
    })
}

/// Points `{{> old}}` includes at the new ID, in libraries you commit to.
fn rewrite_includes(shelf: &Shelf, library: &str, old: &str, new: &str) -> Result<usize> {
    let re = regex::Regex::new(r"\{\{\s*>\s*([a-z0-9/-]+)\s*\}\}").expect("include regex");
    let old_full = format!("{library}/{old}");
    let mut changed = 0;
    for lib in shelf.libraries.iter().filter(|l| !is_team(l)) {
        for p in &lib.prompts {
            let Ok(text) = std::fs::read_to_string(&p.path) else {
                continue;
            };
            let out = re.replace_all(&text, |c: &regex::Captures| {
                let target = &c[1];
                if target == old_full {
                    format!("{{{{> {library}/{new}}}}}")
                } else if target == old && lib.name() == library {
                    format!("{{{{> {new}}}}}")
                } else {
                    c[0].to_string()
                }
            });
            if out != text {
                fsutil::write_atomic(&p.path, out.as_bytes())?;
                changed += 1;
            }
        }
    }
    Ok(changed)
}

/// Moves a prompt, usually a draft, into a library as `kind`. A team library
/// gets it as a new proposal. Its pin follows it. Returns the new full ID.
pub fn move_to(shelf: &Shelf, full_id: &str, library: &str, kind: Kind) -> Result<String> {
    let p = shelf.resolve(full_id)?;
    refuse_team(shelf.library(&p.library)?, "Move")?;
    let target = shelf.library(library)?;
    // In its own library the prompt's ID is free: it moves.
    let id = if target.name() == p.library {
        p.id.clone()
    } else {
        free_id(target, &p.id).0
    };
    let text = std::fs::read_to_string(&p.path).map_err(|e| Error::io(&p.path, e))?;
    let text = set_field(&set_field(&text, "id", &id), "kind", kind.as_str());
    let root = if is_team(target) {
        proposals::start(target, &id)?.worktree()
    } else {
        target.root.clone()
    };
    let path = root.join(kind.folder()).join(format!("{id}.md"));
    if !fsutil::create_new(&path, text.as_bytes())? {
        return Err(Error::Invalid(format!("{} exists", path.display())));
    }
    std::fs::remove_file(&p.path).map_err(|e| Error::io(&p.path, e))?;
    let new = format!("{}/{id}", target.name());
    replace_pin(shelf, &p.full_id(), Some(&new))?;
    Ok(new)
}

/// Replaces (or with `None`, removes) a pin in Personal's pin list.
fn replace_pin(shelf: &Shelf, old: &str, new: Option<&str>) -> Result<()> {
    let Some(personal) = shelf.personal() else {
        return Ok(());
    };
    let mut pins = Pins::load(&personal.root)?;
    let Some(i) = pins.pins.iter().position(|p| p == old) else {
        return Ok(());
    };
    match new {
        Some(new) => pins.pins[i] = new.to_string(),
        None => {
            pins.pins.remove(i);
        }
    }
    pins.save(&personal.root)
}

/// Sets one top-level frontmatter field and keeps every other byte. Adds the
/// frontmatter when the file has none. `value` must be valid YAML.
pub fn set_field(text: &str, key: &str, value: &str) -> String {
    let line = format!("{key}: {value}\n");
    let Some(rest) = text.strip_prefix("---\n") else {
        return format!("---\n{line}---\n{text}");
    };
    let mut out = String::from("---\n");
    let mut found = false;
    let mut lines = rest.split_inclusive('\n');
    for l in lines.by_ref() {
        if l.trim_end() == "---" {
            if !found {
                out.push_str(&line);
            }
            out.push_str(l);
            out.extend(lines);
            return out;
        }
        if l.strip_prefix(key).is_some_and(|r| r.starts_with(':')) {
            out.push_str(&line);
            found = true;
        } else {
            out.push_str(l);
        }
    }
    // No closing line: the file has no frontmatter after all.
    format!("---\n{line}---\n{text}")
}

/// True when `path` is inside one of the shelf's library folders.
pub fn in_a_library(shelf: &Shelf, path: &Path) -> bool {
    shelf.libraries.iter().any(|l| path.starts_with(&l.root))
}

#[cfg(test)]
#[path = "tests/edit.rs"]
mod tests;
