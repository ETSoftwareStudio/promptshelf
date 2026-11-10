//! The outside programs the core runs: git and gh. Where they are, if they
//! are set up, and plain words for their common failures.
//!
//! An app started from the Dock or a desktop menu gets a short `PATH`, without
//! Homebrew or `~/.local/bin`. Every program the core starts gets [`path_env`],
//! which adds those folders after your own `PATH`, so a person's own setup
//! still wins.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

use serde::Serialize;

use crate::config::home_dir;
use crate::error::{Error, Result};

/// `PATH` plus the usual install folders, for every program the core starts.
pub fn path_env() -> &'static OsString {
    static PATH: OnceLock<OsString> = OnceLock::new();
    PATH.get_or_init(|| {
        let mut dirs: Vec<PathBuf> = std::env::var_os("PATH")
            .map(|p| std::env::split_paths(&p).collect())
            .unwrap_or_default();
        for extra in extra_dirs() {
            if !dirs.contains(&extra) {
                dirs.push(extra);
            }
        }
        std::env::join_paths(dirs).unwrap_or_default()
    })
}

fn extra_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(home) = home_dir() {
        dirs.push(home.join(".local").join("bin"));
        dirs.push(home.join(".cargo").join("bin"));
    }
    if !cfg!(windows) {
        for d in [
            "/opt/homebrew/bin",
            "/usr/local/bin",
            "/home/linuxbrew/.linuxbrew/bin",
        ] {
            dirs.push(PathBuf::from(d));
        }
    }
    dirs
}

/// A command with [`path_env`] as its `PATH`.
pub fn command(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut cmd = Command::new(program);
    cmd.env("PATH", path_env());
    cmd
}

/// The full path of a program on [`path_env`].
pub fn find_program(name: &str) -> Option<PathBuf> {
    let exe = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    std::env::split_paths(path_env())
        .map(|d| d.join(&exe))
        .find(|p| p.is_file())
}

/// git, and the name and email it puts on commits.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct GitInfo {
    /// `git --version`, or `None` when git does not run.
    pub version: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
}

pub fn git_info() -> GitInfo {
    let run = |args: &[&str]| -> Option<String> {
        let out = command("git")
            .args(args)
            .stdin(Stdio::null())
            .output()
            .ok()
            .filter(|o| o.status.success())?;
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        (!s.is_empty()).then_some(s)
    };
    let Some(version) = run(&["--version"]) else {
        return GitInfo::default();
    };
    GitInfo {
        version: Some(version),
        name: run(&["config", "--global", "user.name"]),
        email: run(&["config", "--global", "user.email"]),
    }
}

/// Sets the name and email git puts on your commits, for every repo.
pub fn set_git_identity(name: &str, email: &str) -> Result<()> {
    let (name, email) = (name.trim(), email.trim());
    if name.is_empty() || !email.contains('@') {
        return Err(Error::Invalid("give a name and an email address".into()));
    }
    for (key, value) in [("user.name", name), ("user.email", email)] {
        let out = command("git")
            .args(["config", "--global", key, value])
            .output()
            .map_err(|e| Error::Git(format!("could not run git: {e}")))?;
        if !out.status.success() {
            return Err(Error::Git(format!(
                "git config {key} failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )));
        }
    }
    Ok(())
}

/// The GitHub CLI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum Gh {
    Missing,
    SignedOut,
    SignedIn {
        #[serde(skip_serializing_if = "Option::is_none")]
        account: Option<String>,
    },
}

/// Asks `gh` (or the program at `program`) if it is signed in.
pub fn gh_status(program: &str) -> Gh {
    let Ok(out) = command(program)
        .args(["auth", "status", "--hostname", "github.com"])
        .stdin(Stdio::null())
        .output()
    else {
        return Gh::Missing;
    };
    if !out.status.success() {
        return Gh::SignedOut;
    }
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let account = text
        .split_whitespace()
        .skip_while(|w| *w != "account")
        .nth(1);
    Gh::SignedIn {
        account: account.map(str::to_string),
    }
}

/// Creates a private GitHub repo with `gh` (or the program at `program`) and
/// adds it as `origin` of the library at `root`. Returns the remote URL. It
/// pushes nothing; the next sync does.
pub fn create_github_repo(program: &str, root: &Path, name: &str) -> Result<String> {
    if !crate::prompt::is_valid_id(name) {
        return Err(Error::Invalid(format!(
            "\"{name}\" is not a repo name: use a-z, 0-9 and -"
        )));
    }
    if !crate::git::is_repo(root) {
        crate::git::run(root, &["init", "-q", "-b", "main"])?;
    }
    if crate::git::try_run(root, &["remote", "get-url", "origin"])?.ok {
        return Err(Error::Invalid("this library has a remote already".into()));
    }
    let out = command(program)
        .args(["repo", "create", name, "--private", "--remote", "origin"])
        .arg("--source")
        .arg(root)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| Error::Git(format!("could not run gh: {e}")))?;
    if !out.status.success() {
        return Err(Error::Git(format!(
            "gh repo create failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(crate::git::run(root, &["remote", "get-url", "origin"])?
        .trim()
        .to_string())
}

/// Plain words for a git or gh failure, and where to read more.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Hint {
    pub code: &'static str,
    pub message: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<&'static str>,
}

const HINTS: &[(&[&str], Hint)] = &[
    (
        &["could not run git", "invalid active developer path"],
        Hint {
            code: "git-missing",
            message: "Git is not installed. Install it, then open PromptShelf again.",
            url: Some("https://git-scm.com/downloads"),
        },
    ),
    (
        &[
            "please tell me who you are",
            "empty ident",
            "unable to auto-detect email",
        ],
        Hint {
            code: "no-identity",
            message: "Git has no name and email for your commits. Set them in Settings → Libraries.",
            url: None,
        },
    ),
    (
        &["host key verification failed"],
        Hint {
            code: "host-key",
            message: "SSH does not know this host yet. Use the HTTPS address of the repo instead.",
            url: None,
        },
    ),
    (
        &[
            "permission denied (publickey)",
            "authentication failed",
            "could not read username",
            "could not read password",
            "terminal prompts disabled",
            "invalid username or password",
            "error: 403",
            "the requested url returned error: 401",
        ],
        Hint {
            code: "auth",
            message: "The host did not accept your login. Sign in to GitHub CLI (gh auth login) or set up an SSH key, then try again.",
            url: Some(
                "https://docs.github.com/en/get-started/git-basics/caching-your-github-credentials-in-git",
            ),
        },
    ),
    (
        &[
            "repository not found",
            "does not appear to be a git repository",
            "not found",
        ],
        Hint {
            code: "no-repo",
            message: "The repo was not found. Check the address, and that your account can open it.",
            url: None,
        },
    ),
    (
        &[
            "could not resolve host",
            "network is unreachable",
            "timed out",
            "connection refused",
        ],
        Hint {
            code: "offline",
            message: "The host could not be reached. Check the network, then try again.",
            url: None,
        },
    ),
];

/// Finds the plain words for an error message from git or gh.
pub fn explain(message: &str) -> Option<Hint> {
    let lower = message.to_lowercase();
    HINTS
        .iter()
        .find(|(needles, _)| needles.iter().any(|n| lower.contains(n)))
        .map(|(_, hint)| *hint)
}

#[cfg(test)]
#[path = "tests/tools.rs"]
mod tests;
