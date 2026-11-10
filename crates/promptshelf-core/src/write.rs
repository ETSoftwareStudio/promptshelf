//! Creating files: libraries, drafts and new prompts.

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::fsutil;
use crate::prompt::{FOLDERS, Kind, slugify};
use crate::shelf::Library;

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
fn yaml_string(s: &str) -> String {
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
