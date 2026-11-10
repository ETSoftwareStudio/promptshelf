//! The Inbox (drafts and team updates), and copying prompts to Personal.

use promptshelf_core::inbox::{self, Change};
use promptshelf_core::{Error, Kind, ListFilter, Result, fsutil, prompt::split_frontmatter};
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

/// Writes `text` into Personal's templates as a copy of `source`.
fn copy_to_personal(ctx: &Ctx, text: &str, source: &str, id: &str) -> Result<u8> {
    let shelf = load()?;
    let personal = shelf
        .personal()
        .ok_or_else(|| Error::Invalid("no Personal library; run `shelf init`".into()))?;
    let mut new_id = id.to_string();
    let mut n = 1;
    while personal.get(&new_id).is_some() {
        n += 1;
        new_id = format!("{id}-{n}");
    }
    let (yaml, body) = split_frontmatter(text);
    let mut fm = String::new();
    for line in yaml.unwrap_or("").lines() {
        if !line.starts_with("id:") && !line.starts_with("forked_from:") {
            fm.push_str(line);
            fm.push('\n');
        }
    }
    let out = format!("---\nid: {new_id}\nforked_from: {source}\n{fm}---\n{body}");
    let path = personal.root.join("templates").join(format!("{new_id}.md"));
    if !fsutil::create_new(&path, out.as_bytes())? {
        return Err(Error::Invalid(format!("{} exists", path.display())));
    }
    let full = format!("{}/{new_id}", personal.name());
    if ctx.json {
        output::json(
            json!({ "id": full, "path": path.display().to_string(), "forked_from": source }),
        );
    } else {
        println!("Copied {source} to {full}.");
    }
    Ok(exit::OK)
}

pub(super) fn copy(ctx: &Ctx, a: IdArgs) -> Result<u8> {
    let shelf = load()?;
    ctx.caller
        .may_write(Some(&shelf.config), "Copying a prompt")?;
    let p = shelf.resolve(&a.id)?;
    let text = std::fs::read_to_string(&p.path).map_err(|e| Error::io(&p.path, e))?;
    copy_to_personal(ctx, &text, &p.full_id(), &p.id)
}

pub(super) fn keep_previous(ctx: &Ctx, a: IdArgs) -> Result<u8> {
    let shelf = load()?;
    ctx.caller
        .may_write(Some(&shelf.config), "Keeping a previous version")?;
    let update = inbox::load()
        .into_iter()
        .find(|u| format!("{}/{}", u.library, u.id) == a.id || u.id == a.id)
        .ok_or_else(|| Error::NotFound(format!("no Inbox update for {}", a.id)))?;
    if update.change == Change::Added {
        return Err(Error::Invalid(format!(
            "{} is new; it has no previous version",
            a.id
        )));
    }
    let lib = shelf.library(&update.library)?;
    let text = inbox::previous_text(&lib.root, &update)?;
    let source = format!(
        "{}/{}@{}",
        update.library,
        update.id,
        &update.from[..update.from.len().min(7)]
    );
    copy_to_personal(ctx, &text, &source, &update.id)
}
