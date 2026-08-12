# Appendix C: Continuous Integration

Once your models declare [`test:`](./06-tests.md) checks, you can run them in
CI the same way you run them locally. This appendix shows how to wire Oneil
into a GitHub Actions workflow for a **model repository** (a repo that
contains `.on` / `.one` files, not the Oneil language repo itself).

## What to run

```sh
oneil test path/to/model.on
oneil test --recursive path/to/model.on   # include tests in imported submodels
oneil test --format json path/to/model.on # machine-readable report for tooling
```

`oneil test` exits with status **1** if there were any error diagnostics or any
failing test, and **0** otherwise — so a bare `oneil test …` step already fails
the job when something is wrong.

JSON mode (`--format json`) prints a structured report (diagnostics plus
per-test pass/fail, with dependency values on failures). Prefer that when a
script or Action will parse the result; keep the default text format for
humans reading the log.

Install a released CLI binary (see [Installation](./02-installation.md)) or
build from a pinned Oneil ref in the workflow. Pin the Oneil version your
models are validated against so CI does not silently move under you.

## Minimal workflow: run tests on every push

```yaml
name: Oneil model tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Oneil
        run: |
          TAG=v1.0.0
          ARCHIVE="oneil-${TAG}-x86_64-unknown-linux-gnu.tar.gz"
          curl -fsSL \
            "https://github.com/careweather/oneil/releases/download/${TAG}/${ARCHIVE}" \
            -o oneil.tar.gz
          tar -xzf oneil.tar.gz
          sudo mv oneil /usr/local/bin/
          oneil --version

      # Only needed if models import Python functions:
      - uses: actions/setup-python@v5
        with:
          python-version: "3.12"

      - name: Run model tests
        run: |
          # Adjust paths to your entry-point models.
          oneil test --recursive model/radar.on
```

Discovering every entry-point model is project-specific. Many repos keep
top-level `.on` / `.one` files under a `model/` directory; loop over those, or
call out a fixed list in the workflow.

## Recommended: `model-test-report` Action

For richer CI output — especially on pull requests — use the
[`careweather/oneil/actions/model-test-report`](https://github.com/careweather/oneil/tree/main/actions/model-test-report)
Action. It installs a pinned Oneil ref, runs `oneil test --format json` on
discovered models, writes a Markdown report to the job summary, and can
**diff head vs. base** so the report highlights regressions and fixes rather
than only a raw pass/fail count.

Pin the Action ref and `oneil-ref` to the **same** Oneil version (for example
both `v1.0.0`).

### Single checkout

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

The Action builds Oneil from `oneil-ref` (so the calling workflow must install
Rust first). It does not install Python itself — add `setup-python` only when
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

For the full input/output reference, see the
[Action README](https://github.com/careweather/oneil/blob/main/actions/model-test-report/README.md).

## Tips

- **Pin versions.** Treat Oneil like a compiler: bump `oneil-ref` (and the
  Action tag) deliberately when you adopt a new release.
- **Keep tests close to requirements.** CI is most useful when `test:` lines
  encode margins and constraints you care about — see [Tests](./06-tests.md).
- **Python models.** If imports need packages, install them in the workflow
  before the Action (or before `oneil test`).
- **Other CI systems.** Install a release binary (or build from source), then
  run `oneil test --recursive …` and rely on the exit code; use
  `--format json` if you want to parse results yourself.
