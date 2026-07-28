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
    - `nodejs`
    - `npm` or `pnpm`
    - `vscode`
3. Run `cargo build` to compile Oneil
4. Run `cd vscode` followed by `pnpm run compile` to bundle Oneil as a vscode LSP plugin
5. If not already opened, open VSCode in the current directory
6. Press the `F5` key to launch VSCode's Extension Development Host (should open a separate window)
7. Open any `.on` file you want to try the LSP on! Have a look at [the `test` directory](./test) for some example files

## System Architecture

The architecture of the system is described in [`docs/architecture/README.md`](docs/architecture/README.md). The code itself is found in [`src-rs/`](src-rs/).

## Coding Standards

Code should follow the principles laid out in
[`docs/principles.md`](docs/principles.md).

## Git hooks

Run `git config core.hooksPath .githooks` to set up some automated checks to run before committing and pushing.

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

### Coding standards review (advisory)

[`.github/workflows/coding-standards-review.yml`](.github/workflows/coding-standards-review.yml)
runs when a PR changes files under `src-rs/`. It installs the
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
picks a random crate under `src-rs/` and a random review focus
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
user guide under `docs/guide/` and deploys it to GitHub Pages on pushes to the
`gh-pages` branch (and via manual `workflow_dispatch`).

### Releases

[`.github/workflows/release.yml`](.github/workflows/release.yml) runs when a
version tag matching `v*` is pushed. It builds release binaries for Linux,
Windows, and macOS, then attaches them to a GitHub Release with generated notes.

## Resources

- [Crafting Interpreters](https://craftinginterpreters.com/) - If you've never
  worked on a programming language before, this is a great resource for
  understanding how to build a programming language!
