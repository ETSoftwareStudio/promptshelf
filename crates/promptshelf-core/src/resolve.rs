//! Resolvers: where a blank's value comes from outside a chat (early binding).
//!
//! `from:` names the source. `conversation` is never resolved here: a model
//! fills it from the chat (late binding).

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::git;
use crate::prompt::VarSpec;
use crate::template::{Source, Value};

const COMMAND_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_OUTPUT: usize = 256 * 1024;

/// The sources that are not shell commands.
pub const NAMED_SOURCES: &[&str] = &["conversation", "branch", "clipboard", "selection", "last"];

/// True when `from` names a shell command, not a built-in source.
pub fn is_command(from: &str) -> bool {
    !NAMED_SOURCES.contains(&from)
}

/// What a resolver may do.
#[derive(Debug, Clone)]
pub struct Options {
    /// The folder for `branch` and shell commands: the repo you work in.
    pub cwd: PathBuf,
    /// Run shell commands at all. `--no-run` turns them off.
    pub run_commands: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            run_commands: true,
        }
    }
}

/// The outcome for one blank.
#[derive(Debug)]
pub enum Outcome {
    Value(Value),
    /// Left for the model or the person, with the reason when a resolver failed.
    Open(Option<String>),
}

/// What the caller knows that the resolver needs.
pub struct Facts<'a> {
    pub last: Option<&'a str>,
    /// True when shell commands of this prompt can run.
    pub command_allowed: bool,
    pub allow_hint: &'a str,
}

pub fn resolve(spec: &VarSpec, opts: &Options, facts: &Facts) -> Outcome {
    let Some(from) = spec.from.as_deref() else {
        return Outcome::Open(None);
    };
    let raw = match from {
        "conversation" => return Outcome::Open(None),
        "selection" => Err("only the desktop app can read the selection".to_string()),
        "last" => facts
            .last
            .map(|v| (v.to_string(), Source::LastUsed, None))
            .ok_or_else(|| "no last value yet".to_string()),
        "branch" => git::branch(&opts.cwd)
            .map(|b| (b, Source::Branch, None))
            .ok_or_else(|| format!("{} is not on a git branch", opts.cwd.display())),
        "clipboard" => clipboard()
            .map(|c| (c, Source::Clipboard, None))
            .ok_or_else(|| "could not read the clipboard".to_string()),
        cmd if !opts.run_commands => Err(format!("commands are off; did not run `{cmd}`")),
        cmd if !facts.command_allowed => {
            Err(format!("`{cmd}` is not allowed yet; {}", facts.allow_hint))
        }
        cmd => run_shell(cmd, &opts.cwd).map(|out| (out, Source::Command, Some(cmd.to_string()))),
    };
    let (text, source, detail) = match raw {
        Ok(r) => r,
        Err(reason) => return Outcome::Open(Some(reason)),
    };
    let value = match &spec.pattern {
        None => text,
        Some(p) => match pick(p, &text) {
            Ok(v) => v,
            Err(reason) => return Outcome::Open(Some(reason)),
        },
    };
    Outcome::Value(Value {
        value,
        source,
        detail,
    })
}

/// The first match of `pattern`, or its first capture group when it has one.
fn pick(pattern: &str, text: &str) -> Result<String, String> {
    let re = regex::Regex::new(pattern).map_err(|e| format!("bad pattern `{pattern}`: {e}"))?;
    let caps = re
        .captures(text)
        .ok_or_else(|| format!("no match for `{pattern}`"))?;
    let m = caps.get(1).or_else(|| caps.get(0)).expect("group 0 exists");
    Ok(m.as_str().to_string())
}

/// Runs a shell command in `cwd` with a time limit. Returns trimmed stdout.
pub fn run_shell(cmd: &str, cwd: &Path) -> Result<String, String> {
    let mut command = if cfg!(windows) {
        let mut c = Command::new("cmd");
        c.args(["/C", cmd]);
        c
    } else {
        let mut c = Command::new("sh");
        c.args(["-c", cmd]);
        c
    };
    let mut child = command
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("GIT_TERMINAL_PROMPT", "0")
        .spawn()
        .map_err(|e| format!("could not run `{cmd}`: {e}"))?;
    let mut stdout = child.stdout.take().expect("piped");
    let mut stderr = child.stderr.take().expect("piped");
    let out_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = (&mut stdout)
            .take(MAX_OUTPUT as u64 + 1)
            .read_to_end(&mut buf);
        // Drain the rest so the child does not block on a full pipe.
        let _ = std::io::copy(&mut stdout, &mut std::io::sink());
        buf
    });
    let err_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = (&mut stderr).take(4096).read_to_end(&mut buf);
        let _ = std::io::copy(&mut stderr, &mut std::io::sink());
        buf
    });
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if start.elapsed() > COMMAND_TIMEOUT => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "`{cmd}` took longer than {} seconds",
                    COMMAND_TIMEOUT.as_secs()
                ));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(e) => return Err(format!("`{cmd}` failed: {e}")),
        }
    };
    let out = out_reader.join().unwrap_or_default();
    let err = err_reader.join().unwrap_or_default();
    if !status.success() {
        let msg = String::from_utf8_lossy(&err);
        let first = msg.lines().next().unwrap_or("").trim();
        return Err(format!("`{cmd}` failed: {first}"));
    }
    if out.len() > MAX_OUTPUT {
        return Err(format!(
            "`{cmd}` printed more than {} KB",
            MAX_OUTPUT / 1024
        ));
    }
    let text = String::from_utf8_lossy(&out)
        .trim_end_matches(['\n', '\r'])
        .to_string();
    if text.trim().is_empty() {
        return Err(format!("`{cmd}` printed nothing"));
    }
    Ok(text)
}

/// Reads the clipboard with the system's own tool.
pub fn clipboard() -> Option<String> {
    let tries: &[&[&str]] = if cfg!(target_os = "macos") {
        &[&["pbpaste"]]
    } else if cfg!(windows) {
        &[&["powershell", "-NoProfile", "-Command", "Get-Clipboard -Raw"]]
    } else {
        &[
            &["wl-paste", "--no-newline"],
            &["xclip", "-selection", "clipboard", "-o"],
            &["xsel", "--clipboard", "--output"],
        ]
    };
    for argv in tries {
        let Ok(out) = Command::new(argv[0])
            .args(&argv[1..])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
        else {
            continue;
        };
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout)
                .trim_end_matches(['\n', '\r'])
                .to_string();
            if !text.is_empty() {
                return Some(text);
            }
        }
    }
    None
}

/// Copies text to the clipboard with the system's own tool.
pub fn set_clipboard(text: &str) -> Result<(), String> {
    use std::io::Write;
    let tries: &[&[&str]] = if cfg!(target_os = "macos") {
        &[&["pbcopy"]]
    } else if cfg!(windows) {
        &[&["clip"]]
    } else {
        &[
            &["wl-copy"],
            &["xclip", "-selection", "clipboard"],
            &["xsel", "--clipboard", "--input"],
        ]
    };
    for argv in tries {
        let Ok(mut child) = Command::new(argv[0])
            .args(&argv[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        else {
            continue;
        };
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        if child.wait().is_ok_and(|s| s.success()) {
            return Ok(());
        }
    }
    Err("no clipboard tool found (pbcopy, clip, wl-copy, xclip or xsel)".into())
}

#[cfg(test)]
#[path = "tests/resolve.rs"]
mod tests;
