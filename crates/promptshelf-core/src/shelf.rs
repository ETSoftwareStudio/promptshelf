//! The shelf: every attached library, scanned into one list of prompts.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::config::{Config, LibraryConfig};
use crate::error::{Error, Result};
use crate::prompt::{FOLDERS, Kind, Prompt};

/// A problem with one file that does not stop the others from loading.
#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    pub path: PathBuf,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct Library {
    pub config: LibraryConfig,
    pub root: PathBuf,
    pub prompts: Vec<Prompt>,
}

impl Library {
    pub fn name(&self) -> &str {
        &self.config.name
    }

    /// Reads every `.md` file in the standard folders. Bad files become
    /// diagnostics.
    pub fn scan(config: LibraryConfig, diagnostics: &mut Vec<Diagnostic>) -> Self {
        let root = config.root();
        let mut prompts: Vec<Prompt> = Vec::new();
        let mut by_id: BTreeMap<String, PathBuf> = BTreeMap::new();
        for (folder, kind) in FOLDERS {
            let dir = root.join(folder);
            let mut files = Vec::new();
            collect_markdown(&dir, &mut files);
            files.sort();
            for path in files {
                match Prompt::load(&path, &config.name, *kind) {
                    Ok(prompt) => {
                        if let Some(first) = by_id.get(&prompt.id) {
                            diagnostics.push(Diagnostic {
                                path: path.clone(),
                                message: format!(
                                    "duplicate ID \"{}\"; {} already uses it",
                                    prompt.id,
                                    first.display()
                                ),
                            });
                            continue;
                        }
                        by_id.insert(prompt.id.clone(), path.clone());
                        prompts.push(prompt);
                    }
                    Err(e) => diagnostics.push(Diagnostic {
                        path: path.clone(),
                        message: match e {
                            Error::Frontmatter { message, .. } => message,
                            other => other.to_string(),
                        },
                    }),
                }
            }
        }
        if !root.is_dir() {
            diagnostics.push(Diagnostic {
                path: root.clone(),
                message: format!("library \"{}\" folder does not exist", config.name),
            });
        }
        Library {
            config,
            root,
            prompts,
        }
    }

    pub fn get(&self, id: &str) -> Option<&Prompt> {
        self.prompts.iter().find(|p| p.id == id)
    }
}

fn collect_markdown(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            collect_markdown(&path, out);
        } else if path.extension().is_some_and(|e| e == "md") {
            out.push(path);
        }
    }
}

/// Filters for [`Shelf::list`].
#[derive(Debug, Clone, Default)]
pub struct ListFilter {
    pub library: Option<String>,
    pub kind: Option<Kind>,
    pub tag: Option<String>,
}

/// A search result.
#[derive(Debug, Clone, Serialize)]
pub struct Hit<'a> {
    pub score: u32,
    #[serde(flatten)]
    pub prompt: &'a Prompt,
}

#[derive(Debug, Clone)]
pub struct Shelf {
    pub config: Config,
    pub libraries: Vec<Library>,
    pub diagnostics: Vec<Diagnostic>,
}

impl Shelf {
    /// Loads every library. Your open proposals replace the team's version of
    /// those prompts, so on this machine a prompt you proposed fills with your
    /// text. [`Shelf::load_main`] skips that.
    pub fn load(config: Config) -> Self {
        let mut shelf = Self::load_main(config);
        shelf.overlay_proposals();
        shelf
    }

    fn overlay_proposals(&mut self) {
        for p in crate::proposals::load() {
            let Some(lib) = self.libraries.iter_mut().find(|l| l.name() == p.library) else {
                continue;
            };
            let path = crate::proposals::file_in_worktree(lib, &p);
            let kind = lib.get(&p.id).map(|x| x.kind).unwrap_or(Kind::Template);
            let Ok(mut prompt) = Prompt::load(&path, &p.library, kind) else {
                continue;
            };
            if prompt.id != p.id {
                continue;
            }
            prompt.proposal = true;
            lib.prompts.retain(|x| x.id != p.id);
            lib.prompts.push(prompt);
        }
    }

    /// Loads every library as it is on its main branch.
    pub fn load_main(config: Config) -> Self {
        let mut diagnostics = Vec::new();
        let libraries = config
            .libraries
            .iter()
            .cloned()
            .map(|c| Library::scan(c, &mut diagnostics))
            .collect();
        Shelf {
            config,
            libraries,
            diagnostics,
        }
    }

