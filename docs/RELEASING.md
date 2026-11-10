# Releasing

Releases use [release-please](https://github.com/googleapis/release-please). Commit messages follow Conventional Commits, and they decide the next version.

## How a release happens
1. Merge pull requests into `main`. A `feat:` or `fix:` commit makes release-please open or update the release PR, "chore(main): release X.Y.Z". It holds the version bump and the changelog.
2. The release workflow updates `Cargo.lock` on the release PR and starts CI on it.
3. Merge the release PR. The workflow then:
   - tags `vX.Y.Z` and creates the GitHub release;
   - builds `shelf` for macOS (Apple silicon and Intel), Linux (x86_64, static musl) and Windows (x86_64);
   - uploads each archive with its SHA-256, and `checksums.txt`;
   - publishes to crates.io and npm, and updates the Homebrew tap, when the repository variable `PUBLISH` is `true`.

| Commit type | Version change before 1.0 |
| --- | --- |
| `fix:`, `perf:` | patch |
| `feat:` | minor |
| `feat!:` or `BREAKING CHANGE:` | minor (major after 1.0) |
| `build:`, `ci:`, `docs:`, `chore:`, `test:` | none |

## Where the version lives
release-please updates every file listed in `release-please-config.json`:
- `Cargo.toml`: the lines that end with `# x-release-please-version`.
- `README.md`: the install snippet between the `x-release-please` markers.
- `npm/promptshelf/package.json`: `version`.

## Secrets
None are stored for crates.io or npm. Both use trusted publishing with GitHub's OIDC identity. The Homebrew tap needs `HOMEBREW_TAP_TOKEN`: a fine-grained token with write access to `ETSoftwareStudio/homebrew-tap` only.

## Before the first public release
Do these in order. Each one is hard to undo, so a person does them.

1. Scan the whole history: `gitleaks git --redact .` must find nothing.
2. Check that no tracked file names a private repo, a client or a real person. Examples use made-up names only.
3. Switch the repo to public.
4. In `.github/workflows/ci.yml` and `release.yml`, change the macOS runners from `[self-hosted, macOS, ARM64]` to `macos-latest`. Public repos never use the self-hosted runner.
5. Add `aarch64-unknown-linux-musl` on `ubuntu-24.04-arm` to the build matrix.
6. Publish `promptshelf-core` and `promptshelf` to crates.io once by hand (`cargo publish`), because trusted publishing needs the crates to exist. Then add this repo and `release.yml` as a trusted publisher for both crates.
7. Publish the npm package once by hand from `npm/promptshelf`, then add the trusted publisher for it.
8. Create `ETSoftwareStudio/homebrew-tap` and the `HOMEBREW_TAP_TOKEN` secret.
9. Set the repository variable `PUBLISH` to `true`.
