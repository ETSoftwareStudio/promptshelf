//! Sync a library with its git remote. It never force-pushes, and a real conflict
//! never leaves the repo half-merged: the rebase is aborted and your version stays.

use std::path::Path;

use serde::Serialize;

use crate::config::Writes;
use crate::error::Result;
use crate::git;
use crate::pins;
use crate::shelf::Library;

const PUSH_ATTEMPTS: usize = 3;

#[derive(Debug, Clone, Copy)]
pub struct Options<'a> {
    /// Commit finished edits first (a checkpoint).
    pub commit: bool,
    pub push: bool,
    /// The program git runs as the pin merge driver, usually `shelf`.
    pub merge_driver: &'a str,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Report {
    pub library: String,
    /// Why nothing happened, for example "not a git repo".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped: Option<String>,
    /// The checkpoint commit message, when one was made.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub committed: Option<String>,
    /// Commits pulled from the remote.
    pub pulled: u32,
    /// Commits pushed to the remote.
    pub pushed: u32,
    /// Files edited both here and on the remote. Your version stays in use.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub conflicts: Vec<String>,
    /// Uncommitted changes left after the sync.
    pub changes: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub fn sync(lib: &Library, personal: bool, opts: &Options) -> Report {
    let mut report = Report {
        library: lib.name().to_string(),
        ..Default::default()
    };
    if !git::is_repo(&lib.root) {
        report.skipped = Some("not a git repo".into());
        return report;
    }
    if let Err(e) = run(lib, personal, opts, &mut report) {
        report.error = Some(e.to_string());
    }
    report.changes = git::changed_paths(&lib.root).map(|c| c.len()).unwrap_or(0);
    report
}

fn run(lib: &Library, personal: bool, opts: &Options, report: &mut Report) -> Result<()> {
    let root = &lib.root;
    if personal {
        setup_merge_driver(root, opts.merge_driver)?;
    }
    let team = lib.config.writes == Writes::PullRequest;
    if opts.commit && !team {
        report.committed = checkpoint(root)?;
    }
    if git::upstream(root).is_none() {
        let has_remote = !git::run(root, &["remote"])?.trim().is_empty();
        if has_remote && opts.push && !team && git::branch(root).is_some() {
            git::run(root, &["push", "-q", "-u", "origin", "HEAD"])?;
            report.pushed = count(root, "HEAD").unwrap_or(0);
        }
        return Ok(());
    }
    for attempt in 1..=PUSH_ATTEMPTS {
        git::run(root, &["fetch", "-q", "--prune"])?;
        let (ahead, behind) = git::ahead_behind(root).unwrap_or((0, 0));
        if behind > 0 {
            if team {
                git::run(root, &["merge", "-q", "--ff-only", "@{u}"]).map_err(|_| {
                    crate::Error::Git(
                        "the library has local commits on its main branch; it can only fast-forward"
                            .into(),
                    )
                })?;
            } else if !rebase(root, report)? {
                return Ok(());
            }
            report.pulled += behind;
        }
        if team || !opts.push || ahead == 0 {
            return Ok(());
        }
        let push = git::try_run(root, &["push", "-q"])?;
        if push.ok {
            report.pushed += ahead;
            return Ok(());
        }
        let rejected = push.stderr.contains("rejected") || push.stderr.contains("fetch first");
        if !rejected || attempt == PUSH_ATTEMPTS {
            return Err(crate::Error::Git(format!(
                "git push failed: {}",
                push.stderr.trim()
            )));
        }
    }
    Ok(())
}

/// Rebases onto the upstream. On a conflict it aborts, records the files and
/// returns false.
fn rebase(root: &Path, report: &mut Report) -> Result<bool> {
    let r = git::try_run(root, &["rebase", "-q", "--autostash", "@{u}"])?;
    if r.ok {
        return Ok(true);
    }
    let files = git::run(root, &["diff", "--name-only", "--diff-filter=U"]).unwrap_or_default();
    report.conflicts = files.lines().map(str::to_string).collect();
    if report.conflicts.is_empty() {
        report.conflicts.push("(unknown)".into());
    }
    let _ = git::try_run(root, &["rebase", "--abort"]);
    Ok(false)
}

fn count(root: &Path, rev: &str) -> Option<u32> {
    git::run(root, &["rev-list", "--count", rev])
        .ok()?
        .trim()
        .parse()
        .ok()
}

/// Commits every change in the library. Returns the message, or `None` when
/// there was nothing to commit.
pub fn checkpoint(root: &Path) -> Result<Option<String>> {
    let status = git::run(root, &["status", "--porcelain", "--untracked-files=all"])?;
    let lines: Vec<&str> = status.lines().filter(|l| l.len() > 3).collect();
    if lines.is_empty() {
        return Ok(None);
    }
    let message = match lines.as_slice() {
        [one] => {
            let (code, path) = one.split_at(3);
            let stem = Path::new(path.trim_matches('"'))
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            let verb = match code.trim() {
                "??" | "A" => "Add",
                "D" => "Remove",
                _ => "Edit",
            };
            format!("{verb} {stem}")
        }
        many => format!("Update {} files", many.len()),
    };
    git::run(root, &["add", "-A"])?;
    let mut args: Vec<&str> = Vec::new();
    let has_identity = git::try_run(root, &["config", "user.email"])?.ok;
    if !has_identity {
        args.extend([
            "-c",
            "user.name=PromptShelf",
            "-c",
            "user.email=promptshelf@localhost",
        ]);
    }
    args.extend(["commit", "-q", "--no-verify", "-m", &message]);
    git::run(root, &args)?;
    Ok(Some(message))
}

/// Points git at `shelf merge-pins` for the pin list, so two Macs that pin
/// different prompts never conflict.
pub fn setup_merge_driver(root: &Path, program: &str) -> Result<()> {
    let driver = format!("\"{program}\" merge-pins %O %A %B");
    git::run(
        root,
        &["config", "merge.promptshelf-pins.name", "PromptShelf pins"],
    )?;
    git::run(root, &["config", "merge.promptshelf-pins.driver", &driver])?;
    let path = root.join(".gitattributes");
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    if !text.lines().any(|l| l.trim() == pins::GITATTRIBUTES_LINE) {
        let mut text = text;
        if !text.is_empty() && !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str(pins::GITATTRIBUTES_LINE);
        text.push('\n');
        crate::fsutil::write_atomic(&path, text.as_bytes())?;
    }
    Ok(())
}
