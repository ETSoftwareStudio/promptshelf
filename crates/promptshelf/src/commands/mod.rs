//! One function per command. Each returns an exit code or an error.

mod fill;
mod find;
mod save;
mod setup;
mod sync;

use clap::CommandFactory;
use promptshelf_core::{Config, Error, ListFilter, Result, Shelf, VarSpec};

use crate::caller::Caller;
use crate::output;
use crate::{Cli, Command, FilterArgs, LibsCommand, exit};

struct Ctx {
    json: bool,
    caller: Caller,
}

pub fn dispatch(cli: Cli) -> u8 {
    let ctx = Ctx {
        json: cli.json,
        caller: Caller::detect(cli.user_action),
    };
    let result = match cli.command {
        None => {
            let _ = Cli::command().print_help();
            Ok(exit::OK)
        }
        Some(Command::Init(a)) => setup::init(&ctx, a),
        Some(Command::List(a)) => find::list(&ctx, a),
        Some(Command::Search(a)) => {
            let query = a.query.join(" ");
            find::search(&ctx, &query, a.filter)
        }
        Some(Command::Show(a)) => find::show(&ctx, a),
        Some(Command::Fill(a)) => fill::fill_cmd(&ctx, a),
        Some(Command::Save(a)) => save::save(&ctx, a),
        Some(Command::New(a)) => save::new(&ctx, a),
        Some(Command::Libs(a)) => match a.command.unwrap_or(LibsCommand::List) {
            LibsCommand::List => setup::libs_list(&ctx),
            LibsCommand::Add(a) => setup::libs_add(&ctx, a),
        },
        Some(Command::Skill(a)) => setup::skill(&ctx, a.command),
        Some(Command::Check) => find::check(&ctx),
        Some(Command::Pin(a)) => sync::pin(&ctx, a),
        Some(Command::Unpin(a)) => sync::unpin(&ctx, a),
        Some(Command::Pins) => sync::pins(&ctx),
        Some(Command::Sync(a)) => sync::sync(&ctx, a),
        Some(Command::Allow(a)) => sync::allow(&ctx, a),
        Some(Command::MergePins { base, ours, theirs }) => sync::merge_pins(&base, &ours, &theirs),
    };
    match result {
        Ok(code) => code,
        Err(e) => report(&ctx, &e),
    }
}

fn report(ctx: &Ctx, e: &Error) -> u8 {
    let code = match e {
        Error::NotFound(_) | Error::Ambiguous { .. } | Error::UnknownLibrary(_) => exit::NOT_FOUND,
        Error::Invalid(_) => exit::USAGE,
        Error::Refused(_) => exit::REFUSED,
        _ => exit::ERROR,
    };
    let matches = match e {
        Error::Ambiguous { matches, .. } => matches.clone(),
        _ => Vec::new(),
    };
    if ctx.json {
        output::json_error(e.code(), &e.to_string(), &matches);
    } else {
        eprintln!("shelf: error: {e}");
    }
    code
}

fn load() -> Result<Shelf> {
    Ok(Shelf::load(Config::load()?))
}

fn filter(a: FilterArgs) -> ListFilter {
    ListFilter {
        library: a.library,
        kind: a.kind.map(Into::into),
        tag: a.tag,
    }
}

fn describe_spec(spec: &VarSpec) -> String {
    let mut parts = Vec::new();
    if let Some(from) = &spec.from {
        parts.push(format!("from {from}"));
    }
    if let Some(hint) = &spec.hint {
        parts.push(hint.clone());
    }
    if !spec.choices.is_empty() {
        parts.push(format!("one of: {}", spec.choices.join(", ")));
    }
    if let Some(d) = &spec.default {
        parts.push(format!("default: {d}"));
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!(" ({})", parts.join("; "))
    }
}
