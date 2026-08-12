# Appendix C: Continuous Integration

Once your models declare [`test:`](./06-tests.md) checks, run them in CI the
same way you run them locally. This appendix is for **model repositories**
(repos that contain `.on` / `.one` files).

Pin Action refs and Oneil versions to the same release tag (for example
`v1.0.0`) so CI does not silently move under you.

## Recommended: `model-test-report`

Use
[`careweather/oneil/actions/model-test-report`](https://github.com/careweather/oneil/tree/main/actions/model-test-report)
as the default CI integration. It installs a pinned Oneil, runs
`oneil test --format json` on discovered models, writes a Markdown report to
the job summary, and can **diff a PR head against its base** so the report
highlights regressions and fixes rather than only a raw pass/fail count.

### Single checkout (push / PR)

```yaml
name: Oneil model tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
      # Only needed if models import Python functions:
      - uses: actions/setup-python@v5
        with:
          python-version: "3.12"

      - uses: careweather/oneil/actions/model-test-report@v1.0.0
        with:
          oneil-ref: v1.0.0
          model-dir: model
```

The Action builds Oneil from `oneil-ref`, so the calling workflow must install
Rust first. It does not install Python itself — add `setup-python` only when
models call into Python.

### Compare a PR against its base

```yaml
name: Oneil model test report
on:
  pull_request:

jobs:
  test:
    runs-on: ubuntu-latest
    permissions:
      contents: read
    steps:
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-python@v5
        with:
          python-version: "3.12"

      - uses: actions/checkout@v4
        with:
          path: head

      - uses: actions/checkout@v4
        with:
          ref: ${{ github.event.pull_request.base.sha }}
          path: base

      - uses: careweather/oneil/actions/model-test-report@v1.0.0
        id: report
        with:
          oneil-ref: v1.0.0
          head-dir: head
          base-dir: base
          model-dir: model
          head-label: ${{ github.event.pull_request.head.ref }}
          base-label: ${{ github.event.pull_request.base.ref }}
          report-path: oneil-test-report.md

      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: oneil-test-report
          path: ${{ steps.report.outputs.report-path }}
```

With `base-dir` set, the Action fails the job when there are **regressions**,
new failures, or new diagnostics (not merely because the base branch already
had failing tests). Outputs include `has-problems`, `report` (Markdown), and
`report-path` for posting a PR comment or uploading an artifact.

### Useful inputs

| Input | Purpose |
|-------|---------|
| `oneil-ref` | Tag / branch / SHA of Oneil to install (required) |
| `model-dir` | Directory of `.on` / `.one` files (default `model`) |
| `models` | Explicit comma-separated file list (skips auto-discovery) |
| `skip-models` | Files to exclude from auto-discovery |
| `timeout-seconds` | Per-model timeout (default `120`) |
| `fail-on-problems` | Set `false` to report without failing the job |
| `report-path` | Also write the Markdown report to a file |

Auto-discovery only considers **top-level** `.on` / `.one` files in
`model-dir` that declare at least one `test:` block. Submodel tests reached
via imports are covered when the Action runs `oneil test --recursive` on those
entry points. Design files (`.one`) that declare their own tests are included.

Full reference:
[Action README](https://github.com/careweather/oneil/blob/main/actions/model-test-report/README.md).

## Install the CLI: `install-oneil`

When you need `oneil` on `PATH` for custom steps (or a minimal workflow of your
own), use
[`careweather/oneil/actions/install-oneil`](https://github.com/careweather/oneil/tree/main/actions/install-oneil).
It downloads the pre-built CLI from a GitHub Release for the runner’s OS and
architecture — no Rust toolchain required.

```yaml
name: Oneil model tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: careweather/oneil/actions/install-oneil@v1.0.0
        with:
          version: v1.0.0

      # Only needed if models import Python functions:
      - uses: actions/setup-python@v5
        with:
          python-version: "3.12"

      - run: oneil test --recursive model/radar.on
```

`oneil test` exits with status **1** if there were any error diagnostics or any
failing test, and **0** otherwise. Use `--format json` when a later step will
parse the report.

| Input | Purpose |
|-------|---------|
| `version` | Release tag to install (required), e.g. `v1.0.0` |
| `github-token` | Optional; defaults to the job token |

Outputs: `version` (from `oneil --version`) and `oneil-path`.

Full reference:
[Action README](https://github.com/careweather/oneil/blob/main/actions/install-oneil/README.md).

## Tips

- **Prefer `model-test-report` for model repos.** Use `install-oneil` when you
  need a plain CLI for scripts or a hand-rolled test loop.
- **Pin versions.** Treat Oneil like a compiler: bump Action tags and
  `oneil-ref` / `version` together when you adopt a new release.
- **Keep tests close to requirements.** CI is most useful when `test:` lines
  encode margins and constraints you care about — see [Tests](./06-tests.md).
- **Python models.** If imports need packages, install them in the workflow
  before the Action (or before `oneil test`).
- **Other CI systems.** Download a release binary from
  [Releases](https://github.com/careweather/oneil/releases) (see
  [Installation](./02-installation.md)), then run `oneil test --recursive …`
  and rely on the exit code.
