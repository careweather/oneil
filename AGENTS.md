# Agent Instructions

This file orients AI coding agents working in this repository. Detailed Cursor rules live in [`.cursor/rules/`](.cursor/rules/); they apply to every agent, not only Cursor.

## Read first

- [`CONTRIBUTING.md`](CONTRIBUTING.md): setup, commands, git hooks, and CI.
- [`docs/architecture/README.md`](docs/architecture/README.md): crate map and the libraries to use for each concern.
- [`docs/architecture/compiler-pipeline.md`](docs/architecture/compiler-pipeline.md): what each compiler phase consumes, produces, and validates, what is fixed after it, and where new checks and transformations belong. Read it before changing anything under `src/`.
- [`docs/architecture/vscode-extension.md`](docs/architecture/vscode-extension.md): read it before changing anything under `vscode/` or `packages/ts-interfaces/`.
- [`docs/CODING_STANDARDS.md`](docs/CODING_STANDARDS.md) for Rust, and [`docs/ONEIL_CODING_STANDARDS.md`](docs/ONEIL_CODING_STANDARDS.md) for Oneil models.
- [`docs/decisions/`](docs/decisions/README.md): record significant decisions as new ADRs from `template.md`.

## Rules

- [`.cursor/rules/oneil-compiler.mdc`](.cursor/rules/oneil-compiler.mdc): the Rust workspace, pipeline invariants, specs, ADRs, and the git workflow.
- [`.cursor/rules/vscode-extension.mdc`](.cursor/rules/vscode-extension.mdc): the extension and the Rendered View.

## Workflow essentials

- Work in a worktree under `.worktrees/`, and run `git config core.hooksPath .githooks` in it.
- Before finishing Rust changes, run `cargo clippy --all-targets --all-features` and `cargo test --all-features`.
- After changing Rust types sent to the editor, run `./scripts/generate-ts-interfaces.sh`.
- Don't leave `TODO`-style comments; record deferred work in the pull request (or, for external contributors, in a linked GitHub issue).
- Keep `docs/` current when behavior or organization changes: update the spec and the guide for language changes, and the architecture docs for phase or structure changes.
