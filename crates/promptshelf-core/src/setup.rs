//! Setting up libraries: the Personal library, adding, changing and removing
//! libraries, and connecting a library to a remote. The CLI and the desktop
//! app both call these.

use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::config::{Config, LibraryConfig, Writes, expand_tilde};
use crate::error::{Error, Result};
use crate::{git, write};

/// What [`init_personal`] did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Init {
    /// False when a Personal library was already set up.
    pub created: bool,
    pub name: String,
    pub root: PathBuf,
}

/// Adds the Personal library at `path` when the config has none. The folder gets the standard layout and becomes a git repo.
/// Existing files are kept, so `path` can be a library you already have.
pub fn init_personal(config: &mut Config, path: &str, merge_driver: &str) -> Result<Init> {
    let created = config.personal().is_none();
    if created {
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
    }
    let personal = config.personal().expect("personal library exists");
    let root = personal.root();
    write::init_library(&root)?;
    // Personal is a git repo from the start, so sync can work later.
    if !git::is_repo(&root) && git::run(&root, &["init", "-q", "-b", "main"]).is_ok() {
        crate::sync::setup_merge_driver(&root, merge_driver)?;
    }
    Ok(Init {
        created,
        name: personal.name.clone(),
        root,
    })
}

/// True when `s` looks like a git URL rather than a folder.
pub fn looks_like_git_url(s: &str) -> bool {
    s.contains("://") || (s.contains('@') && s.contains(':')) || s.ends_with(".git")
}

/// A library to attach.
#[derive(Debug, Clone, Default)]
pub struct AddLibrary<'a> {
    pub name: &'a str,
    /// A folder, or a git URL to clone.
    pub source: &'a str,
    /// Where to clone a git URL. Defaults to `~/Prompts/<name>`.
    pub path: Option<&'a str>,
    /// Defaults to pull requests for a git URL and commits for a folder.
    pub writes: Option<Writes>,
    pub trusted: Option<bool>,
    /// Show git's progress on the terminal (the CLI). Without it, git's output
    /// goes into the error, and git never asks for a password.
    pub show_progress: bool,
}

/// Checks a new library name against the config.
pub fn check_name(config: &Config, name: &str) -> Result<()> {
    if !crate::prompt::is_valid_id(name) {
        return Err(Error::Invalid(format!(
            "library name \"{name}\" must use only a-z, 0-9 and -"
        )));
    }
    if config.libraries.iter().any(|l| l.name == name) {
        return Err(Error::Invalid(format!("a library named \"{name}\" exists")));
    }
    Ok(())
}

/// Attaches a library: a folder as it is, or a clone of a git URL. Returns
/// the new entry.
pub fn add_library(config: &mut Config, a: &AddLibrary) -> Result<LibraryConfig> {
    check_name(config, a.name)?;
    let is_url = looks_like_git_url(a.source);
    let (path, remote) = if is_url {
        let path = a
            .path
            .map(str::to_string)
            .unwrap_or_else(|| format!("~/Prompts/{}", a.name));
        let root = expand_tilde(&path);
        if root.exists() {
            return Err(Error::Invalid(format!(
                "{} exists; pass --path for another folder",
                root.display()
            )));
        }
        clone(a.source, &root, a.show_progress)?;
        (path, Some(a.source.to_string()))
    } else {
        let root = expand_tilde(a.source);
        if !root.is_dir() {
            return Err(Error::Invalid(format!(
                "{} is not a folder",
                root.display()
            )));
        }
        // Store an absolute path, so the library works from any folder.
        let path = if a.source.starts_with('~') {
            a.source.to_string()
        } else {
            std::path::absolute(&root)
                .map_err(|e| Error::io(&root, e))?
                .display()
                .to_string()
        };
        (path, None)
    };
    let writes = a.writes.unwrap_or(if is_url {
        Writes::PullRequest
    } else {
        Writes::Commit
    });
    let lib = LibraryConfig {
        name: a.name.to_string(),
        path,
        remote,
        writes,
        personal: false,
        trusted: a.trusted,
        forge: None,
        api: None,
    };
    config.libraries.push(lib.clone());
    Ok(lib)
}

/// Clones `url` into `dest`.
pub fn clone(url: &str, dest: &std::path::Path, show_progress: bool) -> Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
    }
    let mut cmd = Command::new("git");
    cmd.args(["clone", "--", url]).arg(dest);
    if show_progress {
        let status = cmd
            .status()
            .map_err(|e| Error::Git(format!("could not run git: {e}")))?;
        if !status.success() {
            return Err(Error::Git(format!("git clone {url} failed")));
        }
        return Ok(());
    }
    let out = cmd
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::null())
        .output()
        .map_err(|e| Error::Git(format!("could not run git: {e}")))?;
    if !out.status.success() {
        return Err(Error::Git(format!(
            "git clone {url} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(())
}

/// Changes how changes go into a library, and whether its commands run
/// without an allow step. `None` keeps the current value.
pub fn update_library(
    config: &mut Config,
    name: &str,
    writes: Option<Writes>,
    trusted: Option<bool>,
) -> Result<()> {
    let lib = config
        .libraries
        .iter_mut()
        .find(|l| l.name == name)
        .ok_or_else(|| Error::UnknownLibrary(name.to_string()))?;
    if let Some(w) = writes {
        lib.writes = w;
    }
    if let Some(t) = trusted {
        lib.trusted = Some(t);
    }
    Ok(())
}

/// Removes a library from the config. Its folder and files stay on disk.
/// The Personal library holds pins and drafts, so it can not be removed.
pub fn remove_library(config: &mut Config, name: &str) -> Result<LibraryConfig> {
    config.library(name)?;
    if config.personal().is_some_and(|p| p.name == name) {
        return Err(Error::Invalid(
            "the Personal library holds your pins and drafts; it can not be removed".into(),
        ));
    }
    let index = config
        .libraries
        .iter()
        .position(|l| l.name == name)
        .expect("library exists");
    Ok(config.libraries.remove(index))
}

/// Connects a library to a remote: makes the folder a git repo when it is not
/// one, sets `origin` and keeps the URL in the config. The next sync pulls
/// what the remote has and pushes.
pub fn connect_remote(config: &mut Config, name: &str, url: &str) -> Result<()> {
    let url = url.trim();
    if url.is_empty() {
        return Err(Error::Invalid("the remote URL is empty".into()));
    }
    let root = config.library(name)?.root();
    if !root.is_dir() {
        return Err(Error::Invalid(format!("{} does not exist", root.display())));
    }
    if !git::is_repo(&root) {
        git::run(&root, &["init", "-q", "-b", "main"])?;
    }
    let has_origin = git::try_run(&root, &["remote", "get-url", "origin"])?.ok;
    let verb = if has_origin { "set-url" } else { "add" };
    git::run(&root, &["remote", verb, "origin", url])?;
    let lib = config
        .libraries
        .iter_mut()
        .find(|l| l.name == name)
        .expect("library exists");
    lib.remote = Some(url.to_string());
    Ok(())
}

#[cfg(test)]
#[path = "tests/setup.rs"]
mod tests;
