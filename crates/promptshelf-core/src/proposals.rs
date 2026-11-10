//! Proposals: your edits to a team library, one prompt each, on its own branch
//! `shelf/<you>/<id>` and in its own git worktree. The library's main folder
//! never leaves its main branch, so one proposal can not hide another.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config;
use crate::error::{Error, Result};
use crate::forge::{self, Forge, Sent};
use crate::fsutil;
use crate::git;
use crate::prompt::{FOLDERS, slugify};
use crate::shelf::Library;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    /// Edits stay on this machine until you propose them.
    NotSent,
    /// Sent; the host has not said more, or there is no host login.
    Sent,
    ChangesRequested,
    /// Closed on the host without a merge. Your version stays until you choose.
    NotMerged,
    /// The branch is gone from the host and the change is not in main.
    Gone,
}

/// One proposal, as kept in `proposals.json` in the state folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proposal {
    pub library: String,
    /// The prompt ID inside the library.
    pub id: String,
    pub branch: String,
    pub base: String,
    pub status: Status,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sent: Option<Sent>,
    /// The commit that was last sent, to find edits made after it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_sent: Option<String>,
}

impl Proposal {
    pub fn full_id(&self) -> String {
        format!("{}/{}", self.library, self.id)
    }

    /// The worktree folder for this proposal.
    pub fn worktree(&self) -> PathBuf {
        worktree_dir(&self.library, &self.id)
    }
}

pub fn worktree_dir(library: &str, id: &str) -> PathBuf {
    config::state_dir().join("proposals").join(library).join(id)
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Records {
    proposals: Vec<Proposal>,
}

fn records_path() -> PathBuf {
    config::state_dir().join("proposals.json")
}

/// Every proposal on this machine.
pub fn load() -> Vec<Proposal> {
    std::fs::read(records_path())
        .ok()
        .and_then(|b| serde_json::from_slice::<Records>(&b).ok())
        .map(|r| r.proposals)
        .unwrap_or_default()
}

fn save(all: &[Proposal]) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(&Records {
        proposals: all.to_vec(),
    })
    .map_err(|e| Error::Invalid(e.to_string()))?;
    fsutil::write_atomic(&records_path(), &bytes)
}

/// Saves one proposal, replacing the record with the same library and ID.
pub fn put(p: &Proposal) -> Result<()> {
    let mut all = load();
    all.retain(|x| !(x.library == p.library && x.id == p.id));
    all.push(p.clone());
    save(&all)
}

pub fn remove(library: &str, id: &str) -> Result<()> {
    let mut all = load();
    all.retain(|x| !(x.library == library && x.id == id));
    save(&all)
}

pub fn find(library: &str, id: &str) -> Option<Proposal> {
    load()
        .into_iter()
        .find(|p| p.library == library && p.id == id)
}

/// Your name for branch names: git's user.name, else the login name.
fn user_slug(root: &Path) -> String {
    let name = git::run(root, &["config", "user.name"])
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("USER").ok())
        .or_else(|| std::env::var("USERNAME").ok())
        .unwrap_or_else(|| "me".into());
    slugify(&name)
}

/// The team's main branch, from `origin/HEAD`, else the main folder's branch.
pub fn base_branch(root: &Path) -> String {
    git::run(
        root,
        &["symbolic-ref", "--short", "refs/remotes/origin/HEAD"],
    )
    .ok()
    .and_then(|s| s.trim().strip_prefix("origin/").map(str::to_string))
    .or_else(|| git::branch(root))
    .unwrap_or_else(|| "main".into())
}

