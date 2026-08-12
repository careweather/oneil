# Appendix C: Continuous Integration

Oneil’s GitHub Actions workflows live under [`.github/workflows/`](https://github.com/careweather/oneil/tree/main/.github/workflows).
They keep the Rust toolchain, editor packages, generated TypeScript bindings,
and release artifacts in sync. This appendix is a map of what runs when — not a
substitute for reading the workflow YAML itself.

Locally, point git at [`.githooks/`](https://github.com/careweather/oneil/tree/main/.githooks)
(`git config core.hooksPath .githooks`) so format, clippy, and TypeScript
binding regeneration run before you push. CI will still enforce the same
checks.

## Rust core

| Workflow | When it runs | What it does |
|----------|--------------|--------------|
| `rust.yml` | Every push and pull request | `cargo build`, `cargo test`, `clippy`, and `rustfmt --check` with warnings denied |
| `rust-pr.yml` | Pull requests | Timed `cargo fuzz` runs for `oneil_output` targets, plus an unused-dependency check (`cargo-udeps`) |

Fuzz targets are listed explicitly in `rust-pr.yml`. Failures upload fuzz
artifacts for debugging. Unit and integration tests intentionally skip fuzz
targets so `cargo test` does not run forever.

## Generated TypeScript bindings

JSON wire types used by the LSP rendered view and by `oneil test --format json`
are generated with `ts-rs` into `packages/ts-interfaces`.

| Workflow | When it runs | What it does |
|----------|--------------|--------------|
| `ts-interfaces.yml` | Changes under `src-rs/`, `packages/ts-interfaces/`, or the generate/check scripts | Runs `./scripts/check-ts-interfaces.sh` so committed bindings match the Rust types |

Regenerate locally with `./scripts/generate-ts-interfaces.sh` (also invoked by
the pre-commit hook when Rust files are staged).

## VS Code model renderer

| Workflow | When it runs | What it does |
|----------|--------------|--------------|
| `model-renderer.yml` | Changes under `vscode/model-renderer/`, `packages/ts-interfaces/`, or Rust sources | `npm ci`, build/typecheck, and Vitest for the rendered-view webview |

## Downstream model-test-report Action

The reusable Action at `actions/model-test-report` installs a pinned Oneil ref,
runs `oneil test --format json`, and can diff base vs. head results for
downstream model repos. See that Action’s README for consumer usage.

| Workflow | When it runs | What it does |
|----------|--------------|--------------|
| `model-test-report-action.yml` | Changes under `actions/model-test-report/`, related scripts, or Rust sources | Typecheck, lint, test, and `npm run check-dist` so the committed `dist/` bundle stays current |

## Releases

| Workflow | When it runs | What it does |
|----------|--------------|--------------|
| `release.yml` | Push of a version tag (`v*`, e.g. `v1.0.0`) | Cross-builds the `oneil` CLI for Linux, Windows, and macOS and attaches archives to the GitHub Release |

Tagging is the release trigger; bump versions in `Cargo.toml` / `pyproject.toml`
(and related packages) in the same change set as the changelog entry.

## Docs site

| Workflow | When it runs | What it does |
|----------|--------------|--------------|
| `guide.yml` | Push to `gh-pages` (or manual `workflow_dispatch`) | Builds this mdBook guide and deploys it to GitHub Pages |

## Automated reviews

These jobs use the Cursor CLI. They need repository secrets (`CURSOR_API_KEY`,
and for the weekly job a `WEEKLY_QUALITY_PAT`) and are advisory or
maintenance-oriented rather than merge gates for ordinary contributors.

| Workflow | When it runs | What it does |
|----------|--------------|--------------|
| `coding-standards-review.yml` | PRs that touch `src-rs/` | Posts a sticky PR comment reviewing the diff against `docs/CODING_STANDARDS.md` (does not fail on style findings) |
| `weekly-quality-review.yml` | Mondays (UTC) or manual dispatch | Picks a random crate and focus, writes a quality review, and opens a PR |

Coding-standards review skips PRs whose head branch starts with
`weekly-quality/`, so the weekly agent is not reviewed by itself.
