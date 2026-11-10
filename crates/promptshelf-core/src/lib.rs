//! Core library for PromptShelf.
//!
//! This crate owns the prompt files, templates, libraries and sync. Every
//! surface uses it: the `shelf` CLI, the terminal app and the desktop app.

pub mod error;
pub mod fsutil;

pub use error::{Error, Result};

/// The version of this crate, from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
