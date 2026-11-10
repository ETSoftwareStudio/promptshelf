//! Team libraries and the Inbox, without Tauri: proposals, their states, and
//! the updates a pull brought in.

use promptshelf_core::inbox::{self, Change};
use promptshelf_core::proposals::{self, Proposal, Status};
use promptshelf_core::state::Allowed;
use promptshelf_core::{Error, Kind, ListFilter, Result, Shelf, git};

use crate::service::summary;
use crate::views::{InboxView, ProposalView, UpdateView};

/// True when the proposal has edits that were not sent yet.
fn unsent(p: &Proposal) -> bool {
    if p.status == Status::NotSent {
        return true;
    }
    let wt = p.worktree();
    let dirty = git::changed_paths(&wt).is_ok_and(|c| !c.is_empty());
    let head = git::run(&wt, &["rev-parse", "HEAD"]).map(|h| h.trim().to_string());
    dirty || head.ok() != p.last_sent
}

pub fn proposal_view(shelf: &Shelf, p: &Proposal) -> ProposalView {
    let full_id = p.full_id();
    ProposalView {
        title: shelf
            .resolve(&full_id)
            .map(|x| x.title.clone())
            .unwrap_or_else(|_| p.id.clone()),
        library: p.library.clone(),
        id: p.id.clone(),
        status: p.status,
        number: p.sent.as_ref().and_then(|s| s.number),
        url: p.sent.as_ref().and_then(|s| s.url.clone()),
        branch: p.branch.clone(),
        unsent: unsent(p),
        full_id,
    }
}

/// Your proposals in the libraries of this shelf. With `refresh`, asks the
/// host (or git) for each state first, and cleans up the merged ones.
pub fn proposals(shelf: &Shelf, refresh: bool) -> Vec<ProposalView> {
    let mut all = proposals::load();
    if refresh {
        for p in &mut all {
            if let Ok(lib) = shelf.library(&p.library) {
                let _ = proposals::check(lib, p);
            }
        }
        all = proposals::load();
    }
    all.iter()
        .filter(|p| shelf.library(&p.library).is_ok())
        .map(|p| proposal_view(shelf, p))
        .collect()
}

fn find(full_id: &str) -> Result<Proposal> {
    let (lib, id) = full_id
        .split_once('/')
        .ok_or_else(|| Error::Invalid(format!("\"{full_id}\" is not library/id")))?;
    proposals::find(lib, id).ok_or_else(|| Error::NotFound(format!("no proposal for {full_id}")))
}

/// The title and description the Propose sheet starts with.
pub fn proposal_text(shelf: &Shelf, full_id: &str) -> Result<(String, String)> {
    let p = find(full_id)?;
    Ok(proposals::default_text(shelf.library(&p.library)?, &p))
}

/// Sends a proposal: the pull request the first time, an update after that.
pub fn propose(shelf: &Shelf, full_id: &str, title: &str, body: &str) -> Result<ProposalView> {
    let mut p = find(full_id)?;
    let lib = shelf.library(&p.library)?;
    let (t, b) = proposals::default_text(lib, &p);
    let title = if title.trim().is_empty() { &t } else { title };
    let body = if body.trim().is_empty() { &b } else { body };
    proposals::send(lib, &mut p, title, body)?;
    Ok(proposal_view(shelf, &p))
}

pub fn withdraw(shelf: &Shelf, full_id: &str) -> Result<()> {
    let p = find(full_id)?;
    proposals::withdraw(shelf.library(&p.library)?, &p)
}

/// The Inbox: drafts in Personal, and updates pulled from team libraries.
pub fn inbox(shelf: &Shelf) -> Result<InboxView> {
    let pins = promptshelf_core::pins::load(shelf).unwrap_or_default();
    let usage = promptshelf_core::state::UsageLog::load();
    let drafts = match shelf.personal() {
        Some(p) => shelf
            .list(&ListFilter {
                library: Some(p.name().to_string()),
                kind: Some(Kind::Draft),
                tag: None,
            })?
            .into_iter()
            .map(|x| summary(x, &pins, &usage))
            .collect(),
        None => Vec::new(),
    };
    let allowed = Allowed::load().unwrap_or_default();
    let updates = inbox::load()
        .into_iter()
        .filter(|u| shelf.library(&u.library).is_ok())
        .rev()
        .map(|u| {
            let full_id = format!("{}/{}", u.library, u.id);
            UpdateView {
                title: shelf
                    .resolve(&full_id)
                    .map(|p| p.title.clone())
                    .unwrap_or_else(|_| u.id.clone()),
                allowed: u
                    .new_commands
                    .iter()
                    .filter(|c| allowed.allows(&full_id, c))
                    .cloned()
                    .collect(),
                full_id,
                update: u,
            }
        })
        .collect();
    Ok(InboxView { drafts, updates })
}

fn update(full_id: &str) -> Result<inbox::Update> {
    inbox::load()
        .into_iter()
        .find(|u| format!("{}/{}", u.library, u.id) == full_id)
        .ok_or_else(|| Error::NotFound(format!("no Inbox update for {full_id}")))
}

/// What a team update changed, as a unified diff.
pub fn update_diff(shelf: &Shelf, full_id: &str) -> Result<String> {
    let u = update(full_id)?;
    if u.change == Change::Removed {
        return Ok(String::new());
    }
    inbox::diff(&shelf.library(&u.library)?.root, &u)
}
