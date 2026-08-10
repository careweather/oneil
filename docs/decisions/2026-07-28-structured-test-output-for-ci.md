# Structured `oneil test` Output and Exit Codes for CI

## Status

Accepted

## Context

Repositories that define Oneil models (e.g. `careweather/veery`) want CI that
runs `oneil test` across their models and reports pass/fail, ideally
comparing a PR's head against its base to flag regressions. An earlier
attempt at solving this shared-tooling problem vendored a ~390-line Python
script that shelled out to `oneil test`, regex-parsed its human-readable text
output (`test: <expr>` / `  Result: PASS|FAIL` / `Tests: X/Y (...)`), and
diffed two runs. This has several problems:

1. **Fragile parsing.** Any change to `oneil test`'s text formatting (colors,
   wording, spacing) can silently break downstream regex parsing, with no
   compiler or type system to catch it.
2. **Wrong language for the job.** Python has no static types here — the
   parsing script's `TestStatus` was a `Literal["PASS", "FAIL", "ERROR"]`
   string with no enforcement outside a handful of call sites, in a codebase
   that otherwise commits to strong typing throughout.
3. **No exit code to build on.** `oneil test` always exited 0 regardless of
   test outcome, so any CI wrapper — Python, TypeScript, or shell — had to
   reimplement pass/fail detection from output rather than checking `$?`,
   unlike `oneil check`, which already exits 1 on diagnostics for exactly
   this reason.

Rather than immediately writing a replacement wrapper in a different
language, the first problem to fix is in `oneil` itself: the CLI should give
any caller (a CI script, a GitHub Action, a Makefile) a scriptable pass/fail
signal and structured data, without requiring output scraping at all.

## Decision

1. **Fix `oneil test`'s exit code.** It now exits 1 if there were any error
   diagnostics or any failing test, in both output formats — mirroring
   `handle_check_command`'s existing `#[expect(clippy::exit, ...)]`
   convention. This alone lets simple callers do
   `oneil test model.on && ...` without any wrapper.
2. **Add `oneil test --format json`.** A new `TestOutputFormat` (`text`
   default, `json`) on `TestArgs`. In JSON mode, `oneil test` prints one JSON
   object to stdout (see `oneil_cli::json_test_report` for the full schema)
   containing:
   - `success`: whether the run should be considered passing.
   - `diagnostics`: parse/resolution/eval diagnostics, structured (kind,
     path, message, line, column) rather than rendered text.
   - `models`: per-model test entries (expression text, source span,
     pass/fail, and — for failures only — the dependency values that were
     live when the test was evaluated, mirroring the text output's "FAILING
     TESTS" section).

   This schema is intentionally **separate** from `oneil_lsp`'s
   `RenderedTree`/`RenderedTest` JSON (used by the VS Code rendered view):
   that payload is an internal webview transport that can evolve freely,
   while `oneil test --format json`'s schema is now a CI-facing contract
   that downstream tooling depends on.

## Consequences

**Easier:**
- CI tooling (in any language) that needs per-test detail — e.g. to diff
  base vs. head results, as `veery`'s original script did — can call `oneil
  test --format json` twice and diff two JSON payloads by (model,
  expression) key, with no text parsing.
- Simple CI usage needs no tooling at all: `oneil test model.on` is now a
  valid pass/fail gate on its own.
- Sets up a stable foundation for a properly-typed CI integration (a
  TypeScript GitHub Action) to be built on top, instead of a Python wrapper
  parsing text output — see
  [`2026-07-28-model-test-report-action.md`](./2026-07-28-model-test-report-action.md)
  for that follow-up.

**Harder:**
- `oneil test`'s exit code is a behavior change: any existing caller that
  assumed `oneil test` always exits 0 (e.g. to keep running after a failing
  model) needs `|| true` or equivalent. This is called out in the
  changelog as a `Changed` entry.
- The JSON schema (`oneil_cli::json_test_report`) is now a contract:
  changes to its shape should be considered breaking for downstream
  consumers, similar to any other CI-facing tool output.
