//! Filling a prompt. For each blank, in order: the value the caller gives, the
//! `from:` resolver, then the default. Blanks still open are left for the model
//! or the person, with the reason when a resolver failed.

use std::collections::BTreeMap;

use crate::error::Result;
use crate::prompt::Prompt;
use crate::resolve::{self, Facts, Options, Outcome};
use crate::shelf::Shelf;
use crate::state::{Allowed, UsageLog};
use crate::template::{self, Rendered, Source, Value};

/// Fills a prompt. `given` holds values from `--var` or the UI.
pub fn fill(
    shelf: &Shelf,
    prompt: &Prompt,
    given: &BTreeMap<String, String>,
    opts: &Options,
) -> Result<Rendered> {
    let usage = UsageLog::load();
    let last = usage.get(&prompt.full_id());
    let allowed = Allowed::load().unwrap_or_default();
    let mut values: BTreeMap<String, Value> = given
        .iter()
        .map(|(k, v)| (k.clone(), Value::given(v.clone())))
        .collect();
    let mut reasons: BTreeMap<String, String> = BTreeMap::new();

    for blank in template::blanks_owned(shelf, prompt)? {
        if values.contains_key(&blank.name) {
            continue;
        }
        let owner_library = blank.owner.split('/').next().unwrap_or("");
        let command_allowed = blank.spec.from.as_deref().is_some_and(|cmd| {
            shelf
                .library(owner_library)
                .is_ok_and(|l| l.config.is_trusted())
                || allowed.allows(&blank.owner, cmd)
        });
        let allow_hint = format!("run `shelf allow {}` to allow it", blank.owner);
        let facts = Facts {
            last: last
                .and_then(|u| u.values.get(&blank.name))
                .map(String::as_str),
            command_allowed,
            allow_hint: &allow_hint,
        };
        match resolve::resolve(&blank.spec, opts, &facts) {
            Outcome::Value(v) => {
                values.insert(blank.name, v);
            }
            Outcome::Open(reason) => {
                if let Some(default) = blank.spec.default {
                    values.insert(
                        blank.name,
                        Value {
                            value: default,
                            source: Source::Default,
                            detail: None,
                        },
                    );
                } else if let Some(reason) = reason {
                    reasons.insert(blank.name, reason);
                }
            }
        }
    }

    let mut rendered = template::render(shelf, prompt, &values)?;
    for m in &mut rendered.missing {
        m.reason = reasons.remove(&m.name);
    }
    Ok(rendered)
}

/// Records a complete fill in this machine's usage log.
pub fn record_use(prompt: &Prompt, rendered: &Rendered) -> Result<()> {
    if !rendered.complete {
        return Ok(());
    }
    let values = rendered
        .values
        .iter()
        .map(|f| (f.name.clone(), f.value.value.clone()))
        .collect();
    UsageLog::record(&prompt.full_id(), values)
}

/// Shell commands a prompt and its includes would run, with the prompt that
/// declares each one.
pub fn commands(shelf: &Shelf, prompt: &Prompt) -> Result<Vec<(String, String)>> {
    Ok(template::blanks_owned(shelf, prompt)?
        .into_iter()
        .filter_map(|b| {
            let from = b.spec.from?;
            resolve::is_command(&from).then_some((b.owner, from))
        })
        .collect())
}

/// Allows the shell commands of a prompt and its includes, for a prompt from
/// a library you do not trust. Only a person does this; nothing calls it on
/// its own. Returns each command with the prompt that declares it.
pub fn allow(shelf: &Shelf, prompt: &Prompt) -> Result<Vec<(String, String)>> {
    let commands = commands(shelf, prompt)?;
    let mut allowed = Allowed::load()?;
    for (owner, cmd) in &commands {
        allowed.allow(owner, cmd);
    }
    allowed.save()?;
    Ok(commands)
}
