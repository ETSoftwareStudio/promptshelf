//! Templates: `{{var}}` blanks and `{{> include}}` partials. No logic.

use std::collections::{BTreeMap, HashMap};

use serde::Serialize;

use crate::error::{Error, Result};
use crate::prompt::{Prompt, VarSpec};
use crate::shelf::Shelf;

const MAX_DEPTH: usize = 16;

/// The instruction that goes with a prompt whose blanks are still open, so a model
/// fills them from the conversation (late binding).
pub const LATE_BINDING: &str = "Some blanks in this prompt are not filled. Fill each blank from the conversation so far. Ask the user only about values you can not work out.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    Text(String),
    Var(String),
    Include(String),
}

/// Splits a template into text, blanks and includes. Anything between `{{ }}` that
/// is not a blank name or an include stays as text.
pub fn parse(template: &str) -> Vec<Segment> {
    let mut out = Vec::new();
    let mut text = String::new();
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else {
            break;
        };
        let inner = after[..end].trim();
        let segment = if let Some(name) = inner.strip_prefix('>') {
            let name = name.trim();
            (!name.is_empty() && !name.contains(char::is_whitespace))
                .then(|| Segment::Include(name.to_string()))
        } else if is_var_name(inner) {
            Some(Segment::Var(inner.to_string()))
        } else {
            None
        };
        match segment {
            Some(seg) => {
                text.push_str(&rest[..start]);
                if !text.is_empty() {
                    out.push(Segment::Text(std::mem::take(&mut text)));
                }
                out.push(seg);
            }
            None => text.push_str(&rest[..start + 2 + end + 2]),
        }
        rest = &after[end + 2..];
    }
    text.push_str(rest);
    if !text.is_empty() {
        out.push(Segment::Text(text));
    }
    out
}

pub fn is_var_name(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// Where a value came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Source {
    /// Given on the command line or by the caller.
    Given,
    Default,
    LastUsed,
    Branch,
    Clipboard,
    Selection,
    Command,
    Conversation,
}

/// A value with its source, for the footnotes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Value {
    pub value: String,
    pub source: Source,
    /// Detail for the footnote, such as the command that ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl Value {
    pub fn given(value: impl Into<String>) -> Self {
        Value {
            value: value.into(),
            source: Source::Given,
            detail: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Filled {
    pub name: String,
    #[serde(flatten)]
    pub value: Value,
}

/// A blank the core could not fill, with what the caller needs to fill it.
#[derive(Debug, Clone, Serialize)]
pub struct Missing {
    pub name: String,
    #[serde(flatten)]
    pub spec: VarSpec,
    /// Why the core could not fill it, when a resolver tried and failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Rendered {
    /// The text with every filled blank replaced. Open blanks stay as `{{name}}`.
    pub text: String,
    pub complete: bool,
    pub values: Vec<Filled>,
    pub missing: Vec<Missing>,
}

/// Every blank a prompt uses, its own and its includes', in the order they first
/// appear, with the spec that applies. The outer prompt's spec wins.
pub fn blanks(shelf: &Shelf, prompt: &Prompt) -> Result<Vec<(String, VarSpec)>> {
    Ok(blanks_owned(shelf, prompt)?
        .into_iter()
        .map(|b| (b.name, b.spec))
        .collect())
}

/// A blank with the prompt that declares its spec.
#[derive(Debug, Clone)]
pub struct Blank {
    pub name: String,
    pub spec: VarSpec,
    /// The full ID of the prompt whose `vars:` gave the spec. Shell commands are
    /// checked against this prompt's library.
    pub owner: String,
}

/// Like [`blanks`], with the prompt that declares each spec.
pub fn blanks_owned(shelf: &Shelf, prompt: &Prompt) -> Result<Vec<Blank>> {
    let mut order: Vec<String> = Vec::new();
    let mut specs: HashMap<String, (VarSpec, String)> = HashMap::new();
    collect(shelf, prompt, &mut order, &mut specs, &mut Vec::new())?;
    for (name, spec) in &prompt.vars {
        specs.insert(name.clone(), (spec.clone(), prompt.full_id()));
        if !order.contains(name) {
            order.push(name.clone());
        }
    }
    Ok(order
        .into_iter()
        .map(|name| {
            let (spec, owner) = specs
                .remove(&name)
                .unwrap_or_else(|| (VarSpec::default(), prompt.full_id()));
            Blank { name, spec, owner }
        })
        .collect())
}

fn collect(
    shelf: &Shelf,
    prompt: &Prompt,
    order: &mut Vec<String>,
    specs: &mut HashMap<String, (VarSpec, String)>,
    stack: &mut Vec<String>,
) -> Result<()> {
    enter(prompt, stack)?;
    for seg in parse(&prompt.body) {
        match seg {
            Segment::Var(name) => {
                if !order.contains(&name) {
                    order.push(name.clone());
                }
                if let Some(spec) = prompt.vars.get(&name) {
                    specs
                        .entry(name)
                        .or_insert_with(|| (spec.clone(), prompt.full_id()));
                }
            }
            Segment::Include(id) => {
                let inner = include(shelf, prompt, &id)?;
                collect(shelf, inner, order, specs, stack)?;
            }
            Segment::Text(_) => {}
        }
    }
    stack.pop();
    Ok(())
}

fn enter(prompt: &Prompt, stack: &mut Vec<String>) -> Result<()> {
    let id = prompt.full_id();
    if stack.contains(&id) {
        stack.push(id);
        return Err(Error::Template(format!(
            "include loop: {}",
            stack.join(" → ")
        )));
    }
    if stack.len() >= MAX_DEPTH {
        return Err(Error::Template(format!(
            "includes nest deeper than {MAX_DEPTH} levels"
        )));
    }
    stack.push(id);
    Ok(())
}

fn include<'a>(shelf: &'a Shelf, from: &Prompt, id: &str) -> Result<&'a Prompt> {
    shelf.resolve_from(&from.library, id).map_err(|e| match e {
        Error::NotFound(_) | Error::UnknownLibrary(_) => Error::Template(format!(
            "{} includes \"{id}\", which does not exist",
            from.full_id()
        )),
        other => other,
    })
}

