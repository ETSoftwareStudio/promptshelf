//! Setup: init, libraries and the Claude Code skill.

use std::path::PathBuf;

use promptshelf_core::config::{Writes, expand_tilde};
use promptshelf_core::{Config, Error, Result, fsutil, setup};
use serde_json::json;

use super::{Ctx, load};
use crate::output;
use crate::skill::SKILL_MD;
use crate::{InitArgs, LibsAddArgs, SkillCommand, WritesArg, exit};

pub(super) fn init(ctx: &Ctx, a: InitArgs) -> Result<u8> {
    let existing = match Config::load() {
        Ok(c) => Some(c),
        Err(Error::NoConfig) => None,
        Err(e) => return Err(e),
    };
    ctx.caller
        .may_write(existing.as_ref(), "Creating a library")?;
    let mut config = existing.unwrap_or_default();
    let init = setup::init_personal(&mut config, &a.path, &super::sync::driver_program())?;
    if init.created {
        config.save()?;
    }
    let (created, root) = (init.created, init.root);
    if ctx.json {
        output::json(json!({
            "config": Config::path().display().to_string(),
            "library": { "name": init.name, "path": root.display().to_string() },
            "created": created,
        }));
    } else if created {
        println!("Created {}", Config::path().display());
        println!("Personal library: {}", root.display());
    } else {
        println!(
            "Already set up. Personal library: {} ({})",
            init.name,
            root.display()
        );
    }
    Ok(exit::OK)
}

pub(super) fn libs_list(ctx: &Ctx) -> Result<u8> {
    let shelf = load()?;
    let personal = shelf.personal().map(|l| l.name().to_string());
    let rows: Vec<_> = shelf
        .libraries
        .iter()
        .map(|l| {
            json!({
                "name": l.name(),
                "path": l.root.display().to_string(),
                "remote": l.config.remote,
                "writes": l.config.writes,
                "personal": Some(l.name()) == personal.as_deref(),
                "trusted": l.config.is_trusted(),
                "prompts": l.prompts.len(),
            })
        })
        .collect();
    if ctx.json {
        output::json(json!({ "libraries": rows }));
    } else {
        for l in &shelf.libraries {
            let mark = if Some(l.name()) == personal.as_deref() {
                " (personal)"
            } else {
                ""
            };
            println!(
                "{}{}  {} prompts  {}",
                l.name(),
                mark,
                l.prompts.len(),
                l.root.display()
            );
        }
    }
    Ok(exit::OK)
}

pub(super) fn libs_add(ctx: &Ctx, a: LibsAddArgs) -> Result<u8> {
    let mut config = match Config::load() {
        Ok(c) => c,
        Err(Error::NoConfig) => Config::default(),
        Err(e) => return Err(e),
    };
    ctx.caller.may_write(Some(&config), "Adding a library")?;
    let lib = setup::add_library(
        &mut config,
        &setup::AddLibrary {
            name: &a.name,
            source: &a.source,
            path: a.path.as_deref(),
            writes: a.writes.map(|w| match w {
                WritesArg::Commit => Writes::Commit,
                WritesArg::PullRequest => Writes::PullRequest,
            }),
            trusted: None,
            show_progress: true,
        },
    )?;
    config.save()?;
    let root = expand_tilde(&lib.path);
    if ctx.json {
        output::json(
            json!({ "library": { "name": lib.name, "path": root.display().to_string(), "writes": lib.writes } }),
        );
    } else {
        println!("Added library {} ({})", lib.name, root.display());
    }
    Ok(exit::OK)
}

pub(super) fn skill(ctx: &Ctx, cmd: SkillCommand) -> Result<u8> {
    match cmd {
        SkillCommand::Print => {
            print!("{SKILL_MD}");
            Ok(exit::OK)
        }
        SkillCommand::Install { project } => {
            let config = Config::load().ok();
            ctx.caller
                .may_write(config.as_ref(), "Installing the skill")?;
            let base = if project {
                std::env::current_dir().map_err(|e| Error::Invalid(e.to_string()))?
            } else {
                promptshelf_core::config::home_dir()
                    .ok_or_else(|| Error::Invalid("no home folder".into()))?
            };
            let path: PathBuf = base
                .join(".claude")
                .join("skills")
                .join("promptshelf")
                .join("SKILL.md");
            fsutil::write_atomic(&path, SKILL_MD.as_bytes())?;
            if ctx.json {
                output::json(json!({ "path": path.display().to_string() }));
            } else {
                println!("Installed {}", path.display());
            }
            Ok(exit::OK)
        }
    }
}
