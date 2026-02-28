# Modernization Plan

This page tracks the active modernization strategy for `git-wippy`.

## Goals

1. Keep CI reliable and deterministic.
2. Keep code quality strict but practical.
3. Modernize dependencies in safe, reviewable batches.

## Quality Gates

All pull requests should pass:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings -W clippy::pedantic -W clippy::nursery`
3. `cargo test --workspace --all-features --locked`
4. `cargo build --workspace --release --locked`
5. `cargo doc --workspace --all-features --no-deps`
6. `mdbook build book`

## Upgrade Sequence

1. Stabilize CI and documentation checks.
2. Pin and document toolchain policy (`rust-version` and MSRV lane).
3. Update dependencies in small batches with full verification after each batch.
4. Harden supply chain checks (audit policy, action pinning as needed).

## Safety Rules

- No broad lint suppression.
- No test deletion to hide failures.
- Keep behavior-changing work paired with tests and docs updates.
