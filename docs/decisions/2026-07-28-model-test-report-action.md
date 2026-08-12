# `model-test-report` GitHub Action

## Status

Accepted

## Context

[`2026-07-28-structured-test-output-for-ci.md`](./2026-07-28-structured-test-output-for-ci.md)
gave `oneil test` a scriptable exit code and a `--format json` output, so
that shared CI tooling for downstream model repos (e.g. `careweather/veery`)
wouldn't need to regex-parse text output. That decision explicitly deferred
the tooling itself. Two options were considered:

1. Improve `oneil test` further so no wrapper is needed at all.
2. Provide a GitHub Action, in a statically-typed language, that consumes
   `--format json` to add the one thing `oneil test` itself can't reasonably
   do: compare a PR's head against its base and report regressions vs.
   fixes, rather than a raw pass/fail count for a single checkout.

(1) alone doesn't solve the base/head diffing problem — that's inherently a
CI-orchestration concern, not something a single `oneil test` invocation
can know about. So this ADR is about (2).

## Decision

**A JavaScript/TypeScript GitHub Action lives at `actions/model-test-report/`
in this repo**, consumed as `careweather/oneil/actions/model-test-report@<ref>`.

- **In-repo, not a separate repo.** It's versioned and tagged alongside the
  `oneil test --format json` schema it depends on, so a given action ref and
  a given `oneil-ref` input are inherently compatible — no separate release
  process to keep in sync.
- **Standard JS-action packaging.** TypeScript source in `src/`, bundled by
  `esbuild` into a committed `dist/index.cjs` (`runs: using: node24, main:
  dist/index.cjs`) — the same approach used by `actions/checkout` and most
  marketplace actions, and already familiar from `vscode/esbuild.mjs`. `.cjs`
  (not `.js`) because the package is `"type": "module"` for the TypeScript
  source's benefit, but the bundle itself is CommonJS.
- **Runtime-validated JSON parsing**, not a type cast. `src/schema.ts` uses
  `zod` to parse and validate `oneil test --format json`'s output. This
  crossed the whole point of doing this in TypeScript: a `JSON.parse(...) as
  TestReport` would compile fine even if `oneil`'s schema changed underneath
  it, silently reintroducing exactly the fragility this was meant to fix. A
  schema mismatch now throws a clear error pointing at the actual field that
  changed.
- **Toolchain setup and checkouts are the caller's job, not the action's.**
  The action only installs a pinned `oneil` ref and runs its `test`
  subcommand; it assumes `cargo`/`python3` are already on `PATH` and that
  `head-dir`/`base-dir` are already checked out. This keeps the action
  focused on the one bespoke, worth-testing piece of logic (installing
  `oneil`, running tests, diffing, reporting) instead of also reimplementing
  `dtolnay/rust-toolchain` / `actions/setup-python` / `actions/checkout`,
  which already do their jobs well. See `actions/model-test-report/README.md`
  for full usage examples, including the base/head checkout pattern.
- **Comparison, not just reporting, is the action's core value.** `compare.ts`
  diffs two `TestReport`s by `(model, expression)` key into `regressed`,
  `fixed`, `newFailing`, `newPassing`, `removed`, and `stillFailing`
  categories. Only `regressed`, `newFailing`, and new diagnostics fail the
  action by default (`fail-on-problems`) — a pre-existing failure that
  hasn't changed shouldn't block an unrelated PR, but it's still rendered
  (`stillFailing`) so it isn't silently invisible.
- **`base-dir` is optional.** Without it, the action just reports the head
  checkout's own results (still through the same `stillFailing`/pass-count
  reporting path) — simple push/schedule triggers don't need to fabricate a
  "base" to get a useful report.

## Consequences

- Downstream repos get a single `uses:` line instead of a vendored script,
  with the diffing logic now type-checked, lint-checked, and unit-tested
  (`npm test` covers `compare.ts`, `report.ts`, `schema.ts`, and
  `process.ts`) rather than living in a text-parsing script no one owns.
- `dist/index.cjs` is a build artifact that must be kept in sync with `src/`;
  `.github/workflows/model-test-report-action.yml` enforces this with `npm
  run check-dist` (rebuild and `git diff --exit-code`) on every change under
  `actions/model-test-report/`.
- The action's own toolchain (Node, TypeScript, esbuild, zod, vitest) is
  independent of the Rust workspace's toolchain — it has its own
  `package.json`/`package-lock.json` under `actions/model-test-report/` and
  isn't part of the Cargo workspace.
- Downstream repos still need a few lines of workflow YAML for checkout and
  toolchain setup (see the README). A reusable *workflow* wrapping all of
  that was considered and rejected for now: it would hide `actions/checkout`
  behind an opinionated ref-resolution policy that different repos are
  likely to want to customize (e.g. "always diff against `main`" vs. "diff
  against the PR base"), whereas the action alone stays composable. This can
  be revisited if downstream repos converge on one policy.
