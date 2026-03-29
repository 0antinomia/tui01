# Release Checklist

This is the canonical local-first release path for the `0.3.0` reset release. Complete these steps in order. Optional automation may wrap the same commands later, but that automation is non-blocking and not required for shipping this release.

## Prerequisites

- `cargo`
- `rustc`
- `git`
- `rg`
- A crates.io token when you are ready to run the real publish step

## Checklist

1. Confirm the worktree is ready to ship with `git status --short` and resolve any intentional local changes before publishing.
2. Confirm the release version is aligned across `Cargo.toml`, `CHANGELOG.md`, and `docs/VERSIONING.md`, and review `docs/MIGRATION.md` for the breaking-reset guidance that ships with `0.3.0`.
3. Run the full release gate with `./scripts/verify_release_readiness.sh`.
4. Review the packaged file list reported by the release gate and confirm the shipped artifact includes `README.md`, `CHANGELOG.md`, `docs/GETTING_STARTED.md`, `docs/MIGRATION.md`, `docs/VERSIONING.md`, `docs/RELEASE_CHECKLIST.md`, and `examples/host_template.rs`.
5. Authenticate to crates.io if needed so the final publish command can succeed on this machine.
6. Publish the crate with `cargo publish`.
7. Create the release tag for `0.3.0`, push the tag, and publish any repo-hosted release notes that correspond to `CHANGELOG.md`.

Future automation may wrap the same script and checklist order, but no GitHub Actions workflow, trusted publishing setup, or other external platform configuration is required to ship this reset release.
