//! Filling a prompt: values the caller gives first, then defaults.

use std::collections::BTreeMap;

use crate::error::Result;
use crate::prompt::Prompt;
use crate::shelf::Shelf;
use crate::template::{self, Rendered, Source, Value};

/// Fills a prompt. `given` holds values from `--var` or the UI.
pub fn fill(shelf: &Shelf, prompt: &Prompt, given: &BTreeMap<String, String>) -> Result<Rendered> {
    let mut values: BTreeMap<String, Value> = given
        .iter()
        .map(|(k, v)| (k.clone(), Value::given(v.clone())))
        .collect();
    for (name, spec) in template::blanks(shelf, prompt)? {
        if values.contains_key(&name) {
            continue;
        }
        if let Some(default) = spec.default {
            values.insert(
                name,
                Value {
                    value: default,
                    source: Source::Default,
                    detail: None,
                },
            );
        }
    }
    template::render(shelf, prompt, &values)
}
