//! JSON and text output.

use promptshelf_core::{JSON_SCHEMA, Prompt, Shelf, shelf::Diagnostic, template};
use serde::Serialize;
use serde_json::{Value, json};

/// Prints `{"schema": 1, ...fields}` on stdout.
pub fn json(fields: Value) {
    let mut obj = serde_json::Map::new();
    obj.insert("schema".into(), json!(JSON_SCHEMA));
    if let Value::Object(map) = fields {
        obj.extend(map);
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&Value::Object(obj)).unwrap_or_default()
    );
}

pub fn json_error(code: &str, message: &str, matches: &[String]) {
    let mut error = json!({ "code": code, "message": message });
    if !matches.is_empty() {
        error["matches"] = json!(matches);
    }
    json(json!({ "error": error }));
}

/// The short form of a prompt, for lists.
#[derive(Serialize)]
pub struct Summary<'a> {
    pub id: String,
    pub library: &'a str,
    pub kind: &'static str,
    pub title: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'a str>,
    pub tags: &'a [String],
    pub blanks: Vec<String>,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<u32>,
}

impl<'a> Summary<'a> {
    pub fn new(shelf: &Shelf, p: &'a Prompt, score: Option<u32>) -> Self {
        let blanks = template::blanks(shelf, p)
            .map(|b| b.into_iter().map(|(n, _)| n).collect())
            .unwrap_or_default();
        Summary {
            id: p.full_id(),
            library: &p.library,
            kind: p.kind.as_str(),
            title: &p.title,
            description: p.description.as_deref(),
            tags: &p.tags,
            blanks,
            path: p.path.display().to_string(),
            score,
        }
    }
}

pub fn diagnostics_json(diags: &[Diagnostic]) -> Value {
    json!(
        diags
            .iter()
            .map(|d| json!({ "path": d.path.display().to_string(), "message": d.message }))
            .collect::<Vec<_>>()
    )
}

pub fn warn_diagnostics(diags: &[Diagnostic]) {
    for d in diags {
        eprintln!("shelf: warning: {}: {}", d.path.display(), d.message);
    }
}

/// Prints prompts as aligned rows: ID, kind, title.
pub fn table(rows: &[Summary]) {
    let width = rows.iter().map(|r| r.id.len()).max().unwrap_or(0);
    for r in rows {
        let tags = if r.tags.is_empty() {
            String::new()
        } else {
            format!("  [{}]", r.tags.join(", "))
        };
        println!("{:width$}  {:8}  {}{}", r.id, r.kind, r.title, tags);
    }
}
