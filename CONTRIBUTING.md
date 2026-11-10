# Contributing

Thank you for your help. PromptShelf is in early development, so open an issue before a large change.

## Licence
PromptShelf uses the MIT licence. When you send a pull request, you agree that your contribution is licensed under MIT too. There is no contributor licence agreement.

## Before you send a pull request
1. Run `cargo fmt --all`.
2. Run `cargo clippy --all-targets -- -D warnings`.
3. Run `cargo test`.

## Dependencies
Use only crates with a permissive licence: MIT, Apache-2.0, BSD or ISC. CI checks this with `cargo deny`.

## Commits
Use [Conventional Commits](https://www.conventionalcommits.org/), for example `feat(cli): add shelf search`.