/// Fills a prompt with the values that are known. Blanks without a value stay
/// open and are listed in `missing`.
pub fn render(
    shelf: &Shelf,
    prompt: &Prompt,
    values: &BTreeMap<String, Value>,
) -> Result<Rendered> {
    let specs = blanks(shelf, prompt)?;
    for (name, value) in values {
        if let Some((_, spec)) = specs.iter().find(|(n, _)| n == name) {
            if value.source == Source::Given
                && !spec.choices.is_empty()
                && !spec.choices.contains(&value.value)
            {
                return Err(Error::Invalid(format!(
                    "\"{}\" is not a choice for {name}; choose one of: {}",
                    value.value,
                    spec.choices.join(", ")
                )));
            }
        }
    }
    let text = to_text(&pieces(shelf, prompt, values)?);
    let mut filled = Vec::new();
    let mut missing = Vec::new();
    for (name, spec) in specs {
        match values.get(&name) {
            Some(v) => filled.push(Filled {
                name,
                value: v.clone(),
            }),
            None => missing.push(Missing {
                name,
                spec,
                reason: None,
            }),
        }
    }
    Ok(Rendered {
        text,
        complete: missing.is_empty(),
        values: filled,
        missing,
    })
}

/// A piece of filled text: plain text, a filled value, or an open blank.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Piece {
    Text(String),
    Value { name: String, value: Value },
    Open { name: String },
}

impl Piece {
    fn push_text(pieces: &mut Vec<Piece>, s: &str) {
        if let Some(Piece::Text(last)) = pieces.last_mut() {
            last.push_str(s);
        } else if !s.is_empty() {
            pieces.push(Piece::Text(s.to_string()));
        }
    }
}

/// The filled text as pieces, so a UI can colour each value by its source.
pub fn pieces(
    shelf: &Shelf,
    prompt: &Prompt,
    values: &BTreeMap<String, Value>,
) -> Result<Vec<Piece>> {
    let mut out = Vec::new();
    write(shelf, prompt, values, &mut out, &mut Vec::new())?;
    Ok(out)
}

/// Joins pieces into text. Open blanks stay as `{{name}}`.
pub fn to_text(pieces: &[Piece]) -> String {
    let mut s = String::new();
    for p in pieces {
        match p {
            Piece::Text(t) => s.push_str(t),
            Piece::Value { value, .. } => s.push_str(&value.value),
            Piece::Open { name } => {
                s.push_str("{{");
                s.push_str(name);
                s.push_str("}}");
            }
        }
    }
    s
}

fn write(
    shelf: &Shelf,
    prompt: &Prompt,
    values: &BTreeMap<String, Value>,
    out: &mut Vec<Piece>,
    stack: &mut Vec<String>,
) -> Result<()> {
    enter(prompt, stack)?;
    for seg in parse(&prompt.body) {
        match seg {
            Segment::Text(t) => Piece::push_text(out, &t),
            Segment::Var(name) => match values.get(&name) {
                Some(v) => out.push(Piece::Value {
                    name,
                    value: v.clone(),
                }),
                None => out.push(Piece::Open { name }),
            },
            Segment::Include(id) => {
                let inner = include(shelf, prompt, &id)?;
                let mut included = Vec::new();
                write(shelf, inner, values, &mut included, stack)?;
                // An included file usually ends with a newline; inline it cleanly.
                if let Some(Piece::Text(last)) = included.last_mut() {
                    if last.ends_with('\n') {
                        last.pop();
                        if last.is_empty() {
                            included.pop();
                        }
                    }
                }
                for piece in included {
                    match piece {
                        Piece::Text(s) => Piece::push_text(out, &s),
                        other => out.push(other),
                    }
                }
            }
        }
    }
    stack.pop();
    Ok(())
}

#[cfg(test)]
#[path = "tests/template.rs"]
mod tests;
