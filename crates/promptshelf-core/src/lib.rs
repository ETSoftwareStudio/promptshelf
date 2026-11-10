//! Core library for PromptShelf.
//!
//! This crate owns the prompt files, templates, libraries and sync. Every
//! surface uses it: the `shelf` CLI, the terminal app and the desktop app.

pub mod config;
pub mod edit;
pub mod error;
pub mod fill;
pub mod forge;
pub mod fsutil;
pub mod git;
pub mod inbox;
pub mod pins;
pub mod prompt;
pub mod proposals;
pub mod resolve;
pub mod setup;
pub mod shelf;
pub mod state;
pub mod sync;
pub mod template;
pub mod tools;
pub mod write;

pub use config::Config;
pub use error::{Error, Result};
pub use prompt::{Kind, Prompt, VarSpec};
pub use shelf::{ListFilter, Shelf};

/// The version of this crate, from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The version of the `--json` output. A breaking change to the output raises it.
pub const JSON_SCHEMA: u32 = 1;
