//! Pins, sync and allowing commands.

use std::path::{Path, PathBuf};

use promptshelf_core::pins::{self, Pins};
use promptshelf_core::sync::{self as core_sync, Options};
use promptshelf_core::{Error, Result, Shelf, fill};
use serde_json::json;

use super::{Ctx, load};
use crate::output;
use crate::{IdArgs, PinArgs, SyncArgs, exit};

fn personal_root(shelf: &Shelf) -> Result<PathBuf> {
    shelf
        .personal()
        .map(|l| l.root.clone())
        .ok_or_else(|| Error::Invalid("no Personal library; run `shelf init`".into()))
}

fn print_pins(ctx: &Ctx, pins: &Pins) {
    if ctx.json {
        output::json(json!({ "pins": pins.pins }));
    } else if pins.pins.is_empty() {
        println!("No pins. Pin a prompt with `shelf pin <id>`.");
    } else {
        for (i, id) in pins.pins.iter().enumerate() {
            println!("{}  {id}", i + 1);
        }
    }
}

pub(super) fn pin(ctx: &Ctx, a: PinArgs) -> Result<u8> {
    let shelf = load()?;
    ctx.caller.may_write(Some(&shelf.config), "Pinning")?;
    let full = shelf.resolve(&a.id)?.full_id();
    let root = personal_root(&shelf)?;
    let mut pins = Pins::load(&root)?;
    pins.pin(&full, a.at.map(usize::from))?;
    pins.save(&root)?;
    print_pins(ctx, &pins);
    Ok(exit::OK)
}

pub(super) fn unpin(ctx: &Ctx, a: IdArgs) -> Result<u8> {
    let shelf = load()?;
    ctx.caller.may_write(Some(&shelf.config), "Unpinning")?;
    let root = personal_root(&shelf)?;
    let mut pins = Pins::load(&root)?;
    // Unpin works for a prompt that no longer exists, too.
    let full = shelf
        .resolve(&a.id)
        .map(|p| p.full_id())
        .unwrap_or_else(|_| a.id.clone());
    if !pins.unpin(&full) {
        return Err(Error::NotFound(format!("{} is not pinned", a.id)));
    }
    pins.save(&root)?;
    print_pins(ctx, &pins);
    Ok(exit::OK)
}

pub(super) fn pins(ctx: &Ctx) -> Result<u8> {
    let shelf = load()?;
    print_pins(ctx, &pins::load(&shelf)?);
    Ok(exit::OK)
}

/// The program git runs as the pin merge driver: `shelf` when it is on the
/// PATH (it survives upgrades), else this binary's path.
pub(super) fn driver_program() -> String {
    let name = if cfg!(windows) { "shelf.exe" } else { "shelf" };
    let on_path = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|d| d.join(name).is_file()))
        .unwrap_or(false);
    if on_path {
        return "shelf".into();
    }
    std::env::current_exe()
        .map(|p| p.display().to_string().replace('\\', "/"))
        .unwrap_or_else(|_| "shelf".into())
}

pub(super) fn sync(ctx: &Ctx, a: SyncArgs) -> Result<u8> {
    let shelf = load()?;
    // Agents only pull, unless the config lets them write.
    let may_write = ctx.caller.may_write(Some(&shelf.config), "Pushing").is_ok();
    let driver = driver_program();
    let opts = Options {
        commit: may_write,
        push: may_write && !a.no_push,
        merge_driver: &driver,
    };
    let personal = shelf.personal().map(|l| l.name().to_string());
    let mut reports = Vec::new();
    for lib in &shelf.libraries {
        if a.library.as_deref().is_some_and(|n| n != lib.name()) {
            continue;
        }
        let is_personal = personal.as_deref() == Some(lib.name());
        reports.push(core_sync::sync(lib, is_personal, &opts));
    }
    // Check proposals on team libraries and clean up the merged ones.
    let mut checked = Vec::new();
    if may_write {
        for mut p in promptshelf_core::proposals::load() {
            if a.library.as_deref().is_some_and(|n| n != p.library) {
                continue;
            }
            if let Ok(lib) = shelf.library(&p.library) {
                if let Ok(outcome) = promptshelf_core::proposals::check(lib, &mut p) {
                    checked.push(json!({ "proposal": p.full_id(), "outcome": outcome }));
                }
            }
        }
    }
    if let Some(name) = &a.library {
        if reports.is_empty() {
            return Err(Error::UnknownLibrary(name.clone()));
        }
    }
    let failed = reports
        .iter()
        .any(|r| r.error.is_some() || !r.conflicts.is_empty());
    if ctx.json {
        output::json(json!({ "libraries": reports, "proposals": checked }));
    } else {
        for r in &reports {
            let mut parts = Vec::new();
            if let Some(s) = &r.skipped {
                parts.push(s.clone());
            }
            if let Some(c) = &r.committed {
                parts.push(format!("committed \"{c}\""));
            }
            if r.pulled > 0 {
                parts.push(format!("pulled {}", r.pulled));
            }
            if r.updates > 0 {
                parts.push(format!("{} update(s) in the Inbox", r.updates));
            }
            if r.pushed > 0 {
                parts.push(format!("pushed {}", r.pushed));
            }
            if !r.conflicts.is_empty() {
                parts.push(format!(
                    "edited here and on the remote: {}. Your version stays; resolve it in the app",
                    r.conflicts.join(", ")
                ));
            }
            if let Some(e) = &r.error {
                parts.push(format!("error: {e}"));
            }
            if parts.is_empty() {
                parts.push("up to date".into());
            }
            println!("{}: {}", r.library, parts.join("; "));
        }
    }
    Ok(if failed { exit::ERROR } else { exit::OK })
}

pub(super) fn allow(ctx: &Ctx, a: IdArgs) -> Result<u8> {
    if ctx.caller.agent && !ctx.caller.user_action {
        return Err(Error::Refused(
            "Only a person can allow a prompt's commands.".into(),
        ));
    }
    let shelf = load()?;
    let p = shelf.resolve(&a.id)?;
    let commands = fill::allow(&shelf, p)?;
    if ctx.json {
        let list: Vec<_> = commands
            .iter()
            .map(|(owner, cmd)| json!({ "prompt": owner, "command": cmd }))
            .collect();
        output::json(json!({ "allowed": list }));
    } else if commands.is_empty() {
        println!("{} runs no commands.", p.full_id());
    } else {
        for (owner, cmd) in &commands {
            println!("Allowed for {owner}: {cmd}");
        }
    }
    Ok(exit::OK)
}

pub(super) fn merge_pins(base: &Path, ours: &Path, theirs: &Path) -> Result<u8> {
    let merged = pins::merge(
        &Pins::load_file(base)?,
        &Pins::load_file(ours)?,
        &Pins::load_file(theirs)?,
    );
    merged.save_file(ours)?;
    Ok(exit::OK)
}
