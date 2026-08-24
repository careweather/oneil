I'll read the crate, its tests, and the coding-standards testing guidance so the review can be specific and grounded.### Summary

`oneil_py_call_cache` is a small serialization crate (`FileCache`, `ImportHash`, `FunctionCall` / `FunctionCallResult`, I/O errors) with **no tests in the crate**. Downstream `oneil_runtime` tests exercise cache *policy* using `FileCache` as a fixture; they do not lock the JSON contract, hex hashing, untagged result decoding, or `read_from_path` / `write_to_path`. That is the main testing gap relative to `docs/CODING_STANDARDS.md` (unit tests for one function / one input / one output; snapshots for readable output; properties for invariants). Coverage of the *public* API is effectively zero; the type is simple enough that a handful of unit tests plus one snapshot of a cache document would cover the expected use cases without chasing 100% branch coverage.

### Findings

- **No crate-local tests.** There is no `#[cfg(test)]`, `tests/`, insta usage, or fuzz target. `cargo test -p oneil_py_call_cache` would compile the library and run nothing. Coding standards ask for unit tests of main functionality first; this crate has none.

- **Downstream tests are not a substitute.** `python_call_cache_tests` in `oneil_runtime/src/cache.rs` builds `FileCache` in memory (`file_cache` helper) and asserts `PythonCallCache::get` / `insert` / warnings. Only `insert_invalid_cache_json_emits_warning` touches disk JSON, and it writes `{not valid json` to exercise *runtime* recovery, not `FileCache::read_from_path` success/error mapping.

- **Highest-value missing unit tests (prepare / run / assert).** These are single-function, one-input/one-output cases the standards describe:
  - `ImportHash` serialize always emits 16 lowercase hex digits; deserialize accepts shorter hex, rejects empty (`"empty hexadecimal string"`), rejects non-hex, rejects overflow; `Display` matches serialize.
  - `FileCache` JSON uses `{"version":"v1", ...}`; unknown `version` fails; getters/`function_calls_mut` round-trip the maps they wrap.
  - `write_to_path` / `read_from_path` round-trip on a temp path; missing file → `ReadCacheError::Io`; invalid JSON → `ReadCacheError::Serde`.
  - `FunctionCallResult`: `From`/`Into` `Result<Value, PythonEvalError>` both directions; **untagged** success vs `PythonEvalError` (`{"error":...}`) must not collide (this is the fragile serde choice).
  - `WriteCacheError` / `ReadCacheError` `Display` and `Error::source`.

- **Snapshots belong on the JSON document, not `Debug`.** Standards put insta tests in `oneil_snapshot_tests` and forbid default `Debug` dumps. A pretty-printed `FileCache` with one success and one failure call would make the on-disk format reviewable. That crate currently has no cache snapshots.

- **Property tests are unused but a good fit for a small invariant.** Workspace fuzz lives under `oneil_output`, not here. A round-trip property (`ImportHash` ↔ hex string, or `FileCache` ↔ JSON) would match the standards’ “many semi-random inputs” guidance without needing full fuzz infrastructure if you start with a few table-driven unit cases.

- **No reusable test helpers in this crate.** Runtime’s `file_cache` / `python_path` helpers are fine for *runtime* tests; they skip hash padding, serde, I/O, and failure rows. A crate-local constructor (module path + hash + one call) would keep unit tests short if tests are added here.

- **Test design vs standards.** There is nothing to nit on AAA, `let … else`, or over-asserting—because there are no tests. Runtime tests mix several assertions per case (prompt kind, diagnostics, paths); that is reasonable for *policy*, not a model for *this* crate’s functions.

- **Simplicity / conciseness.** The library is small and mostly serde + two path helpers. Tests should stay equally small: no Python, no `PythonCallCache`. Avoid duplicating runtime prompt/policy tests here.

### Notes

- `docs/CODING_STANDARDS.md` explicitly says coverage need not be 100%; several unit tests of the main path (JSON load/save, hash hex, result variants) would match that bar. I/O edge cases (permission denied, parent `create_dir_all`) can wait for a bug-driven test.

- README still describes `CacheValue` and a parameter/test-bucket layout that the types do not implement. Tests (especially a snapshot) would have made that drift obvious.

- `FileCache::new` docs mention panicking if `CARGO_PKG_VERSION` is invalid; the body does not use that. Stale docs, not a test gap, but they would confuse anyone writing tests from comments.

- `indexmap` is a crate dependency unused by this crate; irrelevant to coverage except that extra deps are not exercised here.
