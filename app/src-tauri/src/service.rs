//! What the app's read commands do, without Tauri, so tests can call it:
//! the overview, the list, a prompt, and filling it. Writes are in `writes.rs`.
//!
//! Every function takes the loaded [`Shelf`] and the pins. The Tauri layer in
//! `commands.rs` only loads state and turns errors into strings.

use std::collections::BTreeMap;

use promptshelf_core::config::Writes;
use promptshelf_core::pins::Pins;
use promptshelf_core::resolve::Options;
use promptshelf_core::shelf::Library;
use promptshelf_core::state::UsageLog;
use promptshelf_core::template::{self, LATE_BINDING, Value};
use promptshelf_core::{Config, Error, Kind, ListFilter, Prompt, Result, Shelf, fill};

pub use crate::views::*;

pub fn summary(p: &Prompt, pins: &Pins, usage: &UsageLog) -> Summary {
    let full_id = p.full_id();
    let used = usage.get(&full_id);
    Summary {
        pin: pins.position(&full_id),
        uses: used.map(|u| u.count).unwrap_or(0),
        last_used: used.and_then(|u| u.last_used.clone()),
        full_id,
        id: p.id.clone(),
        library: p.library.clone(),
        kind: p.kind,
        title: p.title.clone(),
        description: p.description.clone(),
        tags: p.tags.clone(),
    }
}

fn library_info(lib: &Library, personal: bool) -> LibraryInfo {
    let c = &lib.config;
    LibraryInfo {
        name: c.name.clone(),
        path: c.path.clone(),
        remote: c.remote.clone(),
        writes: match c.writes {
            Writes::Commit => "commit",
            Writes::PullRequest => "pull-request",
        },
        personal,
        trusted: c.is_trusted(),
        git: promptshelf_core::git::is_repo(&lib.root),
        exists: lib.root.is_dir(),
        count: lib.prompts.len(),
    }
}

pub fn overview(shelf: &Shelf, pins: &Pins) -> Overview {
    let personal = shelf.personal().map(|l| l.name().to_string());
    let libraries = shelf
        .libraries
        .iter()
        .map(|l| library_info(l, personal.as_deref() == Some(l.name())))
        .collect();
    let mut kinds: BTreeMap<Kind, usize> = BTreeMap::new();
    for p in shelf.prompts() {
        *kinds.entry(p.kind).or_default() += 1;
    }
    let pins = pins
        .pins
        .iter()
        .enumerate()
        .map(|(i, id)| {
            let found = shelf.resolve(id).ok();
            PinInfo {
                full_id: id.clone(),
                title: found.map(|p| p.title.clone()).unwrap_or_else(|| id.clone()),
                position: i + 1,
                exists: found.is_some(),
            }
        })
        .collect();
    Overview {
        config_path: Config::path().display().to_string(),
        has_config: true,
        personal,
        libraries,
        kinds: kinds
            .into_iter()
            .map(|(kind, count)| KindCount { kind, count })
            .collect(),
        total: shelf.prompts().count(),
        pins,
        diagnostics: shelf
            .diagnostics
            .iter()
            .map(|d| DiagnosticInfo {
                path: d.path.display().to_string(),
                message: d.message.clone(),
            })
            .collect(),
    }
}

/// The overview when there is no `config.toml` yet.
pub fn empty_overview() -> Overview {
    Overview {
        config_path: Config::path().display().to_string(),
        has_config: false,
        personal: None,
        libraries: Vec::new(),
        kinds: Vec::new(),
        total: 0,
        pins: Vec::new(),
        diagnostics: Vec::new(),
    }
}

