//! Writing prompts: save from stdin, and new in $EDITOR.

use std::io::Read;

use promptshelf_core::write::{self, NewPrompt};
use promptshelf_core::{Error, Kind, Result};
use serde_json::json;

use super::{Ctx, load};
use crate::output;
use crate::{NewArgs, SaveArgs, exit};

pub(super) fn read_stdin() -> Result<String> {
    let mut text = String::new();
    std::io::stdin()
        .read_to_string(&mut text)
        .map_err(|e| Error::Invalid(format!("could not read stdin: {e}")))?;
    if text.trim().is_empty() {
        return Err(Error::Invalid(
            "nothing to save: pipe the prompt text on stdin".into(),
        ));
    }
    Ok(text)
}

pub(super) fn save(ctx: &Ctx, a: SaveArgs) -> Result<u8> {
    let shelf = load()?;
    let (library, kind) = match &a.library {
        Some(name) => {
            ctx.caller
                .may_write(Some(&shelf.config), "Saving straight into a library")?;
            (shelf.library(name)?, Kind::from(a.kind))
        }
        None => (
            shelf
                .personal()
                .ok_or_else(|| Error::Invalid("no Personal library; run `shelf init`".into()))?,
            Kind::Draft,
        ),
    };
    let body = read_stdin()?;
    let (id, path) = write::create_prompt(
        library,
        &NewPrompt {
            title: &a.title,
            kind,
            body: &body,
            tags: &a.tag,
        },
    )?;
    let full = format!("{}/{}", library.name(), id);
    if ctx.json {
        output::json(json!({
            "id": full,
            "kind": kind.as_str(),
            "path": path.display().to_string(),
        }));
    } else {
        println!("Saved {full} ({})", path.display());
    }
    Ok(exit::OK)
}

pub(super) fn new(ctx: &Ctx, a: NewArgs) -> Result<u8> {
    let shelf = load()?;
    ctx.caller
        .may_write(Some(&shelf.config), "Creating a prompt")?;
    let library = match &a.library {
        Some(name) => shelf.library(name)?,
        None => shelf
            .personal()
            .ok_or_else(|| Error::Invalid("no Personal library; run `shelf init`".into()))?,
    };
    let title = a.title.as_deref().unwrap_or("Untitled");
    let (id, path) = write::create_prompt(
        library,
        &NewPrompt {
            title,
            kind: a.kind.into(),
            body: "Write the prompt here. Use {{name}} for a blank.\n",
            tags: &[],
        },
    )?;
    let full = format!("{}/{}", library.name(), id);
    if ctx.json || ctx.caller.agent {
        output::json(json!({ "id": full, "path": path.display().to_string() }));
        return Ok(exit::OK);
    }
    open_editor(&path)?;
    println!("Saved {full} ({})", path.display());
    Ok(exit::OK)
}

pub(super) fn open_editor(path: &std::path::Path) -> Result<()> {
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| if cfg!(windows) { "notepad" } else { "vi" }.into());
    let mut parts = editor.split_whitespace();
    let program = parts.next().unwrap_or("vi");
    let status = std::process::Command::new(program)
        .args(parts)
        .arg(path)
        .status()
        .map_err(|e| Error::Invalid(format!("could not start the editor \"{editor}\": {e}")))?;
    if !status.success() {
        return Err(Error::Invalid(format!("the editor \"{editor}\" failed")));
    }
    Ok(())
}
