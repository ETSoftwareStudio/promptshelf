//! What the app's write commands do: pins, edits, new prompts and sync. The
//! app acts for a person, so it may write.

use promptshelf_core::config::{LibraryConfig, Writes};
use promptshelf_core::pins::Pins;
use promptshelf_core::sync::{self as core_sync, Report};
use promptshelf_core::write::{self, NewPrompt};
use promptshelf_core::{Config, Error, Kind, Result, Shelf, edit};

fn personal_root(shelf: &Shelf) -> Result<std::path::PathBuf> {
    shelf
        .personal()
        .map(|l| l.root.clone())
        .ok_or_else(|| Error::Invalid("no Personal library for pins".into()))
}

/// Pins a prompt at the end, or at `at` (1-based). Returns the new pin list.
pub fn pin(shelf: &Shelf, full_id: &str, at: Option<usize>) -> Result<Pins> {
    let full = shelf.resolve(full_id)?.full_id();
    let root = personal_root(shelf)?;
    let mut pins = Pins::load(&root)?;
    pins.pin(&full, at)?;
    pins.save(&root)?;
    Ok(pins)
}

/// Unpins a prompt. It works for a prompt that no longer exists, too.
pub fn unpin(shelf: &Shelf, full_id: &str) -> Result<Pins> {
    let root = personal_root(shelf)?;
    let mut pins = Pins::load(&root)?;
    let full = shelf
        .resolve(full_id)
        .map(|p| p.full_id())
        .unwrap_or_else(|_| full_id.to_string());
    pins.unpin(&full);
    pins.save(&root)?;
    Ok(pins)
}

/// Writes the whole prompt file after the editor. The text must parse as a
/// prompt. In a team library it goes into your proposal, never into the main
/// folder. Returns the full ID, which changes when the frontmatter changes it.
pub fn save_file(shelf: &Shelf, full_id: &str, text: &str) -> Result<String> {
    edit::save_text(shelf, full_id, text)
}

/// Creates a new prompt in a library (Personal when none is named).
pub fn new_prompt(shelf: &Shelf, title: &str, kind: Kind, library: Option<&str>) -> Result<String> {
    let lib = match library {
        Some(name) => shelf.library(name)?,
        None => shelf
            .personal()
            .ok_or_else(|| Error::Invalid("no Personal library".into()))?,
    };
    let title = if title.trim().is_empty() {
        "Untitled"
    } else {
        title.trim()
    };
    let (id, _) = write::create_in(
        lib,
        &NewPrompt {
            title,
            kind,
            body: "Write the prompt here. Use {{name}} for a blank.\n",
            tags: &[],
        },
    )?;
    Ok(format!("{}/{id}", lib.name()))
}

/// Syncs every library, or one. The app is a person, so it commits and pushes.
pub fn sync(shelf: &Shelf, library: Option<&str>, merge_driver: &str) -> Result<Vec<Report>> {
    let opts = core_sync::Options {
        commit: true,
        push: true,
        merge_driver,
    };
    let personal = shelf.personal().map(|l| l.name().to_string());
    let mut reports = Vec::new();
    for lib in &shelf.libraries {
        if library.is_some_and(|n| n != lib.name()) {
            continue;
        }
        let is_personal = personal.as_deref() == Some(lib.name());
        reports.push(core_sync::sync(lib, is_personal, &opts));
    }
    if let Some(name) = library
        && reports.is_empty()
    {
        return Err(Error::UnknownLibrary(name.to_string()));
    }
    Ok(reports)
}

/// The program git runs as the pin merge driver. The desktop app is not
/// `shelf`, so it names `shelf` and expects it on the PATH, as the CLI does.
pub fn merge_driver() -> String {
    "shelf".into()
}

/// First run: creates `config.toml` with a Personal library, like
/// `shelf init`. Existing config and files are kept.
pub fn init(path: &str) -> Result<String> {
    let mut config = match Config::load() {
        Ok(c) => c,
        Err(Error::NoConfig) => Config::default(),
        Err(e) => return Err(e),
    };
    if config.personal().is_none() {
        config.libraries.push(LibraryConfig {
            name: "personal".into(),
            path: path.to_string(),
            remote: None,
            writes: Writes::Commit,
            personal: true,
            trusted: None,
            forge: None,
            api: None,
        });
        config.save()?;
    }
    let personal = config.personal().expect("personal library exists");
    let root = personal.root();
    write::init_library(&root)?;
    if !promptshelf_core::git::is_repo(&root)
        && promptshelf_core::git::run(&root, &["init", "-q", "-b", "main"]).is_ok()
    {
        core_sync::setup_merge_driver(&root, &merge_driver())?;
    }
    Ok(root.display().to_string())
}

#[cfg(test)]
#[path = "tests/writes.rs"]
mod tests;
