//! Core library for PromptShelf.
//!
//! This crate owns the prompt files, templates, libraries and sync. Every
//! surface uses it: the `shelf` CLI, the terminal app and the desktop app.

/// The version of this crate, from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        assert!(!VERSION.is_empty());
    }
}
