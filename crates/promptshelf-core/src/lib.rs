//! Core library for PromptShelf.
//!
//! This crate owns the prompt files, templates, libraries and sync. Every
//! surface uses it: the `shelf` CLI, the terminal app and the desktop app.

pub mod error;
pub mod fsutil;
pub mod prompt;

pub use error::{Error, Result};
pub use prompt::{Kind, Prompt, VarSpec};

/// The version of this crate, from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
