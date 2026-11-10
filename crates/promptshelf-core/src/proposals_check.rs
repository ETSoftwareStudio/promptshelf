//! Knowing a proposal was merged, and cleaning up after it. Host API first,
//! then plain git, so squash and rebase merges are found too.

use serde::Serialize;

use super::{Proposal, Status, put, remove};
use crate::error::Result;
use crate::forge::{self, Forge, HostState};
use crate::git;
use crate::shelf::Library;

/// What a check did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Outcome {
    /// Merged: the worktree and branches are gone.
    Merged,
    /// Merged, but you edited it after the last send: a new proposal on main.
    MergedWithNewEdits,
    Unchanged,
    Updated,
}

/// Checks one proposal and cleans up after a merge.
pub fn check(lib: &Library, p: &mut Proposal) -> Result<Outcome> {
    if p.status == Status::NotSent || !p.worktree().is_dir() {
        return Ok(Outcome::Unchanged);
    }
    let root = &lib.root;
    let remote_url = git::run(root, &["remote", "get-url", "origin"])
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    let forge = Forge::of(&lib.config, &remote_url);
    let _ = git::try_run(root, &["fetch", "-q", "--prune", "origin"]);
    let base_ref = format!("origin/{}", p.base);
    let number = p.sent.as_ref().and_then(|s| s.number);
    let merged = match forge::state(forge, &remote_url, number) {
        Some(HostState::Merged) => true,
        Some(state) => {
            let status = match state {
                HostState::Closed => Status::NotMerged,
                HostState::ChangesRequested => Status::ChangesRequested,
                _ => Status::Sent,
            };
            return update(p, status);
        }
        None => in_main(p, &base_ref)?,
    };
    if !merged {
        let on_host = git::try_run(
            root,
            &["ls-remote", "--exit-code", "--heads", "origin", &p.branch],
        )?
        .ok;
        let status = if on_host || forge == Forge::Gitea {
            Status::Sent
        } else {
            Status::Gone
        };
        return update(p, status);
    }
    // Merged. Edits made after the last send are never deleted.
    let wt = p.worktree();
    let head = git::run(&wt, &["rev-parse", "HEAD"])?.trim().to_string();
    let dirty = !git::changed_paths(&wt)?.is_empty();
    if dirty || p.last_sent.as_deref() != Some(head.as_str()) {
        crate::sync::checkpoint(&wt)?;
        let mut args = crate::sync::identity(&wt)?;
        args.extend(["rebase", "-q", &base_ref]);
        if !git::try_run(&wt, &args)?.ok {
            let _ = git::try_run(&wt, &["rebase", "--abort"]);
        }
        p.status = Status::NotSent;
        p.sent = None;
        p.last_sent = None;
        put(p)?;
        return Ok(Outcome::MergedWithNewEdits);
    }
    discard(lib, p)?;
    let _ = git::try_run(root, &["merge", "-q", "--ff-only", &base_ref]);
    Ok(Outcome::Merged)
}

fn update(p: &mut Proposal, status: Status) -> Result<Outcome> {
    if p.status == status {
        return Ok(Outcome::Unchanged);
    }
    p.status = status;
    put(p)?;
    Ok(Outcome::Updated)
}

/// True when main has the proposal: as an ancestor (merge commit or fast-forward),
/// or with the same content in the proposal's files (squash or rebase merge).
fn in_main(p: &Proposal, base_ref: &str) -> Result<bool> {
    let wt = p.worktree();
    let Some(sent) = p.last_sent.as_deref() else {
        return Ok(false);
    };
    if git::try_run(&wt, &["merge-base", "--is-ancestor", sent, base_ref])?.ok {
        return Ok(true);
    }
    let fork = git::run(&wt, &["merge-base", sent, base_ref])?;
    let files = git::run(&wt, &["diff", "--name-only", fork.trim(), sent])?;
    let files: Vec<&str> = files.lines().filter(|l| !l.is_empty()).collect();
    if files.is_empty() {
        return Ok(false);
    }
    let mut args = vec!["diff", "--quiet", base_ref, sent, "--"];
    args.extend(files);
    Ok(git::try_run(&wt, &args)?.ok)
}

/// Removes a proposal: its worktree, its local branch, its branch on the host
/// when it is still there, and its record.
pub fn discard(lib: &Library, p: &Proposal) -> Result<()> {
    let root = &lib.root;
    let wt = p.worktree().display().to_string();
    let _ = git::try_run(root, &["worktree", "remove", "--force", &wt]);
    let _ = git::try_run(root, &["worktree", "prune"]);
    let _ = git::try_run(root, &["branch", "-D", &p.branch]);
    if p.sent.is_some() {
        let _ = git::try_run(root, &["push", "-q", "origin", "--delete", &p.branch]);
    }
    remove(&p.library, &p.id)
}
