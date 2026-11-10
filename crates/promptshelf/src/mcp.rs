//! `shelf mcp`: the Model Context Protocol over stdio, for apps with no shell.
//!
//! The app starts it as a child process and stops it when done: no daemon and no
//! network port. Clients are models, so the agent rules apply: they can read,
//! fill and save drafts. Templates also show up as MCP prompts (slash commands).

use std::collections::BTreeMap;
use std::io::{BufRead, Write};

use promptshelf_core::resolve::Options;
use promptshelf_core::template::LATE_BINDING;
use promptshelf_core::write::NewPrompt;
use promptshelf_core::{Config, Error, Kind, ListFilter, Result, Shelf, fill, template};
use serde_json::{Value, json};

/// The protocol versions this server speaks, newest first.
const VERSIONS: &[&str] = &["2025-06-18", "2025-03-26", "2024-11-05"];

pub fn run() -> Result<u8> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Ok(msg) => handle(&msg),
            Err(e) => Some(error(Value::Null, -32700, &format!("parse error: {e}"))),
        };
        if let Some(reply) = reply {
            let _ = writeln!(stdout, "{reply}");
            let _ = stdout.flush();
        }
    }
    Ok(crate::exit::OK)
}

fn error(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

/// Answers one message. Notifications (no `id`) get no answer.
pub fn handle(msg: &Value) -> Option<Value> {
    let id = msg.get("id").cloned()?;
    let method = msg.get("method").and_then(Value::as_str).unwrap_or("");
    let params = msg.get("params").cloned().unwrap_or(json!({}));
    let result = match method {
        "initialize" => Ok(initialize(&params)),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": tools() })),
        "tools/call" => Ok(call_tool(&params)),
        "prompts/list" => prompts_list(),
        "prompts/get" => prompts_get(&params),
        other => return Some(error(id, -32601, &format!("unknown method {other}"))),
    };
    Some(match result {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err(e) => error(id, -32602, &e.to_string()),
    })
}

fn initialize(params: &Value) -> Value {
    let asked = params.get("protocolVersion").and_then(Value::as_str);
    let version = asked
        .filter(|v| VERSIONS.contains(v))
        .unwrap_or(VERSIONS[0]);
    json!({
        "protocolVersion": version,
        "capabilities": { "tools": {}, "prompts": {} },
        "serverInfo": { "name": "promptshelf", "version": promptshelf_core::VERSION },
        "instructions": "PromptShelf holds the user's saved prompts. Search before writing a prompt from scratch. When fill_prompt returns open blanks, fill them from the conversation and ask the user only about what is unclear.",
    })
}

fn tools() -> Value {
    json!([
        {
            "name": "search_prompts",
            "description": "Search the user's saved prompts by words in the ID, title, tags and text.",
            "inputSchema": { "type": "object", "properties": { "query": { "type": "string" } }, "required": ["query"] },
            "annotations": { "readOnlyHint": true },
        },
        {
            "name": "get_prompt",
            "description": "Get one prompt with its text and its blanks.",
            "inputSchema": { "type": "object", "properties": { "id": { "type": "string", "description": "library/id, or a bare id" } }, "required": ["id"] },
            "annotations": { "readOnlyHint": true },
        },
        {
            "name": "fill_prompt",
            "description": "Fill a prompt's blanks. Open blanks come back in `missing` with hints: fill them from the conversation and call again.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string" },
                    "values": { "type": "object", "additionalProperties": { "type": "string" } },
                },
                "required": ["id"],
            },
            "annotations": { "readOnlyHint": true },
        },
        {
            "name": "save_draft",
            "description": "Save text as a draft prompt in the user's Inbox. Use when the user asks to keep a prompt.",
            "inputSchema": {
                "type": "object",
                "properties": { "title": { "type": "string" }, "text": { "type": "string" } },
                "required": ["title", "text"],
            },
        },
    ])
}

fn load() -> Result<Shelf> {
    Ok(Shelf::load(Config::load()?))
}

fn str_arg<'a>(args: &'a Value, key: &str) -> Result<&'a str> {
    args.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| Error::Invalid(format!("missing argument \"{key}\"")))
}

fn call_tool(params: &Value) -> Value {
    let name = params.get("name").and_then(Value::as_str).unwrap_or("");
    let args = params.get("arguments").cloned().unwrap_or(json!({}));
    let result = match name {
        "search_prompts" => search(&args),
        "get_prompt" => get(&args),
        "fill_prompt" => fill_tool(&args),
        "save_draft" => save_draft(&args),
        other => Err(Error::Invalid(format!("unknown tool {other}"))),
    };
    match result {
        Ok(v) => json!({
            "content": [{ "type": "text", "text": v.to_string() }],
            "structuredContent": v,
        }),
        Err(e) => json!({
            "content": [{ "type": "text", "text": e.to_string() }],
            "isError": true,
        }),
    }
}

