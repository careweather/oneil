# .githooks

Local git hooks for the oneil workspace.

## Setup

Point git at this directory once after cloning:

```sh
git config core.hooksPath .githooks
```

## Hooks

- **pre-commit** — when Rust files are staged: run `cargo fmt`, then regenerate
  `packages/ts-interfaces` (`./scripts/generate-ts-interfaces.sh`) and stage
  any binding changes.
- **post-commit** — when the commit touched Rust: run `cargo clippy --fix`
  and amend with any fixes.
- **pre-push** — when the pushed commits touch Rust: run `cargo fmt`, then
  clippy and the full test suite with warnings denied.
