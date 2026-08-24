# oneil_py_call_cache

Types and JSON serialization for caching Python function calls evaluated from Oneil source. The cache is scoped per Python module: an import hash and local dependencies for invalidation, plus cached calls keyed by function name.

Main pieces:

- **`FileCache`** — load and save a cache document with `read_from_path` / `write_to_path` (pretty-printed JSON).
- **`ImportHash`** — fingerprint of a Python module's sources (stored as `u64`, serialized as hex).
- **`FunctionCall`** — one invocation: root models, serialized inputs, and success or failure (`FunctionCallResult`).
