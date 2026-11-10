//! Setup: init, libraries and the Claude Code skill.

use std::path::PathBuf;

use promptshelf_core::config::{LibraryConfig, Writes, expand_tilde};
use promptshelf_core::{Config, Error, Result, fsutil, write};
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
    let created = config.personal().is_none();
    if created {
        config.libraries.push(LibraryConfig {
            name: "personal".into(),
            path: a.path,
            remote: None,
            writes: Writes::Commit,
            personal: true,
            trusted: None,
            forge: None,
            api: None,
        });
        config.save()?;
    }
    let personal = config.personal().expect("personal library exists");
    let root = personal.root();
    write::init_library(&root)?;
    if ctx.json {
        output::json(json!({
            "config": Config::path().display().to_string(),
            "library": { "name": personal.name, "path": root.display().to_string() },
            "created": created,
        }));
    } else if created {
        println!("Created {}", Config::path().display());
        println!("Personal library: {}", root.display());
    } else {
        println!(
            "Already set up. Personal library: {} ({})",
            personal.name,
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

pub(super) fn looks_like_git_url(s: &str) -> bool {
    s.contains("://") || (s.contains('@') && s.contains(':')) || s.ends_with(".git")
}

pub(super) fn libs_add(ctx: &Ctx, a: LibsAddArgs) -> Result<u8> {
    let mut config = match Config::load() {
        Ok(c) => c,
        Err(Error::NoConfig) => Config::default(),
        Err(e) => return Err(e),
    };
    ctx.caller.may_write(Some(&config), "Adding a library")?;
    if !promptshelf_core::prompt::is_valid_id(&a.name) {
        return Err(Error::Invalid(format!(
            "library name \"{}\" must use only a-z, 0-9 and -",
            a.name
        )));
    }
    if config.libraries.iter().any(|l| l.name == a.name) {
        return Err(Error::Invalid(format!(
            "a library named \"{}\" exists",
            a.name
        )));
    }
    let is_url = looks_like_git_url(&a.source);
    let (path, remote) = if is_url {
        let path = a
            .path
            .clone()
            .unwrap_or_else(|| format!("~/Prompts/{}", a.name));
        let root = expand_tilde(&path);
        if root.exists() {
            return Err(Error::Invalid(format!(
                "{} exists; pass --path for another folder",
                root.display()
            )));
        }
        let status = std::process::Command::new("git")
            .args(["clone", "--", &a.source])
            .arg(&root)
            .status()
            .map_err(|e| Error::Git(format!("could not run git: {e}")))?;
        if !status.success() {
            return Err(Error::Git(format!("git clone {} failed", a.source)));
        }
        (path, Some(a.source.clone()))
    } else {
        let root = expand_tilde(&a.source);
        if !root.is_dir() {
            return Err(Error::Invalid(format!(
                "{} is not a folder",
                root.display()
            )));
        }
        // Store an absolute path, so the library works from any folder.
        let path = if a.source.starts_with('~') {
            a.source.clone()
        } else {
            std::path::absolute(&root)
                .map_err(|e| Error::io(&root, e))?
                .display()
                .to_string()
        };
        (path, None)
    };
    let writes = match a.writes {
        Some(WritesArg::Commit) => Writes::Commit,
        Some(WritesArg::PullRequest) => Writes::PullRequest,
        None if is_url => Writes::PullRequest,
        None => Writes::Commit,
    };
    config.libraries.push(LibraryConfig {
        name: a.name.clone(),
        path: path.clone(),
        remote,
        writes,
        personal: false,
        trusted: None,
        forge: None,
        api: None,
    });
    config.save()?;
    if ctx.json {
        output::json(
            json!({ "library": { "name": a.name, "path": expand_tilde(&path).display().to_string(), "writes": writes } }),
        );
    } else {
        println!(
            "Added library {} ({})",
            a.name,
            expand_tilde(&path).display()
        );
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
                dirs::home_dir().ok_or_else(|| Error::Invalid("no home folder".into()))?
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
