# AGENTS Guide for `git-wippy`

This file tells AI coding agents how to work safely and effectively in this repository.

## Project Overview

- Project type: Rust CLI tool (`git-wippy`)
- Entry point: `src/main.rs`
- Command modules: `src/commands/`
- Shared utilities: `src/utils/`
- Integration tests: `tests/cli.rs`
- Locales: `locales/*.ftl`

## Golden Rules

1. Never change behavior without tests or clear rationale.
2. Prefer small, reviewable patches over large rewrites.
3. Keep output and localization consistent across locales.
4. Avoid broad `#[allow(...)]` attributes; use narrow exceptions only.
5. Never use type-safety bypasses or suppression hacks.

## Definition of Done

A change is complete only if all are true:

1. Code compiles locally.
2. `cargo fmt --all -- --check` passes.
3. `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery` passes.
4. `cargo test --workspace --all-features --locked` passes.
5. Documentation builds: `cargo doc --workspace --all-features --no-deps`.
6. mdBook builds: `mdbook build book`.
7. Any behavior change updates docs/tests/changelog as needed.

## Scope Guardrails

In scope for routine work:

- Rust source in `src/`
- Tests in `tests/`
- CI files in `.github/workflows/`
- Docs/specs (`README.md`, `.specs/`)

Out of scope unless explicitly requested:

- Release process redesign
- Cross-platform packaging overhaul
- Large architectural rewrites across all commands

## Preferred Workflow for Agents

1. Read relevant command module(s) and tests first.
2. Reuse existing patterns in `src/commands/*` and `src/utils/*`.
3. Apply smallest possible patch.
4. Run verification commands locally.
5. Report exactly what changed and which checks ran.

## Style and Quality Expectations

- Use idiomatic Rust 2021 patterns already present in codebase.
- Propagate errors with `anyhow::Result`; avoid `unwrap`/`expect` in production paths.
- Keep functions focused and testable.
- Add comments only when logic is non-obvious.
- Keep user-visible strings localized through existing i18n flow.

## Testing Guidance

- For command behavior changes, update or add integration coverage in `tests/cli.rs`.
- For utility-level logic, add focused unit tests close to the module when practical.
- Prefer deterministic tests; avoid time/network flakiness.

## CI and Tooling

- Main CI workflow: `.github/workflows/main.yml`.
- Required CI jobs: fmt, clippy, test (stable + MSRV lane), build, docs, security audit.
- Documentation system: mdBook (`book.toml`, `book/src/`).
- Keep action versions current and avoid deprecated actions.

## Commit Policy

- Use atomic commits: one logical change per commit, with tests/docs paired to that change.
- Use Conventional Commits format: `type(scope): subject`.
- Allowed types: `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `ci`, `chore`.
- Keep subject imperative, concise, and lower-case (except proper nouns).
- Do not mix unrelated refactors with behavior changes in the same commit.
- If a change is too large, split into ordered commits that each pass checks.

## Safety Constraints

- Do not commit secrets, tokens, or machine-specific data.
- Do not run destructive git history commands.
- Do not silently weaken lint/test gates to make CI pass.

## When Unsure

1. Follow existing repository patterns first.
2. Choose the less risky, less invasive option.
3. Document assumptions and trade-offs in the change summary.
