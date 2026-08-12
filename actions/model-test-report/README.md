# `oneil/model-test-report`

Run Oneil model tests in CI for a model repository, and get a readable report
of what failed.

On a single checkout it tells you which models/tests failed (and fails the
job when anything is wrong). On a pull request, point it at both the PR head
and the base branch and it highlights **regressions** and **fixes** — tests
that newly fail or newly pass — instead of only a raw pass/fail count.

The Markdown report is written to the job summary (and optionally to a file /
step output) so you can post it as a PR comment or upload it as an artifact.

## Usage

### Simple: just run the current checkout's tests

```yaml
name: Oneil model tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          repository: careweather/oneil
          path: oneil-src
      - uses: dtolnay/rust-toolchain@stable
      # Only needed if models import Python functions:
      - uses: actions/setup-python@v5
        with:
          python-version: "3.12"
      - uses: actions/checkout@v4
      - uses: careweather/oneil/actions/model-test-report@proj/rust-rewrite
        with:
          oneil-ref: proj/rust-rewrite
          model-dir: model
```

### Comparing a PR's head against its base

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
      # Only needed if models import Python functions:
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

      - uses: careweather/oneil/actions/model-test-report@proj/rust-rewrite
        id: report
        with:
          oneil-ref: proj/rust-rewrite
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

The report is always written to the job's step summary. `report-path` and the
`report` output let you do more with it — post it as a PR comment, upload it
as an artifact, etc. — in a later step.

## Inputs

| Input              | Required | Default | Description                                                                                       |
| ------------------ | -------- | ------- | --------------------------------------------------------------------------------------------------- |
| `oneil-ref`         | yes      |         | Git ref (tag/branch/sha) of `careweather/oneil` to install and test with.                          |
| `head-dir`          | no       | `.`     | Directory containing the head checkout to test.                                                    |
| `base-dir`          | no       |         | Directory containing the base checkout, for regression/fix comparison. Omit to test only `head-dir`. |
| `model-dir`         | no       | `model` | Path (relative to each checkout's root) containing the `.on` / `.one` source files.                |
| `models`            | no       |         | Comma-separated list of specific `.on` / `.one` filenames (relative to `model-dir`). Overrides auto-discovery. |
| `skip-models`       | no       |         | Comma-separated list of `.on` / `.one` filenames to exclude from auto-discovery. Ignored if `models` is set. |
| `timeout-seconds`   | no       | `120`   | Per-model `oneil test` timeout, in seconds.                                                         |
| `report-path`       | no       |         | If set, also write the Markdown report to this file.                                               |
| `fail-on-problems`  | no       | `true`  | Fail the action when there are problems. Set `false` to only warn.                                 |
| `head-label`        | no       | `head`  | Label for the head checkout in the report.                                                         |
| `base-label`        | no       | `base`  | Label for the base checkout in the report (only used when `base-dir` is set).                      |

## Outputs

| Output        | Description                                                                                     |
| ------------- | ------------------------------------------------------------------------------------------------- |
| `has-problems` | `"true"` if there are regressions, new failures, new diagnostics, or (with no `base-dir`) any head failure. |
| `report`      | The rendered Markdown report.                                                                     |
| `report-path` | Echoes the `report-path` input, for convenience in later steps.                                   |

Auto-discovery (no `models` input) only considers top-level `.on` and `.one`
files directly in `model-dir` that declare at least one `test:` block —
`oneil test --recursive` already covers submodel tests reached from an
entry-point model, so there's no need to separately discover files that are
only ever imported as submodels. Design files (`.one`) with their own tests
are included.

## Requirements

- The calling workflow must install Rust (`dtolnay/rust-toolchain`) before
  this Action. The Action itself only installs a pinned `oneil` and runs
  `oneil test` — it does not use Python.
- Install Python (`actions/setup-python`) only if the models under test
  import Python functions. Models with no Python imports do not need it.
- Pin `oneil-ref` and the Action ref to the **same** Oneil version/branch.
  When the test-report contract changes incompatibly, bump both together.

## Development

```sh
npm install
npm run typecheck
npm run lint
npm test
npm run build   # writes dist/index.cjs, which must be committed
```

`dist/index.cjs` is a committed build artifact (the standard packaging for a
`runs: using: node24` action) — run `npm run check-dist` (or the CI workflow)
to verify it's up to date with `src/`.

### Regenerating TypeScript bindings

From the repository root (requires a Rust toolchain):

```sh
./scripts/generate-ts-interfaces.sh   # rewrite packages/ts-interfaces/
./scripts/check-ts-interfaces.sh      # regenerate + git diff --exit-code
```

Commit any intentional changes under `packages/ts-interfaces/` together with
the Rust DTO updates that caused them.
