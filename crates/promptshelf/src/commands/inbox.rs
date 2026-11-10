//! The Inbox (drafts and team updates), and copying prompts to Personal.

use promptshelf_core::inbox::{self, Change};
use promptshelf_core::write::{Copied, copy_prompt};
use promptshelf_core::{Kind, ListFilter, Result};
use serde_json::json;

use super::{Ctx, load};
use crate::output::{self, Summary};
use crate::{IdArgs, InboxArgs, exit};

pub(super) fn inbox(ctx: &Ctx, a: InboxArgs) -> Result<u8> {
    let shelf = load()?;
    if a.done || a.done_id.is_some() {
        ctx.caller
            .may_write(Some(&shelf.config), "Clearing the Inbox")?;
        let n = inbox::clear(a.done_id.as_deref())?;
        if ctx.json {
            output::json(json!({ "cleared": n }));
        } else {
            println!("Cleared {n} update(s).");
        }
        return Ok(exit::OK);
    }
    let pins = promptshelf_core::pins::load(&shelf).unwrap_or_default();
    let drafts: Vec<Summary> = match shelf.personal() {
        Some(p) => shelf
            .list(&ListFilter {
                library: Some(p.name().to_string()),
                kind: Some(Kind::Draft),
                tag: None,
            })?
            .into_iter()
            .map(|x| Summary::new(&shelf, x, None, &pins))
            .collect(),
        None => Vec::new(),
    };
    let updates = inbox::load();
    if ctx.json {
        output::json(json!({ "drafts": drafts, "updates": updates }));
        return Ok(exit::OK);
    }
    if drafts.is_empty() && updates.is_empty() {
        println!("The Inbox is empty.");
    }
    if !drafts.is_empty() {
        println!("Drafts:");
        output::table(&drafts);
    }
    if !updates.is_empty() {
        println!("Team updates:");
        for u in &updates {
            let what = match u.change {
                Change::Added => "added",
                Change::Edited => "edited",
                Change::Removed => "removed",
            };
            println!("  {}/{}  {what}  {}", u.library, u.id, u.date);
            for cmd in &u.new_commands {
                println!("    new command, off until allowed: {cmd}");
            }
        }
        println!(
            "Keep the old text with `shelf keep-previous <id>`. Clear with `shelf inbox --done`."
        );
    }
    Ok(exit::OK)
}

fn print_copy(ctx: &Ctx, c: &Copied) -> Result<u8> {
    if ctx.json {
        output::json(
            json!({ "id": c.id, "path": c.path.display().to_string(), "forked_from": c.forked_from }),
        );
    } else {
        println!("Copied {} to {}.", c.forked_from, c.id);
    }
    Ok(exit::OK)
}

pub(super) fn copy(ctx: &Ctx, a: IdArgs) -> Result<u8> {
    let shelf = load()?;
    ctx.caller
        .may_write(Some(&shelf.config), "Copying a prompt")?;
    print_copy(ctx, &copy_prompt(&shelf, &a.id)?)
}

pub(super) fn keep_previous(ctx: &Ctx, a: IdArgs) -> Result<u8> {
    let shelf = load()?;
    ctx.caller
        .may_write(Some(&shelf.config), "Keeping a previous version")?;
    print_copy(ctx, &inbox::keep_previous(&shelf, &a.id)?)
}
