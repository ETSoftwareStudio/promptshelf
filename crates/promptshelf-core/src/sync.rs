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
    /// Prompts a team pull changed, now in the Inbox.
    #[serde(skip_serializing_if = "is_zero")]
    pub updates: usize,
    /// Uncommitted changes left after the sync.
    pub changes: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

fn is_zero(n: &usize) -> bool {
    *n == 0
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
    if git::upstream(root).is_none() && !track_remote_branch(root)? {
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
                let before = git::run(root, &["rev-parse", "HEAD"])?.trim().to_string();
                git::run(root, &["merge", "-q", "--ff-only", "@{u}"]).map_err(|_| {
                    crate::Error::Git(
                        "the library has local commits on its main branch; it can only fast-forward"
                            .into(),
                    )
                })?;
                let after = git::run(root, &["rev-parse", "HEAD"])?.trim().to_string();
                report.updates = crate::inbox::record(root, lib.name(), &before, &after)?;
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
    let mut args = identity(root)?;
    args.extend(["rebase", "-q", "--autostash", "@{u}"]);
    if git::try_run(root, &args)?.ok {
        return Ok(true);
    }
    rebase_failed(root, report);
    Ok(false)
}

/// A library just connected to a remote that already has its branch: track
/// that branch, so the sync pulls before it pushes. Returns true when the
/// branch now has an upstream.
fn track_remote_branch(root: &Path) -> Result<bool> {
    let Some(branch) = git::branch(root) else {
        return Ok(false);
    };
    if !git::try_run(root, &["remote", "get-url", "origin"])?.ok {
        return Ok(false);
    }
    let _ = git::try_run(root, &["fetch", "-q", "origin"]);
    let remote_branch = format!("origin/{branch}");
    if !git::try_run(root, &["rev-parse", "--verify", "-q", &remote_branch])?.ok {
        return Ok(false);
    }
    Ok(git::try_run(root, &["branch", "-q", "-u", &remote_branch])?.ok)
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
    let mut args = identity(root)?;
    args.extend(["commit", "-q", "--no-verify", "-m", &message]);
    git::run(root, &args)?;
    Ok(Some(message))
}

/// `-c` options with a fallback name and email when git has none, for the
/// commands that make commits. Your own identity always wins.
pub(crate) fn identity(root: &Path) -> Result<Vec<&'static str>> {
    if git::try_run(root, &["config", "user.email"])?.ok {
        return Ok(Vec::new());
    }
    Ok(vec![
        "-c",
        "user.name=PromptShelf",
        "-c",
        "user.email=promptshelf@localhost",
    ])
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

/// What to do with files edited both here and on the remote.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Resolution {
    /// Keep this computer's lines where both sides changed.
    Mine,
    /// Take the remote's lines where both sides changed.
    Theirs,
    /// Take the remote's lines, and save this computer's whole file as a new
    /// prompt next to it (`<id>-2`).
    Both,
}

/// Resolves the conflicts the last sync found, then syncs again. Nothing is
/// force-pushed: the remote's commits stay, and the choice decides only the
/// lines both sides changed.
pub fn resolve(lib: &Library, personal: bool, how: Resolution, opts: &Options) -> Report {
    let mut report = Report {
        library: lib.name().to_string(),
        ..Default::default()
    };
    if let Err(e) = resolve_run(&lib.root, how, &mut report) {
        report.error = Some(e.to_string());
    }
    if report.error.is_some() || !report.conflicts.is_empty() {
        report.changes = git::changed_paths(&lib.root).map(|c| c.len()).unwrap_or(0);
        return report;
    }
    let mut after = sync(lib, personal, opts);
    after.committed = report.committed.or(after.committed);
    after
}

fn resolve_run(root: &Path, how: Resolution, report: &mut Report) -> Result<()> {
    checkpoint(root)?;
    git::run(root, &["fetch", "-q", "--prune"])?;
    if git::upstream(root).is_none() {
        return Err(crate::Error::Invalid(
            "the library has no remote branch to resolve against".into(),
        ));
    }
    let mut keep = Vec::new();
    if how == Resolution::Both {
        let mut trial = Report::default();
        if rebase(root, &mut trial)? {
            return Ok(());
        }
        for file in trial.conflicts {
            if let Ok(text) = git::run(root, &["show", &format!("HEAD:{file}")]) {
                keep.push((file, text));
            }
        }
    }
    // In a rebase, "ours" is the remote and "theirs" is this computer's commits.
    let side = if how == Resolution::Mine {
        "theirs"
    } else {
        "ours"
    };
    let mut args = identity(root)?;
    args.extend(["rebase", "-q", "--autostash", "-X", side, "@{u}"]);
    if !git::try_run(root, &args)?.ok {
        rebase_failed(root, report);
        report.error = Some("git could not combine these files; edit one of them first".into());
        return Ok(());
    }
    for (file, text) in &keep {
        save_copy(root, file, text)?;
    }
    if !keep.is_empty() {
        report.committed = checkpoint(root)?;
    }
    Ok(())
}

/// Records the files of a failed rebase and aborts it.
fn rebase_failed(root: &Path, report: &mut Report) {
    let files = git::run(root, &["diff", "--name-only", "--diff-filter=U"]).unwrap_or_default();
    report.conflicts = files.lines().map(str::to_string).collect();
    if report.conflicts.is_empty() {
        report.conflicts.push("(unknown)".into());
    }
    let _ = git::try_run(root, &["rebase", "--abort"]);
}

/// Saves this computer's version of `file` as `<stem>-<n>.md` next to it,
/// with a matching ID.
fn save_copy(root: &Path, file: &str, text: &str) -> Result<()> {
    let path = root.join(file);
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let is_prompt = path.extension().is_some_and(|e| e == "md");
    for n in 2..100 {
        let id = format!("{stem}-{n}");
        let name = match path.extension() {
            Some(ext) => format!("{id}.{}", ext.to_string_lossy()),
            None => id.clone(),
        };
        let text = if is_prompt {
            crate::edit::set_field(text, "id", &id)
        } else {
            text.to_string()
        };
        if crate::fsutil::create_new(&path.with_file_name(name), text.as_bytes())? {
            return Ok(());
        }
    }
    Err(crate::Error::Invalid(format!(
        "no free name for a copy of {file}"
    )))
}

#[cfg(test)]
#[path = "tests/sync.rs"]
mod tests;