/// Starts a proposal for a prompt, or returns the open one. New prompts work too:
/// the file is created in the worktree.
pub fn start(lib: &Library, id: &str) -> Result<Proposal> {
    if let Some(p) = find(lib.name(), id) {
        if p.worktree().is_dir() {
            return Ok(p);
        }
    }
    let root = &lib.root;
    if !git::is_repo(root) {
        return Err(Error::Invalid(format!(
            "library \"{}\" is not a git repo",
            lib.name()
        )));
    }
    let _ = git::try_run(root, &["fetch", "-q", "origin"]);
    let base = base_branch(root);
    let branch = format!("shelf/{}/{id}", user_slug(root));
    let dir = worktree_dir(lib.name(), id);
    if let Some(parent) = dir.parent() {
        std::fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
    }
    let start_point = format!("origin/{base}");
    let has_remote_base = git::try_run(root, &["rev-parse", "--verify", "-q", &start_point])?.ok;
    let from = if has_remote_base { &start_point } else { &base };
    let dir_s = dir.display().to_string();
    let exists = git::try_run(root, &["rev-parse", "--verify", "-q", &branch])?.ok;
    if exists {
        git::run(root, &["worktree", "add", "-q", &dir_s, &branch])?;
    } else {
        git::run(
            root,
            &["worktree", "add", "-q", "-b", &branch, &dir_s, from],
        )?;
    }
    let p = Proposal {
        library: lib.name().to_string(),
        id: id.to_string(),
        branch,
        base,
        status: Status::NotSent,
        sent: None,
        last_sent: None,
    };
    put(&p)?;
    Ok(p)
}

/// The prompt's file in the proposal worktree: the same place as in the main
/// folder, or `templates/<id>.md` for a new prompt.
pub fn file_in_worktree(lib: &Library, p: &Proposal) -> PathBuf {
    let wt = p.worktree();
    if let Some(prompt) = lib.get(&p.id) {
        if let Ok(rel) = prompt.path.strip_prefix(&lib.root) {
            return wt.join(rel);
        }
    }
    for (folder, _) in FOLDERS {
        let path = wt.join(folder).join(format!("{}.md", p.id));
        if path.exists() {
            return path;
        }
    }
    wt.join("templates").join(format!("{}.md", p.id))
}

/// Proposes the edits: commits them, rebases on the team's main branch, pushes,
/// and opens the pull request the first time.
pub fn send(lib: &Library, p: &mut Proposal, title: &str, body: &str) -> Result<()> {
    let wt = p.worktree();
    crate::sync::checkpoint(&wt)?;
    let head = git::run(&wt, &["rev-parse", "HEAD"])?.trim().to_string();
    let base_ref = format!("origin/{}", p.base);
    if git::try_run(&wt, &["fetch", "-q", "origin"])?.ok
        && git::try_run(&wt, &["rev-parse", "--verify", "-q", &base_ref])?.ok
    {
        let mut args = crate::sync::identity(&wt)?;
        args.extend(["rebase", "-q", &base_ref]);
        if !git::try_run(&wt, &args)?.ok {
            let _ = git::try_run(&wt, &["rebase", "--abort"]);
            return Err(Error::Git(format!(
                "{} changed on {} too; resolve the conflict, then propose again",
                p.id, p.base
            )));
        }
    }
    let ahead = git::run(&wt, &["rev-list", "--count", &format!("{base_ref}..HEAD")])
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    if ahead == "0" && head == p.last_sent.clone().unwrap_or_default() {
        return Err(Error::Invalid(format!("{} has no new edits", p.full_id())));
    }
    let remote_url = git::run(&lib.root, &["remote", "get-url", "origin"])?
        .trim()
        .to_string();
    let forge = Forge::of(&lib.config, &remote_url);
    if p.sent.is_none() {
        let req = forge::Request {
            branch: &p.branch,
            base: &p.base,
            title,
            body,
        };
        p.sent = Some(forge::open(forge, &wt, &remote_url, &req)?);
    } else {
        // Your own proposal branch after a rebase: only it is ever force-pushed,
        // and only if nobody else pushed to it.
        git::run(
            &wt,
            &["push", "-q", "--force-with-lease", "origin", &p.branch],
        )?;
    }
    p.status = Status::Sent;
    p.last_sent = Some(git::run(&wt, &["rev-parse", "HEAD"])?.trim().to_string());
    put(p)
}

#[path = "proposals_check.rs"]
mod check;
pub use check::{Outcome, check, discard};
