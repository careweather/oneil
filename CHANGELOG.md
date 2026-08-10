# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/2.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **ci**: Added `actions/model-test-report`, a TypeScript GitHub Action for
  downstream model repos: installs a pinned `oneil` ref, runs `oneil test
  --format json`, and — when given both a head and base checkout — reports
  regressions and fixes rather than just a pass/fail count. See
  `actions/model-test-report/README.md`.
- **ci**: TypeScript bindings for JSON wire formats (`TestReport`,
  `RenderedTree`, shared leaves) are generated with `ts-rs` into
  `packages/ts-interfaces` (`oneil-ts-interfaces`), with a CI drift check
  (`./scripts/check-ts-interfaces.sh`). See `docs/CODING_STANDARDS.md`.
- **cli**: `oneil test --format json` prints a machine-readable JSON report
  (diagnostics plus per-test pass/fail results, with dependency values for
  failures).

### Changed

- **cli**: `oneil test` now exits with status 1 if there were any error
  diagnostics or any test failed

## [0.16.1] - 2026-07-03

### Added

- **docs**: Added a changelog

### Fixed

- **builtins**: `min` and `max` functions match behavior in Python Oneil
- **cache**: infinity, negative infinity, and NaN encode as special JSON values
  rather than `null`
- **python**: importing a module doesn't fail if `__file__` is `None`
- **parser**: Whitespace at the beginning of a file doesn't cause parsing to
  fail
- **resolution**: If a reference has errors, don't show an "undefined
  reference" error

## [0.16.0] - 2026-06-26

0.16.0 is the initial release of the Rust rewrite to the public. It has feature
parity with the Python version (0.15.0) aside from the missing REPL. There are
also some changes to the syntax and semantics.

### Added

- Unit casting (`(<expr>:<unit>)`) can be used to assign units to literals
  - Example: `(1:kg)`
- LSP support and VS Code Extension

### Changed

- `|` is now an expression operator and can be nested in expressions (ex.
  `(0 | 100) + 273.15`)
- **Breaking:** Strings may now only use single quotes (`'`)
- **Breaking:** Notes are no longer defined by indentation. Instead, use
  `~ my note` for single-line notes and `~~~` to surround multi-line notes.
- **Breaking:** Certain binary operators require the same units on both sides,
  but in the new Oneil, literal numbers are considered to be unitless. This
  means for example that if `x` is in `km`, then `0 | x` or `x > 100` will
  produce a unit mismatch error. Instead, you will need to use unit casting,
  like `(0:km) | x` or `x > (100:km)`.
- **Breaking:** Python API has been completely revamped. See the docs for
  details.
- **Breaking:** Interval arithmetic has been updated to handle more edge cases

### Removed

- **Breaking:** Parameters that use a "pointer" (`=>`) are now obsolete and can
  be replaced with regular parameters.

[Unreleased]: https://github.com/careweather/oneil/compare/v0.16.1...HEAD
[0.16.0]: https://github.com/careweather/oneil/releases/tag/v0.16.0
