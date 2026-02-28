# Contributing Workflow

## Development Loop

1. Make a focused change.
2. Run local checks.
3. Submit a PR with clear scope.

## Required Local Checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings -W clippy::pedantic -W clippy::nursery
cargo test --workspace --all-features --locked
cargo doc --workspace --all-features --no-deps
mdbook build book
```

## Commit Policy

Use atomic commits and Conventional Commits:

- Atomic commit: one logical change per commit.
- Conventional format: `type(scope): subject`
- Allowed types: `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `ci`, `chore`

Examples:

- `fix(ci): repair rust toolchain setup in workflow`
- `docs(book): add modernization roadmap chapter`
- `refactor(save): simplify branch naming flow`

## PR Expectations

- Explain why the change is needed.
- Mention tests/docs touched.
- Avoid mixing unrelated refactors with behavior changes.
