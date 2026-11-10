//! Filling a prompt's blanks.

use std::collections::BTreeMap;

use promptshelf_core::resolve::Options;
use promptshelf_core::template::{self, LATE_BINDING};
use promptshelf_core::{Error, Result, fill};
use serde_json::json;

use super::{Ctx, describe_spec, load};
use crate::output;
use crate::{FillArgs, exit};

pub(super) fn parse_vars(vars: &[String]) -> Result<BTreeMap<String, String>> {
    let mut out = BTreeMap::new();
    for v in vars {
        let Some((k, val)) = v.split_once('=') else {
            return Err(Error::Invalid(format!(
                "--var needs NAME=VALUE, got \"{v}\""
            )));
        };
        let k = k.trim();
        if !template::is_var_name(k) {
            return Err(Error::Invalid(format!("\"{k}\" is not a blank name")));
        }
        out.insert(k.to_string(), val.to_string());
    }
    Ok(out)
}

pub(super) fn fill_cmd(ctx: &Ctx, a: FillArgs) -> Result<u8> {
    let given = parse_vars(&a.vars)?;
    let shelf = load()?;
    let p = shelf.resolve(&a.id)?;
    let opts = Options {
        cwd: match a.cwd {
            Some(dir) => dir,
            None => Options::default().cwd,
        },
        run_commands: !a.no_run,
    };
    let r = fill::fill(&shelf, p, &given, &opts)?;
    if let Err(e) = fill::record_use(p, &r) {
        eprintln!("shelf: warning: could not record usage: {e}");
    }
    let code = if r.complete {
        exit::OK
    } else {
        exit::BLANKS_OPEN
    };
    if ctx.json {
        let mut v = json!({
            "id": p.full_id(),
            "text": r.text,
            "complete": r.complete,
            "values": r.values,
            "missing": r.missing,
        });
        if !r.complete {
            v["instruction"] = json!(LATE_BINDING);
        }
        output::json(v);
    } else {
        print!("{}", r.text);
        if !r.text.ends_with('\n') {
            println!();
        }
        if !r.complete {
            eprintln!();
            eprintln!(
                "shelf: {} blank(s) still open. Fill them with --var NAME=VALUE:",
                r.missing.len()
            );
            for m in &r.missing {
                eprintln!("  {}{}", m.name, describe_spec(&m.spec));
                if let Some(reason) = &m.reason {
                    eprintln!("    {reason}");
                }
            }
        }
    }
    Ok(code)
}
