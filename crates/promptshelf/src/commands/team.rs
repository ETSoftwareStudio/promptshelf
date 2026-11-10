//! Team libraries: edits become proposals (pull requests), one per prompt.

use std::path::PathBuf;

use promptshelf_core::config::Writes;
use promptshelf_core::proposals::{self, Proposal, Status};
use promptshelf_core::shelf::Library;
use promptshelf_core::write::{self, NewPrompt};
use promptshelf_core::{Error, Result, prompt::slugify};
use serde_json::json;

use super::{Ctx, load};
use crate::output;
use crate::{IdArgs, ProposeArgs, exit};

/// Creates a prompt file. In a team library it goes into a new proposal.
pub(super) fn create_in(library: &Library, new: &NewPrompt) -> Result<(String, PathBuf)> {
    if library.config.writes != Writes::PullRequest {
        return write::create_prompt(library, new);
    }
    let base = slugify(new.title);
    let mut id = base.clone();
    let mut n = 1;
    while library.get(&id).is_some() || proposals::find(library.name(), &id).is_some() {
        n += 1;
        id = format!("{base}-{n}");
    }
    let p = proposals::start(library, &id)?;
    let mut in_worktree = library.clone();
    in_worktree.root = p.worktree();
    in_worktree.prompts.clear();
    let title = if id == base {
        new.title.to_string()
    } else {
        format!("{} {n}", new.title)
    };
    write::create_prompt(
        &in_worktree,
        &NewPrompt {
            title: &title,
            ..new.clone()
        },
    )
}

pub(super) fn edit(ctx: &Ctx, a: IdArgs) -> Result<u8> {
    let shelf = load()?;
    ctx.caller
        .may_write(Some(&shelf.config), "Editing a prompt")?;
    let p = shelf.resolve(&a.id)?;
    let lib = shelf.library(&p.library)?;
    let path = if lib.config.writes == Writes::PullRequest {
        let prop = proposals::start(lib, &p.id)?;
        proposals::file_in_worktree(lib, &prop)
    } else {
        p.path.clone()
    };
    if ctx.json {
        output::json(json!({ "id": p.full_id(), "path": path.display().to_string() }));
        return Ok(exit::OK);
    }
    super::save::open_editor(&path)?;
    if lib.config.writes == Writes::PullRequest {
        println!(
            "Saved your version of {}. Send it with `shelf propose {}`.",
            p.full_id(),
            p.full_id()
        );
    }
    Ok(exit::OK)
}

fn find_proposal(shelf: &promptshelf_core::Shelf, id: &str) -> Result<(Proposal, String)> {
    let (lib, local) = match id.split_once('/') {
        Some((l, i)) => (l.to_string(), i.to_string()),
        None => {
            let matches: Vec<Proposal> = proposals::load()
                .into_iter()
                .filter(|p| p.id == id)
                .collect();
            match matches.as_slice() {
                [one] => (one.library.clone(), one.id.clone()),
                [] => return Err(Error::NotFound(format!("no proposal for {id}"))),
                many => {
                    return Err(Error::Ambiguous {
                        id: id.to_string(),
                        matches: many.iter().map(Proposal::full_id).collect(),
                    });
                }
            }
        }
    };
    shelf.library(&lib)?;
    let p = proposals::find(&lib, &local)
        .ok_or_else(|| Error::NotFound(format!("no proposal for {lib}/{local}")))?;
    Ok((p, lib))
}

pub(super) fn propose(ctx: &Ctx, a: ProposeArgs) -> Result<u8> {
    let shelf = load()?;
    ctx.caller.may_write(Some(&shelf.config), "Proposing")?;
    let (mut p, lib_name) = find_proposal(&shelf, &a.id)?;
    let lib = shelf.library(&lib_name)?;
    let title = a.title.unwrap_or_else(|| {
        let verb = if lib.get(&p.id).is_some() {
            "Edit"
        } else {
            "Add"
        };
        format!("{verb} {}", p.id)
    });
    let body = a
        .body
        .unwrap_or_else(|| format!("Proposed with PromptShelf: `{}`.", p.full_id()));
    proposals::send(lib, &mut p, &title, &body)?;
    print_one(ctx, &p);
    Ok(exit::OK)
}

fn describe(p: &Proposal) -> String {
    let number = p
        .sent
        .as_ref()
        .and_then(|s| s.number)
        .map(|n| format!("#{n} "))
        .unwrap_or_default();
    match p.status {
        Status::NotSent => "Not sent".into(),
        Status::Sent => format!("{number}Waiting for review"),
        Status::ChangesRequested => format!("{number}Changes requested"),
        Status::NotMerged => format!("{number}Not merged"),
        Status::Gone => "Closed or merged with edits; check on the host".into(),
    }
}

fn print_one(ctx: &Ctx, p: &Proposal) {
    if ctx.json {
        output::json(json!({ "proposal": p }));
    } else {
        let url = p
            .sent
            .as_ref()
            .and_then(|s| s.url.clone())
            .unwrap_or_default();
        println!("{}  {}  {url}", p.full_id(), describe(p));
    }
}

pub(super) fn list(ctx: &Ctx, check: bool) -> Result<u8> {
    let shelf = load()?;
    let mut all = proposals::load();
    if check {
        for p in &mut all {
            if let Ok(lib) = shelf.library(&p.library) {
                proposals::check(lib, p)?;
            }
        }
        all = proposals::load();
    }
    if ctx.json {
        output::json(json!({ "proposals": all }));
    } else if all.is_empty() {
        println!("No proposals. Edit a team prompt with `shelf edit <id>`.");
    } else {
        for p in &all {
            let url = p
                .sent
                .as_ref()
                .and_then(|s| s.url.clone())
                .unwrap_or_default();
            println!("{}  {}  {url}", p.full_id(), describe(p));
        }
    }
    Ok(exit::OK)
}

pub(super) fn withdraw(ctx: &Ctx, a: IdArgs) -> Result<u8> {
    let shelf = load()?;
    ctx.caller
        .may_write(Some(&shelf.config), "Withdrawing a proposal")?;
    let (p, lib_name) = find_proposal(&shelf, &a.id)?;
    let lib = shelf.library(&lib_name)?;
    if let Some(number) = p.sent.as_ref().and_then(|s| s.number) {
        let remote = promptshelf_core::git::run(&lib.root, &["remote", "get-url", "origin"])?;
        if let Some(repo) = promptshelf_core::forge::repo_path(remote.trim()) {
            let _ = std::process::Command::new("gh")
                .args(["pr", "close", &number.to_string(), "--repo", &repo])
                .stdin(std::process::Stdio::null())
                .output();
        }
    }
    proposals::discard(lib, &p)?;
    if ctx.json {
        output::json(json!({ "withdrawn": p.full_id() }));
    } else {
        println!("Withdrew {}.", p.full_id());
    }
    Ok(exit::OK)
}