/// The list column: search results for a query, else pins first and then the
/// rest, or the recently used prompts.
pub fn list(shelf: &Shelf, pins: &Pins, usage: &UsageLog, q: &Query) -> Result<Vec<Summary>> {
    let filter = ListFilter {
        library: q.library.clone(),
        kind: q.kind,
        tag: None,
    };
    let to_summary = |p: &Prompt| summary(p, pins, usage);
    if !q.text.trim().is_empty() {
        let mut out: Vec<Summary> = shelf
            .search(&q.text, &filter)?
            .into_iter()
            .map(|h| to_summary(h.prompt))
            .collect();
        if q.view == View::Recent {
            out.retain(|s| s.uses > 0);
        }
        return Ok(out);
    }
    let all: Vec<Summary> = shelf.list(&filter)?.into_iter().map(to_summary).collect();
    Ok(match q.view {
        View::All => {
            let (mut pinned, rest): (Vec<Summary>, Vec<Summary>) =
                all.into_iter().partition(|s| s.pin.is_some());
            pinned.sort_by_key(|s| s.pin);
            pinned.extend(rest);
            pinned
        }
        View::Recent => {
            let mut used: Vec<Summary> = all.into_iter().filter(|s| s.uses > 0).collect();
            used.sort_by(|a, b| {
                b.last_used
                    .cmp(&a.last_used)
                    .then(b.uses.cmp(&a.uses))
                    .then_with(|| a.full_id.cmp(&b.full_id))
            });
            used
        }
    })
}

pub fn detail(shelf: &Shelf, pins: &Pins, usage: &UsageLog, full_id: &str) -> Result<Detail> {
    let p = shelf.resolve(full_id)?;
    let file = std::fs::read_to_string(&p.path).map_err(|e| Error::io(&p.path, e))?;
    Ok(Detail {
        summary: summary(p, pins, usage),
        path: p.path.display().to_string(),
        body: p.body.clone(),
        vars: p.vars.clone(),
        forked_from: p.forked_from.clone(),
        file,
    })
}

/// Fills a prompt with the resolvers (they can run commands), then the values
/// the person typed. Empty typed values leave the blank open.
pub fn fill(
    shelf: &Shelf,
    full_id: &str,
    given: &BTreeMap<String, String>,
    opts: &Options,
) -> Result<FillView> {
    let p = shelf.resolve(full_id)?;
    let given: BTreeMap<String, String> = given
        .iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let rendered = fill::fill(shelf, p, &given, opts)?;
    let values = rendered
        .values
        .iter()
        .map(|f| (f.name.clone(), f.value.clone()))
        .collect();
    view(shelf, p, &values)
}

/// Renders a prompt with values the UI already has, without running any
/// resolver. The UI calls it while you type.
pub fn render(shelf: &Shelf, full_id: &str, values: &BTreeMap<String, Value>) -> Result<FillView> {
    let p = shelf.resolve(full_id)?;
    let values: BTreeMap<String, Value> = values
        .iter()
        .filter(|(_, v)| !v.value.is_empty())
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    view(shelf, p, &values)
}

fn view(shelf: &Shelf, p: &Prompt, values: &BTreeMap<String, Value>) -> Result<FillView> {
    let rendered = template::render(shelf, p, values)?;
    let pieces = template::pieces(shelf, p, values)?
        .into_iter()
        .map(PieceView::from)
        .collect();
    Ok(FillView {
        full_id: p.full_id(),
        pieces,
        send_text: send_text(&rendered.text, rendered.complete),
        text: rendered.text,
        complete: rendered.complete,
        values: rendered.values,
        missing: rendered.missing,
        specs: template::blanks(shelf, p)?.into_iter().collect(),
    })
}

/// The text Send copies. With open blanks, the late-binding instruction goes
/// after the prompt, so a model fills them from the chat.
pub fn send_text(text: &str, complete: bool) -> String {
    if complete {
        return text.to_string();
    }
    let mut s = text.trim_end().to_string();
    s.push_str("\n\n");
    s.push_str(LATE_BINDING);
    s.push('\n');
    s
}

/// Renders the values, records the use on this machine and returns the text
/// to copy.
pub fn prepare_send(
    shelf: &Shelf,
    full_id: &str,
    values: &BTreeMap<String, Value>,
    record: bool,
) -> Result<String> {
    let p = shelf.resolve(full_id)?;
    let values: BTreeMap<String, Value> = values
        .iter()
        .filter(|(_, v)| !v.value.is_empty())
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let rendered = template::render(shelf, p, &values)?;
    if record {
        fill::record_use(p, &rendered)?;
    }
    Ok(send_text(&rendered.text, rendered.complete))
}

#[cfg(test)]
#[path = "tests/service.rs"]
mod tests;
