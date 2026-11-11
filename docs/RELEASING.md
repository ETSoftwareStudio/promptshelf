# Releasing

Releases use [release-please](https://github.com/googleapis/release-please). Commit messages follow Conventional Commits, and they decide the next version.

## How a release happens
1. Merge pull requests into `main`. A `feat:` or `fix:` commit makes release-please open or update the release PR, "chore(main): release X.Y.Z". It holds the version bump and the changelog.
2. The release workflow updates `Cargo.lock` and `app/src-tauri/Cargo.lock` on the release PR and starts CI on it.
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

## The first release
v0.1.0 used `"release-as": "0.1.0"` in `release-please-config.json`, because release-please would start at 1.0.0. That line is gone, so later versions follow the commits.

## Where the version lives
release-please updates every file listed in `release-please-config.json`:
- `Cargo.toml`: the lines that end with `# x-release-please-version`.
- `npm/promptshelf/package.json`: `version`.
- `app/src-tauri/Cargo.toml`: the line that ends with `# x-release-please-version`. The desktop app shows this version in Settings → About.
- `app/package.json`: `version`.

`app/src-tauri/tauri.conf.json` has no `version`, so Tauri uses the version in `app/src-tauri/Cargo.toml`. release-please writes JSON files again from the start, and the app's formatter rejects how it writes that file.

The two `Cargo.lock` files hold the versions of the path crates, so the release workflow runs `cargo update --workspace` for each. `app/pnpm-lock.yaml` does not hold the app's own version, so it needs no change.

To check a bump by hand, set a new version in every file above, then run:

```
cargo update --workspace
cargo update --workspace --manifest-path app/src-tauri/Cargo.toml
cargo metadata --locked --format-version 1 --manifest-path app/src-tauri/Cargo.toml > /dev/null
(cd app && vp install --frozen-lockfile)
```

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
7. Publish the npm package once by hand from `npm/promptshelf`, then add the trusted publisher for it. npm rejected the name `promptshelf` for v0.1.0, because it is too similar to `prompt-shelf`. Pick a scoped name first; until then, the npm job fails when `PUBLISH` is `true`.
8. Create `ETSoftwareStudio/homebrew-tap` and the `HOMEBREW_TAP_TOKEN` secret.
9. Set the repository variable `PUBLISH` to `true`.
