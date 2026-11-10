//! Who runs the command: a person at a terminal, or a model.
//!
//! No terminal means a model. A model can read and fill, and write drafts to the
//! Inbox. Other writes need `--user-action` (a UI client acting for a person) or
//! `[agents] allow_writes = true` in the config. This guards against mistakes; it
//! is not a security boundary.

use std::io::IsTerminal;

use promptshelf_core::{Config, Error, Result};

/// Forces the mode: `1` for agent, `0` for person. Tests use it.
pub const AGENT_ENV: &str = "PROMPTSHELF_AGENT";

#[derive(Debug, Clone, Copy)]
pub struct Caller {
    pub agent: bool,
    pub user_action: bool,
}

impl Caller {
    pub fn detect(user_action: bool) -> Self {
        let agent = match std::env::var(AGENT_ENV).ok().as_deref() {
            Some("1") | Some("true") => true,
            Some("0") | Some("false") => false,
            _ => !(std::io::stdin().is_terminal() || std::io::stderr().is_terminal()),
        };
        Caller { agent, user_action }
    }

    /// Checks a write outside the Inbox.
    pub fn may_write(&self, config: Option<&Config>, what: &str) -> Result<()> {
        if !self.agent || self.user_action || config.is_some_and(|c| c.agents.allow_writes) {
            return Ok(());
        }
        Err(Error::Refused(format!(
            "{what} needs a person. Agents can read, fill and save drafts with `shelf save`. \
             To allow more, set [agents] allow_writes = true in the config."
        )))
    }
}
