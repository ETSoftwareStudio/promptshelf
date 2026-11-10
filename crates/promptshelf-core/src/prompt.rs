//! One prompt file: Markdown with optional YAML frontmatter.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// The folders of a library, in scan order, with the kind each one implies.
pub const FOLDERS: &[(&str, Kind)] = &[
    ("templates", Kind::Template),
    ("snippets", Kind::Snippet),
    ("context", Kind::Context),
    ("inbox", Kind::Draft),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Template,
    Snippet,
    Context,
    Persona,
    Workflow,
    Draft,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Template => "template",
            Kind::Snippet => "snippet",
            Kind::Context => "context",
            Kind::Persona => "persona",
            Kind::Workflow => "workflow",
            Kind::Draft => "draft",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "template" => Kind::Template,
            "snippet" => Kind::Snippet,
            "context" => Kind::Context,
            "persona" => Kind::Persona,
            "workflow" => Kind::Workflow,
            "draft" => Kind::Draft,
            _ => return None,
        })
    }

    /// The folder new prompts of this kind go into.
    pub fn folder(self) -> &'static str {
        match self {
            Kind::Snippet => "snippets",
            Kind::Context => "context",
            Kind::Draft => "inbox",
            _ => "templates",
        }
    }
}

/// How a variable gets its value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct VarSpec {
    /// Where the value comes from: `conversation`, `branch`, `clipboard`,
    /// `selection`, `last`, or a shell command such as `git diff --staged`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub choices: Vec<String>,
    /// A regex that picks the value out of the source, for example a ticket ID
    /// out of the branch name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

/// The frontmatter fields the core reads. Other fields are kept on disk and
/// ignored here.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct Frontmatter {
    id: Option<String>,
    title: Option<String>,
    description: Option<String>,
    kind: Option<String>,
    tags: Vec<String>,
    vars: BTreeMap<String, VarSpec>,
    forked_from: Option<String>,
}

/// A prompt as read from disk.
#[derive(Debug, Clone, Serialize)]
pub struct Prompt {
    /// The ID inside its library, for example `strict-pr-review`.
    pub id: String,
    pub library: String,
    pub kind: Kind,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub vars: BTreeMap<String, VarSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forked_from: Option<String>,
    pub body: String,
    pub path: PathBuf,
}

impl Prompt {
    /// The full ID: `<library>/<id>`.
    pub fn full_id(&self) -> String {
        format!("{}/{}", self.library, self.id)
    }

    /// Reads a prompt file. `folder_kind` is the kind its folder implies.
    pub fn load(path: &Path, library: &str, folder_kind: Kind) -> Result<Self> {
        let text = std::fs::read_to_string(path).map_err(|e| Error::io(path, e))?;
        Self::parse(&text, path, library, folder_kind)
    }

    pub fn parse(text: &str, path: &Path, library: &str, folder_kind: Kind) -> Result<Self> {
        let text = text.replace("\r\n", "\n");
        let (yaml, body) = split_frontmatter(&text);
        let fm: Frontmatter = match yaml {
            Some(yaml) if !yaml.trim().is_empty() => {
                serde_saphyr::from_str(yaml).map_err(|e| Error::Frontmatter {
                    path: path.to_path_buf(),
                    message: first_line(&e.to_string()),
                })?
            }
            _ => Frontmatter::default(),
        };

        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let id = fm.id.unwrap_or_else(|| slugify(&stem));
        if !is_valid_id(&id) {
            return Err(Error::Frontmatter {
                path: path.to_path_buf(),
                message: format!("ID \"{id}\" must use only a-z, 0-9 and -"),
            });
        }
        let kind = match fm.kind.as_deref() {
            Some(k) => Kind::parse(k).ok_or_else(|| Error::Frontmatter {
                path: path.to_path_buf(),
                message: format!("unknown kind \"{k}\""),
            })?,
            None => folder_kind,
        };
        let body = body.to_string();
        let title = fm
            .title
            .or_else(|| first_heading(&body))
            .unwrap_or_else(|| title_from_id(&id));

        Ok(Prompt {
            id,
            library: library.to_string(),
            kind,
            title,
            description: fm.description,
            tags: fm.tags,
            vars: fm.vars,
            forked_from: fm.forked_from,
            body,
            path: path.to_path_buf(),
        })
    }
}

/// Splits `---\n<yaml>\n---\n<body>`. A file without the opening line has no
/// frontmatter.
pub fn split_frontmatter(text: &str) -> (Option<&str>, &str) {
    let Some(rest) = text.strip_prefix("---\n") else {
        return (None, text);
    };
    if let Some(body) = rest.strip_prefix("---\n") {
        return (Some(""), body);
    }
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        if line.trim_end() == "---" {
            let yaml = &rest[..offset];
            let body = &rest[offset + line.len()..];
            return (Some(yaml), body.strip_prefix('\n').unwrap_or(body));
        }
        offset += line.len();
    }
    (None, text)
}

/// IDs and library names: lowercase letters, digits and single hyphens.
pub fn is_valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 100
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !id.starts_with('-')
        && !id.ends_with('-')
        && !id.contains("--")
}

/// Turns a title into an ID: "Strict PR review!" → "strict-pr-review".
pub fn slugify(title: &str) -> String {
    let mut out = String::new();
    let mut hyphen = false;
    for c in title.chars().flat_map(char::to_lowercase) {
        if c.is_ascii_alphanumeric() {
            if hyphen && !out.is_empty() {
                out.push('-');
            }
            hyphen = false;
            out.push(c);
        } else {
            hyphen = true;
        }
    }
    out.truncate(80);
    let trimmed = out.trim_end_matches('-').to_string();
    if trimmed.is_empty() {
        "untitled".into()
    } else {
        trimmed
    }
}

fn title_from_id(id: &str) -> String {
    let mut words = id.split('-');
    let mut out = String::new();
    if let Some(first) = words.next() {
        let mut chars = first.chars();
        if let Some(c) = chars.next() {
            out.extend(c.to_uppercase());
            out.push_str(chars.as_str());
        }
    }
    for w in words {
        out.push(' ');
        out.push_str(w);
    }
    out
}

fn first_heading(body: &str) -> Option<String> {
    body.lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .and_then(|l| l.strip_prefix("# "))
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
}

fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or(s).trim().to_string()
}

#[cfg(test)]
#[path = "tests/prompt.rs"]
mod tests;
