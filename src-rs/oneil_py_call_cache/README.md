# oneil_py_call_cache

Types and JSON serialization for caching Python function calls evaluated from Oneil source. The cache is scoped per file: imported modules (with invalidation metadata), calls made from named parameters, and calls made from tests.

Main pieces:

- **`FileCache`** — load and save a cache document with `read_from_path` / `write_to_path` (pretty-printed JSON).
- **`FunctionCall`** — one invocation: function name, serialized inputs, and success or failure (`FunctionCallResult`).
- **`CacheValue`** — values that can be stored in the cache, bridged to `oneil_output::Value` and `oneil_python::PythonEvalError`.
