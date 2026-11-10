//! `shelf mcp` over stdio: one JSON-RPC message per line.

use std::path::Path;

use assert_cmd::Command;
use serde_json::{Value, json};

#[test]
fn speaks_mcp_over_stdio() {
    let dir = tempfile::tempdir().unwrap();
    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/library");
    let personal = dir.path().join("personal");
    std::fs::create_dir_all(personal.join("inbox")).unwrap();
    std::fs::write(
        dir.path().join("config.toml"),
        format!(
            "[[library]]\nname = \"personal\"\npath = {:?}\n\n[[library]]\nname = \"examples\"\npath = {:?}\n",
            personal.display().to_string(),
            examples.display().to_string()
        ),
    )
    .unwrap();
    let messages = [
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "test", "version": "1"}}}),
        json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}),
        json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "search_prompts", "arguments": {"query": "review"}}}),
        json!({"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {"name": "fill_prompt", "arguments": {"id": "strict-pr-review", "values": {"focus": "errors"}}}}),
        json!({"jsonrpc": "2.0", "id": 5, "method": "prompts/list"}),
        json!({"jsonrpc": "2.0", "id": 6, "method": "prompts/get", "params": {"name": "examples/explain-like-a-colleague", "arguments": {}}}),
        json!({"jsonrpc": "2.0", "id": 7, "method": "tools/call", "params": {"name": "save_draft", "arguments": {"title": "From chat", "text": "Summarise this."}}}),
        json!({"jsonrpc": "2.0", "id": 8, "method": "tools/call", "params": {"name": "get_prompt", "arguments": {"id": "nope"}}}),
        json!({"jsonrpc": "2.0", "id": 9, "method": "no/such"}),
    ];
    let input: String = messages.iter().map(|m| format!("{m}\n")).collect();
    let out = Command::cargo_bin("shelf")
        .unwrap()
        .arg("mcp")
        .env("PROMPTSHELF_CONFIG_DIR", dir.path())
        .env("PROMPTSHELF_STATE_DIR", dir.path().join("state"))
        .env("PROMPTSHELF_AGENT", "1")
        .current_dir(dir.path())
        .write_stdin(input)
        .output()
        .unwrap();
    assert!(out.status.success());
    let replies: Vec<Value> = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    // The notification gets no reply.
    assert_eq!(replies.len(), 9);
    let by_id = |id: i64| replies.iter().find(|r| r["id"] == id).unwrap();

    assert_eq!(by_id(1)["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(by_id(1)["result"]["serverInfo"]["name"], "promptshelf");
    let tools: Vec<&str> = by_id(2)["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        tools,
        ["search_prompts", "get_prompt", "fill_prompt", "save_draft"]
    );
    let found = &by_id(3)["result"]["structuredContent"]["prompts"][0]["id"];
    assert_eq!(found, "examples/strict-pr-review");

    let fill = &by_id(4)["result"]["structuredContent"];
    assert_eq!(fill["complete"], false);
    // A model is the caller, so `git diff --staged` did not run.
    assert_eq!(fill["missing"][0]["name"], "diff");
    assert!(fill["text"].as_str().unwrap().contains("Focus on errors."));

    let names: Vec<&str> = by_id(5)["result"]["prompts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "examples/explain-like-a-colleague",
            "examples/strict-pr-review"
        ]
    );

    let msg = by_id(6)["result"]["messages"][0]["content"]["text"]
        .as_str()
        .unwrap();
    assert!(msg.starts_with("Explain {{topic}} to a developer new to this codebase."));
    assert!(msg.contains("- {{topic}}: what to explain"));

    assert_eq!(
        by_id(7)["result"]["structuredContent"]["id"],
        "personal/from-chat"
    );
    assert!(personal.join("inbox/from-chat.md").exists());
    assert_eq!(by_id(8)["result"]["isError"], true);
    assert_eq!(by_id(9)["error"]["code"], -32601);
}