fn search(args: &Value) -> Result<Value> {
    let shelf = load()?;
    let hits = shelf.search(str_arg(args, "query")?, &ListFilter::default())?;
    let prompts: Vec<Value> = hits
        .iter()
        .take(20)
        .map(|h| json!({ "id": h.prompt.full_id(), "title": h.prompt.title, "kind": h.prompt.kind.as_str(), "tags": h.prompt.tags }))
        .collect();
    Ok(json!({ "prompts": prompts }))
}

fn get(args: &Value) -> Result<Value> {
    let shelf = load()?;
    let p = shelf.resolve(str_arg(args, "id")?)?;
    let blanks: Vec<Value> = template::blanks(&shelf, p)?
        .into_iter()
        .map(|(name, spec)| {
            let mut v = serde_json::to_value(spec).unwrap_or_default();
            v["name"] = json!(name);
            v
        })
        .collect();
    Ok(
        json!({ "id": p.full_id(), "title": p.title, "kind": p.kind.as_str(), "body": p.body, "blanks": blanks }),
    )
}

fn fill_with(shelf: &Shelf, id: &str, values: BTreeMap<String, String>) -> Result<(String, Value)> {
    let p = shelf.resolve(id)?;
    // A model is the caller: shell commands from `from:` do not run.
    let opts = Options {
        run_commands: false,
        ..Options::default()
    };
    let r = fill::fill(shelf, p, &values, &opts)?;
    let mut v =
        json!({ "id": p.full_id(), "text": r.text, "complete": r.complete, "missing": r.missing });
    if !r.complete {
        v["instruction"] = json!(LATE_BINDING);
    }
    Ok((r.text, v))
}

fn values_of(args: &Value, key: &str) -> BTreeMap<String, String> {
    args.get(key)
        .and_then(Value::as_object)
        .map(|m| {
            m.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default()
}

fn fill_tool(args: &Value) -> Result<Value> {
    let shelf = load()?;
    Ok(fill_with(&shelf, str_arg(args, "id")?, values_of(args, "values"))?.1)
}

fn save_draft(args: &Value) -> Result<Value> {
    let shelf = load()?;
    let personal = shelf
        .personal()
        .ok_or_else(|| Error::Invalid("no Personal library; run `shelf init`".into()))?;
    let (id, path) = promptshelf_core::write::create_prompt(
        personal,
        &NewPrompt {
            title: str_arg(args, "title")?,
            kind: Kind::Draft,
            body: str_arg(args, "text")?,
            tags: &[],
        },
    )?;
    Ok(json!({ "id": format!("{}/{id}", personal.name()), "path": path.display().to_string() }))
}

fn prompts_list() -> Result<Value> {
    let shelf = load()?;
    let prompts: Vec<Value> = shelf
        .list(&ListFilter {
            kind: Some(Kind::Template),
            ..Default::default()
        })?
        .into_iter()
        .map(|p| {
            let arguments: Vec<Value> = template::blanks(&shelf, p)
                .unwrap_or_default()
                .into_iter()
                .map(|(name, spec)| json!({ "name": name, "description": spec.hint, "required": false }))
                .collect();
            json!({ "name": p.full_id(), "title": p.title, "description": p.description, "arguments": arguments })
        })
        .collect();
    Ok(json!({ "prompts": prompts }))
}

fn prompts_get(params: &Value) -> Result<Value> {
    let shelf = load()?;
    let name = str_arg(params, "name")?;
    let (text, v) = fill_with(&shelf, name, values_of(params, "arguments"))?;
    let mut message = text;
    if v["complete"] == json!(false) {
        message.push_str("\n\n---\n");
        message.push_str(LATE_BINDING);
        for m in v["missing"].as_array().into_iter().flatten() {
            let hint = m["hint"]
                .as_str()
                .map(|h| format!(": {h}"))
                .unwrap_or_default();
            message.push_str(&format!(
                "\n- {{{{{}}}}}{hint}",
                m["name"].as_str().unwrap_or("")
            ));
        }
    }
    Ok(json!({
        "description": v["id"],
        "messages": [{ "role": "user", "content": { "type": "text", "text": message } }],
    }))
}
