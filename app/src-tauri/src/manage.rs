//! Libraries and git, without Tauri: first run, adding and removing
//! libraries, remotes and their state.

use promptshelf_core::config::{LibraryConfig, Writes, expand_tilde};
use promptshelf_core::{Config, Error, Result, Shelf, git, setup, tools};

use crate::service::LastSyncs;
use crate::views::{Environment, LibraryStatus};

/// The default folder for the Personal library.
pub const DEFAULT_PERSONAL: &str = "~/Prompts/personal";

fn load_or_default() -> Result<Config> {
    match Config::load() {
        Ok(c) => Ok(c),
        Err(Error::NoConfig) => Ok(Config::default()),
        Err(e) => Err(e),
    }
}

/// The program git runs as the pin merge driver: the `shelf` command when
/// PromptShelf finds it, by its full path, since git may not have the PATH
/// of a terminal.
pub fn merge_driver() -> String {
    tools::find_program("shelf")
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "shelf".into())
}

/// First run: makes `path` the Personal library. With `url`, clones it there
/// first. A folder that is a library already keeps its files.
pub fn init(path: &str, url: Option<&str>) -> Result<String> {
    let path = if path.trim().is_empty() {
        DEFAULT_PERSONAL
    } else {
        path.trim()
    };
    if let Some(url) = url.map(str::trim).filter(|u| !u.is_empty()) {
        let root = expand_tilde(path);
        if root.read_dir().is_ok_and(|mut d| d.next().is_some()) {
            return Err(Error::Invalid(format!(
                "{} has files already; choose an empty folder for the clone",
                root.display()
            )));
        }
        let _ = std::fs::remove_dir(&root);
        setup::clone(url, &root, false)?;
    }
    let mut config = load_or_default()?;
    let init = setup::init_personal(&mut config, path, &merge_driver())?;
    if init.created {
        if let Some(url) = url.filter(|u| !u.trim().is_empty())
            && let Some(p) = config.libraries.iter_mut().find(|l| l.personal)
        {
            p.remote = Some(url.trim().to_string());
        }
        config.save()?;
    }
    Ok(init.root.display().to_string())
}

/// What Settings → Add Library sends.
#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewLibrary {
    pub name: String,
    /// A folder, or a git URL to clone.
    pub source: String,
    /// Where to clone a git URL.
    pub path: Option<String>,
    pub writes: Option<Writes>,
    pub trusted: Option<bool>,
}

pub fn add_library(new: &NewLibrary) -> Result<LibraryConfig> {
    let mut config = Config::load()?;
    let lib = setup::add_library(
        &mut config,
        &setup::AddLibrary {
            name: new.name.trim(),
            source: new.source.trim(),
            path: new.path.as_deref().map(str::trim).filter(|p| !p.is_empty()),
            writes: new.writes,
            trusted: new.trusted,
            show_progress: false,
        },
    )?;
    config.save()?;
    Ok(lib)
}

pub fn update_library(name: &str, writes: Option<Writes>, trusted: Option<bool>) -> Result<()> {
    let mut config = Config::load()?;
    setup::update_library(&mut config, name, writes, trusted)?;
    config.save()
}

/// Removes a library from the config. Its folder stays.
pub fn remove_library(name: &str) -> Result<String> {
    let mut config = Config::load()?;
    let lib = setup::remove_library(&mut config, name)?;
    config.save()?;
    Ok(lib.root().display().to_string())
}

pub fn connect_remote(name: &str, url: &str) -> Result<()> {
    let mut config = Config::load()?;
    setup::connect_remote(&mut config, name, url)?;
    config.save()
}

/// Creates a private GitHub repo for a library with `gh`, and connects it.
pub fn create_github_repo(name: &str, repo: &str) -> Result<String> {
    let mut config = Config::load()?;
    let root = config.library(name)?.root();
    let url = tools::create_github_repo("gh", &root, repo.trim())?;
    setup::connect_remote(&mut config, name, &url)?;
    config.save()?;
    Ok(url)
}

/// The git state of each library.
pub fn library_status(shelf: &Shelf, last: &LastSyncs) -> Vec<LibraryStatus> {
    shelf
        .libraries
        .iter()
        .map(|lib| {
            let root = &lib.root;
            let is_repo = git::is_repo(root);
            let (ahead, behind) = if is_repo {
                git::ahead_behind(root).unwrap_or((0, 0))
            } else {
                (0, 0)
            };
            LibraryStatus {
                name: lib.name().to_string(),
                remote: is_repo
                    .then(|| git::run(root, &["remote", "get-url", "origin"]).ok())
                    .flatten()
                    .map(|s| s.trim().to_string()),
                branch: is_repo.then(|| git::branch(root)).flatten(),
                ahead,
                behind,
                changes: if is_repo {
                    git::changed_paths(root).map(|c| c.len()).unwrap_or(0)
                } else {
                    0
                },
                last_sync: last.get(lib.name()).cloned(),
            }
        })
        .collect()
}

pub fn environment() -> Environment {
    Environment {
        git: tools::git_info(),
        gh: tools::gh_status("gh"),
    }
}

#[cfg(test)]
#[path = "tests/manage.rs"]
mod tests;
