//! Runs the `git` binary. The core links no git library.

use std::path::Path;
use std::process::{Command, Output};

use crate::error::{Error, Result};

/// The result of a git command that is allowed to fail.
#[derive(Debug)]
pub struct Run {
    pub ok: bool,
    pub stdout: String,
    pub stderr: String,
}

fn command(dir: &Path, args: &[&str]) -> Command {
    let mut cmd = crate::tools::command("git");
    cmd.arg("-C")
        .arg(dir)
        .args(args)
        // Never wait for a password prompt or an editor.
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_EDITOR", "true")
        .env("GIT_MERGE_AUTOEDIT", "no")
        .env("LC_ALL", "C");
    cmd
}

fn finish(out: Output) -> Run {
    Run {
        ok: out.status.success(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

/// Runs git and returns its output, even when it fails.
pub fn try_run(dir: &Path, args: &[&str]) -> Result<Run> {
    let out = command(dir, args)
        .output()
        .map_err(|e| Error::Git(format!("could not run git: {e}")))?;
    Ok(finish(out))
}

/// Runs git and returns stdout. A failure is an error with git's message.
pub fn run(dir: &Path, args: &[&str]) -> Result<String> {
    let r = try_run(dir, args)?;
    if r.ok {
        Ok(r.stdout)
    } else {
        Err(Error::Git(format!(
            "git {} failed: {}",
            args.join(" "),
            r.stderr.trim()
        )))
    }
}

/// True when `dir` is inside a git work tree.
pub fn is_repo(dir: &Path) -> bool {
    try_run(dir, &["rev-parse", "--is-inside-work-tree"])
        .map(|r| r.ok && r.stdout.trim() == "true")
        .unwrap_or(false)
}

/// The current branch, or `None` on a detached HEAD.
pub fn branch(dir: &Path) -> Option<String> {
    let name = run(dir, &["rev-parse", "--abbrev-ref", "HEAD"]).ok()?;
    let name = name.trim();
    (name != "HEAD" && !name.is_empty()).then(|| name.to_string())
}

/// The upstream of the current branch, such as `origin/main`.
pub fn upstream(dir: &Path) -> Option<String> {
    let r = try_run(
        dir,
        &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"],
    )
    .ok()?;
    let name = r.stdout.trim();
    (r.ok && !name.is_empty()).then(|| name.to_string())
}

/// Paths with uncommitted changes, from `git status --porcelain`.
pub fn changed_paths(dir: &Path) -> Result<Vec<String>> {
    let out = run(
        dir,
        &["status", "--porcelain", "-z", "--untracked-files=all"],
    )?;
    let mut paths = Vec::new();
    let mut fields = out.split('\0').filter(|s| !s.is_empty());
    while let Some(entry) = fields.next() {
        if entry.len() < 4 {
            continue;
        }
        let (status, path) = entry.split_at(3);
        paths.push(path.to_string());
        // A rename has a second field with the old path.
        if status.starts_with('R') || status.starts_with('C') {
            fields.next();
        }
    }
    Ok(paths)
}

/// Commits ahead of and behind the upstream.
pub fn ahead_behind(dir: &Path) -> Option<(u32, u32)> {
    let out = run(dir, &["rev-list", "--left-right", "--count", "HEAD...@{u}"]).ok()?;
    let mut it = out.split_whitespace().map(|n| n.parse().ok());
    Some((it.next()??, it.next()??))
}
