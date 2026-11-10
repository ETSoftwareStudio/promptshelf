//! Reading: list, search, show and check.

use promptshelf_core::{Result, template};
use serde_json::json;

use super::{Ctx, describe_spec, filter, load};
use crate::output::{self, Summary};
use crate::{FilterArgs, IdArgs, exit};

pub(super) fn list(ctx: &Ctx, a: FilterArgs) -> Result<u8> {
    let shelf = load()?;
    let prompts = shelf.list(&filter(a))?;
    let rows: Vec<Summary> = prompts
        .iter()
        .map(|p| Summary::new(&shelf, p, None))
        .collect();
    if ctx.json {
        output::json(json!({
            "prompts": rows,
            "diagnostics": output::diagnostics_json(&shelf.diagnostics),
        }));
    } else {
        output::warn_diagnostics(&shelf.diagnostics);
        if rows.is_empty() {
            eprintln!("No prompts. Add one with `shelf new` or `shelf save`.");
        }
        output::table(&rows);
    }
    Ok(exit::OK)
}

pub(super) fn search(ctx: &Ctx, query: &str, a: FilterArgs) -> Result<u8> {
    let shelf = load()?;
    let hits = shelf.search(query, &filter(a))?;
    let rows: Vec<Summary> = hits
        .iter()
        .map(|h| Summary::new(&shelf, h.prompt, Some(h.score)))
        .collect();
    if ctx.json {
        output::json(json!({
            "query": query,
            "prompts": rows,
            "diagnostics": output::diagnostics_json(&shelf.diagnostics),
        }));
    } else {
        output::warn_diagnostics(&shelf.diagnostics);
        if rows.is_empty() {
            eprintln!("No prompts match \"{query}\".");
        }
        output::table(&rows);
    }
    Ok(exit::OK)
}

pub(super) fn show(ctx: &Ctx, a: IdArgs) -> Result<u8> {
    let shelf = load()?;
    let p = shelf.resolve(&a.id)?;
    let blanks = template::blanks(&shelf, p)?;
    if ctx.json {
        let mut prompt = serde_json::to_value(p).unwrap_or_default();
        prompt["id"] = json!(p.full_id());
        prompt["path"] = json!(p.path.display().to_string());
        prompt["blanks"] = json!(
            blanks
                .iter()
                .map(|(name, spec)| {
                    let mut v = serde_json::to_value(spec).unwrap_or_default();
                    v["name"] = json!(name);
                    v
                })
                .collect::<Vec<_>>()
        );
        output::json(json!({ "prompt": prompt }));
    } else {
        println!("{}  ({})", p.full_id(), p.kind.as_str());
        println!("{}", p.title);
        if let Some(d) = &p.description {
            println!("{d}");
        }
        if !p.tags.is_empty() {
            println!("tags: {}", p.tags.join(", "));
        }
        for (name, spec) in &blanks {
            println!("blank: {name}{}", describe_spec(spec));
        }
        println!("path: {}", p.path.display());
        println!();
        print!("{}", p.body);
    }
    Ok(exit::OK)
}

pub(super) fn check(ctx: &Ctx) -> Result<u8> {
    let shelf = load()?;
    let ok = shelf.diagnostics.is_empty();
    if ctx.json {
        output::json(json!({
            "ok": ok,
            "prompts": shelf.prompts().count(),
            "diagnostics": output::diagnostics_json(&shelf.diagnostics),
        }));
    } else if ok {
        println!("OK: {} prompts", shelf.prompts().count());
    } else {
        output::warn_diagnostics(&shelf.diagnostics);
    }
    Ok(if ok { exit::OK } else { exit::ERROR })
}
