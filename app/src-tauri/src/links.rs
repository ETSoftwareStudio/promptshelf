//! `promptshelf://` links. Any web page can open one, so a link is untrusted
//! input: this module checks it and turns it into a typed route. A route only
//! moves the library window; it never writes a file or runs a command.

use promptshelf_core::prompt::is_valid_id;
use serde::Serialize;

pub const SCHEME: &str = "promptshelf";

/// Longer links are refused before they are read.
pub const MAX_LINK: usize = 2048;

/// The longest search or title a link can carry, in characters.
pub const MAX_TEXT: usize = 200;

/// Where a link goes. The UI gets it as JSON with a `route` field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "route", rename_all = "lowercase")]
pub enum Link {
    /// `promptshelf://`: only open the window.
    Home,
    /// `promptshelf://open/<library>/<id>`: select the prompt.
    Open { library: String, id: String },
    /// `promptshelf://fill/<library>/<id>`: select it and open the Send preview.
    Fill { library: String, id: String },
    /// `promptshelf://search?q=<text>`: All Prompts with that search.
    Search { query: String },
    /// `promptshelf://new?title=<text>`: a new prompt with that title, not saved.
    New { title: Option<String> },
    /// A link the app can not open, with the reason in words for a toast.
    Bad { reason: String },
}

impl Link {
    /// A short line for the log. It names the route and the prompt, never
    /// the search or title text.
    pub fn describe(&self) -> String {
        match self {
            Link::Home => "home".into(),
            Link::Open { library, id } => format!("open {library}/{id}"),
            Link::Fill { library, id } => format!("fill {library}/{id}"),
            Link::Search { .. } => "search".into(),
            Link::New { .. } => "new".into(),
            Link::Bad { reason } => format!("refused: {reason}"),
        }
    }
}

fn bad(reason: impl Into<String>) -> Link {
    Link::Bad {
        reason: reason.into(),
    }
}

/// The link for a prompt, as Copy Link puts it on the clipboard.
pub fn open_link(library: &str, id: &str) -> String {
    format!("{SCHEME}://open/{library}/{id}")
}

/// Turns a link into a route. Every problem becomes [`Link::Bad`].
pub fn parse(url: &str) -> Link {
    if url.len() > MAX_LINK {
        return bad("the link is too long");
    }
    let Some(rest) = strip_scheme(url) else {
        return bad("it is not a promptshelf:// link");
    };
    let rest = rest.split_once('#').map_or(rest, |(r, _)| r);
    let (path, query) = rest.split_once('?').unwrap_or((rest, ""));
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let Some((route, args)) = segments.split_first() else {
        return Link::Home;
    };
    match route.to_ascii_lowercase().as_str() {
        r @ ("open" | "fill") => {
            let [library, id] = args else {
                return bad("the link needs a library and an ID");
            };
            if !is_valid_id(library) || !is_valid_id(id) {
                return bad("the prompt ID is not valid");
            }
            let (library, id) = (library.to_string(), id.to_string());
            if r == "open" {
                Link::Open { library, id }
            } else {
                Link::Fill { library, id }
            }
        }
        r @ ("search" | "new") => {
            if !args.is_empty() {
                return bad(format!("nothing can follow \"{r}\" in the path"));
            }
            let key = if r == "search" { "q" } else { "title" };
            match param(query, key) {
                Err(reason) => bad(reason),
                Ok(text) if r == "search" => Link::Search {
                    query: text.unwrap_or_default(),
                },
                Ok(text) => Link::New {
                    title: text.filter(|t| !t.is_empty()),
                },
            }
        }
        // `import` waits for the sharing feature, like every other name.
        name if is_valid_id(name) => bad(format!("\"{name}\" links are not supported yet")),
        _ => bad("this kind of link is not supported yet"),
    }
}

/// The part after `promptshelf:` and an optional `//`. The scheme is not
/// case-sensitive.
fn strip_scheme(url: &str) -> Option<&str> {
    let url = url.trim();
    let (scheme, rest) = url.split_once(':')?;
    if !scheme.eq_ignore_ascii_case(SCHEME) {
        return None;
    }
    Some(rest.strip_prefix("//").unwrap_or(rest))
}

/// The first value of `key` in a query string, decoded and cleaned.
fn param(query: &str, key: &str) -> Result<Option<String>, String> {
    for pair in query.split('&') {
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        if k != key {
            continue;
        }
        let text = decode(v).ok_or("the link has a bad %-code or is not UTF-8")?;
        let text = clean(&text);
        if text.chars().count() > MAX_TEXT {
            return Err(format!("the text is longer than {MAX_TEXT} characters"));
        }
        return Ok(Some(text));
    }
    Ok(None)
}

/// Decodes `%XX` and `+` (a space). `None` for a bad code or bytes that are
/// not UTF-8.
fn decode(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => out.push(b' '),
            b'%' => {
                let hex = bytes.get(i + 1..i + 3)?;
                if !hex.iter().all(u8::is_ascii_hexdigit) {
                    return None;
                }
                let hex = std::str::from_utf8(hex).ok()?;
                out.push(u8::from_str_radix(hex, 16).ok()?);
                i += 2;
            }
            b => out.push(b),
        }
        i += 1;
    }
    String::from_utf8(out).ok()
}

/// One line of text: control characters and runs of space become one space.
fn clean(s: &str) -> String {
    let spaced: String = s
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    spaced.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
#[path = "tests/links.rs"]
mod tests;
