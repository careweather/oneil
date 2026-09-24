# Extension-managed Oneil CLI downloads

## Status

Accepted

## Context

The VS Code / Cursor extension previously required users to install the Oneil CLI themselves (`PATH`, `ONEIL_PATH`, or `oneil.serverPath`). That is awkward for casual users and makes keeping the language server in sync with GitHub Releases harder. We already publish platform archives from the tag-triggered Release workflow (same assets used by `actions/install-oneil`).

## Decision

The extension may download a single managed CLI binary into its `globalStorage` directory from `careweather/oneil` GitHub Releases.

- **Precedence:** `oneil.serverPath` → managed binary (if present) → `ONEIL_PATH` → `oneil` on PATH.
- **One binary on disk:** install, update, and “select version” all overwrite the same managed path; there is no side-by-side version cache. Choosing an older release re-downloads that tag.
- **Active version:** always from running `oneil --version` on the resolved binary. Do not write an `active.json` (or similar) sidecar.
- **Bookkeeping:** only VS Code `globalState` (last update check time, skipped version, Python flavor of the managed binary).
- **Prompts:** offer install when no CLI is found; throttled daily check against GitHub’s latest release (skips prereleases) unless `oneil.serverPath` is set. “Skip this version” opts out of a particular release. Commands: Check for Updates, Install or Update, Select CLI Version.

Platform coverage matches the release matrix (Linux x86_64, Windows x86_64, Apple Silicon macOS).

## Consequences

- New users can get a working language server without a separate CLI install step.
- Developers can still force a local build via `oneil.serverPath`.
- The extension must handle GitHub rate limits and unsupported platforms gracefully.
- Managed updates do not apply while `oneil.serverPath` is set.
- The managed CLI links against one CPython minor version. Releases ship one archive per OS for Python 3.12 and one for 3.14; `oneil` finds the local install of that version. See [exec loader](./2026-09-23-cli-python-exec-loader.md).
