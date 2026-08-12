# `oneil/install-oneil`

Install the Oneil CLI from a [GitHub Release](https://github.com/careweather/oneil/releases)
onto `PATH` for the rest of the job.

Prefer this Action when your workflow needs `oneil` directly (custom scripts,
ad-hoc `oneil test`, etc.). For model-repo CI that should discover models and
produce a Markdown report, use
[`model-test-report`](../model-test-report/README.md) instead.

## Usage

```yaml
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

Pin the Action ref and `version` to the **same** release tag.

## Inputs

| Input | Required | Default | Description |
|-------|----------|---------|-------------|
| `version` | yes | | Release tag to install (e.g. `v1.0.0`) |
| `github-token` | no | `${{ github.token }}` | Token used by `gh release download` (rate limits) |

## Outputs

| Output | Description |
|--------|-------------|
| `version` | Text from `oneil --version` |
| `oneil-path` | Absolute path to the installed binary |

## Platforms

| Runner | Archive |
|--------|---------|
| `ubuntu-*` (x86_64) | `oneil-<tag>-x86_64-unknown-linux-gnu.tar.gz` |
| `windows-*` (x86_64) | `oneil-<tag>-x86_64-pc-windows-msvc.zip` |
| `macos-*` (Apple Silicon) | `oneil-<tag>-aarch64-apple-darwin.tar.gz` |
| `macos-*` (Intel) | `oneil-<tag>-x86_64-apple-darwin.tar.gz` |

These archives are produced by the Oneil
[Release](https://github.com/careweather/oneil/blob/main/.github/workflows/release.yml)
workflow when a `v*` tag is pushed.
