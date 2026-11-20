# Contributing to Oneil

Thank you for your interest in contributing to the Oneil programming language!
This document provides guidelines and instructions for contributing to the
project.

*This document is a work in progress. If you have any suggestions for improvement, feel free to open a pull request!*

## Language Development Setup

1. Clone the repository
2. Install latest stable Rust toolchain (or use `nix develop` if in a Nix environment)
3. Run `cargo build` to verify your setup

For development, you can use these Cargo commands:

- Run tests:

  ```sh
  cargo test
  ```

- Check for compilation errors without producing an executable:

  ```sh
  cargo check
  ```

- Format code:

  ```sh
  cargo fmt
  ```

- Run linter:

  ```sh
  cargo clippy
  ```

You can also run the following developer commands built into Oneil:

- Print the AST that is constructed from an Oneil file:

  ```sh
  cargo run -- dev print-ast path/to/model.on
  ```

- Print the IR that is constructed from an Oneil model

  ```sh
  cargo run -- dev print-ir path/to/model.on
  ```

In addition, you will want to install the
[`rust-analyzer`](https://open-vsx.org/extension/rust-lang/rust-analyzer)
VS Code extension in order to help you develop in Rust.

If you are using `rust-analyzer` in VS Code, ensure that you are using the
`clippy` linter by [updating your
settings](https://users.rust-lang.org/t/how-to-use-clippy-in-vs-code-with-rust-analyzer/41881)

> [!TIP]
> You can run the linter manually using `cargo clippy`. If you are ever failing
> the lint check on Github and `cargo clippy` isn't producing any output,
> run `rustup install stable` to upgrade to the latest version of Rust.

## LSP and Tooling Development Setup

1. Clone the repository
2. Install the following dependencies (or use `nix develop` if in a Nix environment):
    - latest stable Rust toolchain
    - `nodejs` and `npm`
    - VS Code (or a compatible editor such as Cursor)
3. Run `cargo build` to compile Oneil
4. Run `npm install` in `vscode/` and in `vscode/model-renderer/`
5. Open the `vscode/` folder, run the **Build All** task, then press `F5` to launch the Extension Development Host
6. Open any `.on` file you want to try the LSP on! The [`examples` directory](./examples) has example models

See [`vscode/docs/CONTRIBUTING.md`](vscode/docs/CONTRIBUTING.md) for details, including how to point the extension at your local CLI build.

## System Architecture

- [`docs/architecture/README.md`](docs/architecture/README.md) gives an overview of the crates and the libraries they use. The code itself is found in [`src/`](src/).
- [`docs/architecture/compiler-pipeline.md`](docs/architecture/compiler-pipeline.md) describes each compiler phase, what it validates, what is fixed after it, and where new checks belong.
- [`docs/architecture/vscode-extension.md`](docs/architecture/vscode-extension.md) describes the VS Code extension and the Rendered View.
- Significant design decisions are recorded in [`docs/decisions/`](docs/decisions/README.md), and the language is specified in [`docs/specs/`](docs/specs/).

## Coding Standards

Rust code should follow [`docs/CODING_STANDARDS.md`](docs/CODING_STANDARDS.md). Oneil models (examples, fixtures, and the guide) should follow [`docs/ONEIL_CODING_STANDARDS.md`](docs/ONEIL_CODING_STANDARDS.md).

## Git hooks

Run `git config core.hooksPath .githooks` once after cloning. The hooks (see [`.githooks/README.md`](.githooks/README.md)) do the following when Rust files are involved:

- **pre-commit:** run `cargo fmt` and regenerate the TypeScript bindings in `packages/ts-interfaces`, staging the results;
- **post-commit:** run `cargo clippy --fix` and amend the commit with any fixes;
- **pre-push:** run clippy and the full test suite with warnings denied.

## Building the user guide

The user guide, found in [docs/guide](./docs/guide/), is built using `mdbook`.
It also uses the `mdbook-mermaid` plugin. Both can be installed using

```bash
cargo install mdbook mdbook-mermaid
```

See [mdBook documentation](https://rust-lang.github.io/mdBook/index.html) and
[`mdbook-mermaid` documentation](https://github.com/badboy/mdbook-mermaid) for
more details on how to use those tools.

To edit the syntax highlighting, edit the grammar defined in
[theme/highlight.js](./docs/guide/theme/highlight.js).

## Continuous Integration

GitHub Actions workflows live in [`.github/workflows/`](.github/workflows/).
They cover everyday checks, PR-only extras, docs deployment, and releases.

### Rust (every push and pull request)

[`.github/workflows/rust.yml`](.github/workflows/rust.yml) builds the workspace,
runs tests, Clippy, and `cargo fmt --check`. Warnings are treated as errors
(`RUSTFLAGS=--deny warnings`). Fuzz targets are not run here (they would not
finish in CI).

The local equivalent is roughly:

```sh
cargo build --all-targets --all-features
cargo test --all-features
cargo clippy --all-targets --all-features
cargo fmt --check
```

### Pull request extras

[`.github/workflows/rust-pr.yml`](.github/workflows/rust-pr.yml) runs only on
pull requests:

- **Fuzz targets** — each listed `oneil_output` fuzz target runs for a fixed
  time window on nightly Rust. On failure, fuzz artifacts are uploaded for
  debugging.

- **Unused dependencies** — `cargo udeps` on nightly to catch crates that are
  declared but unused.

### Editor tooling and generated types (when their paths change)

- [`.github/workflows/vscode-extension.yml`](.github/workflows/vscode-extension.yml) runs the extension's Vitest suite (`npm test` in `vscode/`).
- [`.github/workflows/model-renderer.yml`](.github/workflows/model-renderer.yml) builds the Rendered View webview and runs its tests (`vscode/model-renderer/`).
- [`.github/workflows/ts-interfaces.yml`](.github/workflows/ts-interfaces.yml) runs `./scripts/check-ts-interfaces.sh`, which fails if the committed TypeScript bindings in `packages/ts-interfaces` differ from what the Rust types generate.
- [`.github/workflows/model-test-report-action.yml`](.github/workflows/model-test-report-action.yml) typechecks, lints, tests, and checks the built output of the [`model-test-report`](actions/model-test-report/README.md) GitHub Action.

### Semgrep (pull requests)

[`.github/workflows/semgrep.yml`](.github/workflows/semgrep.yml) scans Rust,
TypeScript, React, and Python changes with reviewed public Semgrep rules. It
compares the pull request with its exact base commit. Existing findings do not
block unrelated changes.

To reproduce the scan locally:

1. Install the pinned CLI with `pipx install semgrep==1.175.0`.
2. Fetch the public `p/rust`, `p/typescript`, and `p/python` configurations.
3. Verify their SHA-256 values against the values in the workflow.
4. Run `semgrep scan` with those local files, `--strict`, `--error`, and
   `--baseline-commit "$(git merge-base HEAD origin/main)"`.

The workflow uses only read access. It does not require a Semgrep account or
token.

### Coding standards review (advisory)

[`.github/workflows/coding-standards-review.yml`](.github/workflows/coding-standards-review.yml)
runs when a PR changes files under `src/`. It installs the
[Cursor CLI](https://cursor.com/docs/cli) and runs it as a one-shot agent
(`cursor-agent -p --output-format json`) that reviews the diff against
[`docs/CODING_STANDARDS.md`](docs/CODING_STANDARDS.md) using the instructions
in [`.github/coding-standards-review/prompt.md`](.github/coding-standards-review/prompt.md),
then posts (or updates) a sticky PR comment. Style findings do not fail the
job; missing secrets or agent/infra failures do. Requires a `CURSOR_API_KEY`
repository secret. It skips PRs whose head branch starts with
`weekly-quality/`, since those are opened by the weekly quality review below
and re-reviewing agent-generated changes with another agent is redundant.
The review is deliberately read-only: the agent has no `GH_TOKEN` in this
job (posting the comment is left to the separate `post-review` job), and a
`permissions.deny` entry in `~/.cursor/cli-config.json` technically blocks
it from running `gh` or any mutating `git` subcommand (push, commit, branch,
checkout, etc.), while still allowing read-only ones (`git diff`, `git show`,
...) that it needs to compute the diff itself — see
[Cursor CLI permissions](https://cursor.com/docs/cli/reference/permissions).

### Weekly quality review (advisory + automated fixes)

[`.github/workflows/weekly-quality-review.yml`](.github/workflows/weekly-quality-review.yml)
runs on a weekly schedule (Mondays) and via manual `workflow_dispatch`. It
picks a random crate under `src/` and a random review focus
(implementation, architecture, or testing), then uses the Cursor CLI to:

1. Review the crate and write the findings to a markdown file.
2. Classify the recommended changes as `simple` or `complex` (resuming the
   same agent session).
3. If `simple`, have the agent edit the files on a new
   `weekly-quality/<crate>-<focus>` branch (created by the workflow, not the
   agent). The workflow itself then commits, pushes, and opens the PR with
   `gh pr create`, using a commit subject/PR description generated by a
   follow-up (read-only) agent call. Keeping branch naming, commit, push, and
   PR creation as plain scripted steps — rather than trusting the agent to
   run them correctly from a prompt — guarantees the `weekly-quality/` branch
   prefix that the coding-standards-review skip condition above depends on.
   The agent is also technically prevented (not just asked) from running any
   `git`/`gh` command itself, via a `deny` entry in `~/.cursor/cli-config.json`
   written at the start of the job — see [Cursor CLI permissions](https://cursor.com/docs/cli/reference/permissions).
4. If `complex`, open a GitHub issue with the review instead, for a human to
   triage.

This workflow pushes branches and opens PRs using a `WEEKLY_QUALITY_PAT`
repository secret (a fine-grained PAT with `Contents` and `Pull requests`
write access to this repo) rather than the default `GITHUB_TOKEN`. GitHub
does not trigger other workflows (e.g. [`rust.yml`](.github/workflows/rust.yml),
[`rust-pr.yml`](.github/workflows/rust-pr.yml)) for pushes/PRs made with the
default `GITHUB_TOKEN`, so a real token is needed for the opened PRs to get
normal CI. Also requires the `CURSOR_API_KEY` secret.

### User guide (GitHub Pages)

[`.github/workflows/guide.yml`](.github/workflows/guide.yml) builds the mdBook
user guide under `docs/guide/` and deploys it to
[oneil-lang.org](https://oneil-lang.org) on pushes to `main` that change the
guide or the workflow (and via manual `workflow_dispatch`).

### Releases

[`.github/workflows/release.yml`](.github/workflows/release.yml) runs when a
version tag matching `v*` is pushed. It builds release binaries for Linux,
Windows, and macOS, then attaches them to a GitHub Release with generated notes.

## Resources

- [Crafting Interpreters](https://craftinginterpreters.com/) - If you've never
  worked on a programming language before, this is a great resource for
  understanding how to build a programming language!