    pub fn personal(&self) -> Option<&Library> {
        let name = &self.config.personal()?.name;
        self.libraries.iter().find(|l| &l.config.name == name)
    }

    pub fn library(&self, name: &str) -> Result<&Library> {
        self.libraries
            .iter()
            .find(|l| l.name() == name)
            .ok_or_else(|| Error::UnknownLibrary(name.to_string()))
    }

    pub fn prompts(&self) -> impl Iterator<Item = &Prompt> {
        self.libraries.iter().flat_map(|l| l.prompts.iter())
    }

    /// Finds a prompt by `<library>/<id>` or a bare `<id>`. A bare ID looks in
    /// Personal first, then in every library; more than one match is an error that
    /// lists them.
    pub fn resolve(&self, id: &str) -> Result<&Prompt> {
        if let Some((lib, local)) = id.split_once('/') {
            return self
                .library(lib)?
                .get(local)
                .ok_or_else(|| Error::NotFound(id.to_string()));
        }
        if let Some(found) = self.personal().and_then(|l| l.get(id)) {
            return Ok(found);
        }
        let matches: Vec<&Prompt> = self.prompts().filter(|p| p.id == id).collect();
        match matches.as_slice() {
            [] => Err(Error::NotFound(id.to_string())),
            [one] => Ok(one),
            many => Err(Error::Ambiguous {
                id: id.to_string(),
                matches: many.iter().map(|p| p.full_id()).collect(),
            }),
        }
    }

    /// Resolves an include from inside `from_library`: the same library first.
    pub fn resolve_from(&self, from_library: &str, id: &str) -> Result<&Prompt> {
        if !id.contains('/') {
            if let Some(found) = self.library(from_library).ok().and_then(|l| l.get(id)) {
                return Ok(found);
            }
        }
        self.resolve(id)
    }

    pub fn list(&self, filter: &ListFilter) -> Result<Vec<&Prompt>> {
        if let Some(lib) = &filter.library {
            self.library(lib)?;
        }
        Ok(self
            .prompts()
            .filter(|p| filter.library.as_deref().is_none_or(|l| p.library == l))
            .filter(|p| filter.kind.is_none_or(|k| p.kind == k))
            .filter(|p| {
                filter
                    .tag
                    .as_deref()
                    .is_none_or(|t| p.tags.iter().any(|x| x.eq_ignore_ascii_case(t)))
            })
            .collect())
    }

    /// Every word of the query must match the ID, title, tags, description or text.
    /// Matches in the ID and title rank higher than matches in the text.
    pub fn search(&self, query: &str, filter: &ListFilter) -> Result<Vec<Hit<'_>>> {
        let words: Vec<String> = query.split_whitespace().map(|w| w.to_lowercase()).collect();
        let mut hits: Vec<Hit> = self
            .list(filter)?
            .into_iter()
            .filter_map(|p| score(p, &words).map(|score| Hit { score, prompt: p }))
            .collect();
        hits.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.prompt.full_id().cmp(&b.prompt.full_id()))
        });
        Ok(hits)
    }
}

fn score(p: &Prompt, words: &[String]) -> Option<u32> {
    if words.is_empty() {
        return Some(0);
    }
    let id = p.id.to_lowercase();
    let title = p.title.to_lowercase();
    let desc = p.description.as_deref().unwrap_or("").to_lowercase();
    let body = p.body.to_lowercase();
    let mut total = 0;
    for w in words {
        let mut best = 0;
        if id == *w {
            best = best.max(12);
        }
        if id.split('-').any(|part| part.starts_with(w.as_str())) {
            best = best.max(8);
        } else if id.contains(w.as_str()) {
            best = best.max(6);
        }
        if title
            .split_whitespace()
            .any(|part| part.starts_with(w.as_str()))
        {
            best = best.max(7);
        } else if title.contains(w.as_str()) {
            best = best.max(5);
        }
        if p.tags.iter().any(|t| t.eq_ignore_ascii_case(w)) {
            best = best.max(6);
        } else if p.tags.iter().any(|t| t.to_lowercase().contains(w.as_str())) {
            best = best.max(4);
        }
        if desc.contains(w.as_str()) {
            best = best.max(3);
        }
        if body.contains(w.as_str()) {
            best = best.max(1);
        }
        if best == 0 {
            return None;
        }
        total += best;
    }
    Some(total)
}

#[cfg(test)]
#[path = "tests/shelf.rs"]
mod tests;
