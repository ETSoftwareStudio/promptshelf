//! Creating files: libraries, drafts, new prompts and copies.

use std::path::{Path, PathBuf};

use crate::config::Writes;
use crate::error::{Error, Result};
use crate::fsutil;
use crate::prompt::{FOLDERS, Kind, slugify};
use crate::shelf::{Library, Shelf};

/// Creates the standard folders of a library. Existing files are kept.
pub fn init_library(root: &Path) -> Result<()> {
    for (folder, _) in FOLDERS {
        let dir = root.join(folder);
        std::fs::create_dir_all(&dir).map_err(|e| Error::io(&dir, e))?;
        fsutil::create_new(&dir.join(".gitkeep"), b"")?;
    }
    let meta = root.join(".promptshelf");
    std::fs::create_dir_all(&meta).map_err(|e| Error::io(&meta, e))?;
    Ok(())
}

/// What to create.
#[derive(Debug, Clone)]
pub struct NewPrompt<'a> {
    pub title: &'a str,
    pub kind: Kind,
    pub body: &'a str,
    pub tags: &'a [String],
}

/// Writes a new prompt file into the folder for its kind. The ID comes from the
/// title; if it is taken, a number is added (`-2`, `-3`, …).
pub fn create_prompt(library: &Library, new: &NewPrompt) -> Result<(String, PathBuf)> {
    let base = slugify(new.title);
    let dir = library.root.join(new.kind.folder());
    for n in 1..1000 {
        let id = if n == 1 {
            base.clone()
        } else {
            format!("{base}-{n}")
        };
        if library.get(&id).is_some() {
            continue;
        }
        let path = dir.join(format!("{id}.md"));
        let text = render_file(&id, new);
        if fsutil::create_new(&path, text.as_bytes())? {
            return Ok((id, path));
        }
    }
    Err(Error::Invalid(format!("no free ID for \"{base}\"")))
}

fn is_team(library: &Library) -> bool {
    library.config.writes == Writes::PullRequest
}

/// Creates a prompt file. In a team library it goes into a new proposal.
pub fn create_in(library: &Library, new: &NewPrompt) -> Result<(String, PathBuf)> {
    if !is_team(library) {
        return create_prompt(library, new);
    }
    let base = slugify(new.title);
    let (id, n) = free_id(library, &base);
    let p = crate::proposals::start(library, &id)?;
    let mut in_worktree = library.clone();
    in_worktree.root = p.worktree();
    in_worktree.prompts.clear();
    let title = if n == 1 {
        new.title.to_string()
    } else {
        format!("{} {n}", new.title)
    };
    create_prompt(
        &in_worktree,
        &NewPrompt {
            title: &title,
            ..new.clone()
        },
    )
}

/// `base`, or `base-2`, `base-3`, … when a prompt or proposal has it.
pub(crate) fn free_id(library: &Library, base: &str) -> (String, usize) {
    let taken = |id: &str| {
        library.get(id).is_some()
            || (is_team(library) && crate::proposals::find(library.name(), id).is_some())
    };
    let mut n = 1;
    let mut id = base.to_string();
    while taken(&id) {
        n += 1;
        id = format!("{base}-{n}");
    }
    (id, n)
}

/// A copy in Personal.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Copied {
    /// The copy's full ID.
    pub id: String,
    pub path: PathBuf,
    pub forked_from: String,
}

/// Writes `text` into Personal's templates as a copy of `source`, with
/// `forked_from`.
pub fn copy_to_personal(shelf: &Shelf, text: &str, source: &str, id: &str) -> Result<Copied> {
    let personal = shelf
        .personal()
        .ok_or_else(|| Error::Invalid("no Personal library; run `shelf init`".into()))?;
    let mut new_id = id.to_string();
    let mut n = 1;
    while personal.get(&new_id).is_some() {
        n += 1;
        new_id = format!("{id}-{n}");
    }
    let (yaml, body) = crate::prompt::split_frontmatter(text);
    let mut fm = String::new();
    for line in yaml.unwrap_or("").lines() {
        if !line.starts_with("id:") && !line.starts_with("forked_from:") {
            fm.push_str(line);
            fm.push('\n');
        }
    }
    let out = format!("---\nid: {new_id}\nforked_from: {source}\n{fm}---\n{body}");
    let path = personal.root.join("templates").join(format!("{new_id}.md"));
    if !fsutil::create_new(&path, out.as_bytes())? {
        return Err(Error::Invalid(format!("{} exists", path.display())));
    }
    Ok(Copied {
        id: format!("{}/{new_id}", personal.name()),
        path,
        forked_from: source.to_string(),
    })
}

/// Copies a prompt into Personal, to change it for yourself.
pub fn copy_prompt(shelf: &Shelf, full_id: &str) -> Result<Copied> {
    let p = shelf.resolve(full_id)?;
    let text = std::fs::read_to_string(&p.path).map_err(|e| Error::io(&p.path, e))?;
    copy_to_personal(shelf, &text, &p.full_id(), &p.id)
}

fn render_file(id: &str, new: &NewPrompt) -> String {
    let mut s = String::from("---\n");
    s.push_str(&format!("id: {id}\n"));
    s.push_str(&format!("title: {}\n", yaml_string(new.title)));
    s.push_str(&format!("kind: {}\n", new.kind.as_str()));
    if !new.tags.is_empty() {
        let tags: Vec<String> = new.tags.iter().map(|t| yaml_string(t)).collect();
        s.push_str(&format!("tags: [{}]\n", tags.join(", ")));
    }
    s.push_str(&format!("created: {}\n", fsutil::today_utc()));
    s.push_str("---\n");
    s.push_str(new.body);
    if !new.body.ends_with('\n') {
        s.push('\n');
    }
    s
}

/// Quotes a string for YAML when it needs it.
pub(crate) fn yaml_string(s: &str) -> String {
    let plain = !s.is_empty()
        && s.chars()
            .all(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.' | '/'))
        && !s.starts_with([' ', '-'])
        && !s.ends_with(' ')
        && !matches!(
            s.to_ascii_lowercase().as_str(),
            "true" | "false" | "yes" | "no" | "null" | "on" | "off" | "~"
        )
        && s.parse::<f64>().is_err();
    if plain {
        s.to_string()
    } else {
        serde_json::to_string(s).unwrap_or_else(|_| format!("\"{s}\""))
    }
}

#[cfg(test)]
#[path = "tests/write.rs"]
mod tests;
