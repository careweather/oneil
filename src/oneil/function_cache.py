import ast
import builtins
import hashlib
import importlib
import inspect
import json
import math
import os
import pickle
import sys

import numpy as np


CACHE_DIR = "__oncache__"
CACHE_VERSION = 5


def _propagate_current_virtualenv():
    """Expose the current Python virtualenv to imported user code and subprocesses."""
    in_venv = getattr(sys, "base_prefix", sys.prefix) != sys.prefix
    executable_dir = os.path.dirname(os.path.abspath(sys.executable))
    venv_dir = os.path.dirname(executable_dir)

    if not in_venv:
        return

    os.environ["VIRTUAL_ENV"] = venv_dir

    current_path = os.environ.get("PATH", "")
    path_parts = current_path.split(os.pathsep) if current_path else []
    if executable_dir not in path_parts:
        os.environ["PATH"] = os.pathsep.join([executable_dir] + path_parts) if path_parts else executable_dir


_propagate_current_virtualenv()


class FunctionCache:
    """
    Persistent file-based cache for Python breakout function results.

    Cache is stored in one JSON file per model under __oncache__/ and can be:
    - Version controlled (committed to git)
    - Shared with other users (works even without Python dependencies)
    - Inspected as human-readable JSON
    """

    def __init__(self, parameter_class_getter, cache_dir=None):
        self._parameter_class_getter = parameter_class_getter
        self._cache_dir = cache_dir
        self._cache_file = None
        self._model_filename = None
        self._index = {}
        self._index_dirty = False
        self._memory_cache = {}
        self._import_hashes = {}
        self._function_modules = {}
        self._function_runtime_dependencies = {}
        self._verbose_mode = False
        self._active_run_keys = None
        self._run_stack = []
        self._used_cache_entries = set()
        self._used_cache_files = set()

    def _debug_enabled(self):
        """Return whether cache debug logging is enabled."""
        value = os.environ.get("ONEIL_CACHE_DEBUG", "")
        return value.lower() in {"1", "true", "yes", "on"}

    def _debug_log(self, message):
        """Emit cache debug output to stderr when enabled."""
        if self._debug_enabled():
            print(f"[oneil-cache] {message}", file=sys.stderr)

    def set_verbose(self, enabled):
        """Enable or disable verbose mode for Python function calls."""
        self._verbose_mode = enabled

    def is_verbose(self):
        """Check if verbose mode is enabled."""
        return self._verbose_mode

    def set_cache_file(self, model_filename):
        """Set the cache file based on the model's location and name."""
        model_path = os.path.abspath(model_filename)
        model_dir = os.path.dirname(model_path) or os.getcwd()
        model_name = os.path.splitext(os.path.basename(model_path))[0]
        self._cache_dir = os.path.join(model_dir, CACHE_DIR)
        self._cache_file = os.path.join(self._cache_dir, f"{model_name}.json")
        self._model_filename = os.path.basename(model_path)
        self._load_index()

    def _ensure_cache_dir(self):
        """Create the cache directory if it doesn't exist."""
        if not self._cache_dir or not self._cache_file:
            return False
        os.makedirs(self._cache_dir, exist_ok=True)
        return True

    def _clear_cache_dir(self):
        """Remove all cache files in the current cache directory."""
        if not self._cache_dir or not os.path.exists(self._cache_dir):
            return

        for entry in os.listdir(self._cache_dir):
            path = os.path.join(self._cache_dir, entry)
            try:
                if os.path.isdir(path):
                    for root, dirs, files in os.walk(path, topdown=False):
                        for name in files:
                            os.remove(os.path.join(root, name))
                        for name in dirs:
                            os.rmdir(os.path.join(root, name))
                    os.rmdir(path)
                else:
                    os.remove(path)
            except OSError:
                pass

    def _remove_legacy_cache_layout(self):
        """Remove cache artifacts from older cache layouts."""
        if not self._cache_dir:
            return

        legacy_index = os.path.join(self._cache_dir, "index.json")
        legacy_data_dir = os.path.join(self._cache_dir, "data")

        if os.path.exists(legacy_index):
            try:
                os.remove(legacy_index)
            except OSError:
                pass

        if os.path.exists(legacy_data_dir):
            for root, dirs, files in os.walk(legacy_data_dir, topdown=False):
                for name in files:
                    try:
                        os.remove(os.path.join(root, name))
                    except OSError:
                        pass
                for name in dirs:
                    try:
                        os.rmdir(os.path.join(root, name))
                    except OSError:
                        pass
            try:
                os.rmdir(legacy_data_dir)
            except OSError:
                pass

    def _serialize_parameter_value(self, value):
        """Convert a parameter min/max value into JSON."""
        if value is None or isinstance(value, (bool, str, int)):
            return value
        if isinstance(value, np.bool_):
            return bool(value)
        if isinstance(value, np.integer):
            return int(value)
        if isinstance(value, (float, np.floating)):
            value = float(value)
            if math.isfinite(value):
                return value
            return {"__type__": "float", "value": repr(value)}
        raise TypeError(f"Unsupported parameter value type: {type(value)}")

    def _deserialize_parameter_value(self, value):
        """Reconstruct a parameter min/max value from JSON."""
        if isinstance(value, dict) and value.get("__type__") == "float":
            return float(value["value"])
        return value

    def _is_cacheable_parameter(self, value):
        """Check whether a value looks like a Oneil Parameter result."""
        return hasattr(value, "min") and hasattr(value, "max") and hasattr(value, "units")

    def _serialize_parameter_snapshot(self, parameter):
        """Serialize only the cache-relevant parts of a Parameter."""
        if not self._is_cacheable_parameter(parameter):
            raise TypeError(f"Cached simulations must use Parameter inputs/outputs, got {type(parameter)}")
        return {
            "min": self._serialize_parameter_value(parameter.min),
            "max": self._serialize_parameter_value(parameter.max),
            "units": dict(sorted(parameter.units.items())),
        }

    def _deserialize_parameter_snapshot(self, snapshot):
        """Rebuild a minimal Parameter object from cached min/max/units."""
        parameter_class = self._parameter_class_getter()
        parameter = parameter_class.__new__(parameter_class)
        parameter.id = "cached_result"
        parameter.name = "cached_result"
        parameter.line_no = None
        parameter.line = None
        parameter.model = None
        parameter.performance = False
        parameter.independent = True
        parameter.trace = False
        parameter.callable = False
        parameter.isdiscrete = isinstance(snapshot.get("min"), str) or isinstance(snapshot.get("max"), str)
        parameter.min = self._deserialize_parameter_value(snapshot.get("min"))
        parameter.max = self._deserialize_parameter_value(snapshot.get("max"))
        parameter.equation = None
        parameter.args = []
        parameter.section = ""
        parameter.pointer = False
        parameter.piecewise = False
        parameter.minmax_equation = False
        parameter.hr_units = ""
        parameter.used_fallback = False
        parameter.fallback_param = None
        parameter.notes = []
        parameter.note_lines = []
        parameter.options = None
        parameter.units = dict(snapshot.get("units", {}))
        return parameter

    def _build_cache_entry(
        self,
        simulation_id,
        func,
        module_name,
        source_file,
        source_hash,
        dependency_files,
        inputs_hash,
        inputs,
        result,
    ):
        """Create a cache entry for one simulation result."""
        return {
            "simulation_id": simulation_id,
            "function": getattr(func, "__name__", str(func)),
            "module": module_name,
            "simulation_file": os.path.basename(source_file) if source_file else None,
            "source_hash": source_hash,
            "dependency_files": dependency_files,
            "inputs_hash": inputs_hash,
            "inputs_repr": self._inputs_repr(inputs),
            "inputs": [self._serialize_parameter_snapshot(parameter) for parameter in inputs],
            "output": self._serialize_parameter_snapshot(result),
        }

    def _load_cached_result(self, entry):
        """Load a cached simulation output parameter from an entry."""
        return self._deserialize_parameter_snapshot(entry["output"])

    def _serialized_cache(self):
        """Return a deterministic JSON representation of the current model cache."""
        return json.dumps(
            {
                "entries": self._index,
                "model_file": self._model_filename,
                "version": CACHE_VERSION,
            },
            indent=2,
            sort_keys=True,
        ) + "\n"

    def _flush_if_needed(self):
        """Persist the cache immediately if not inside a model run."""
        if self._active_run_keys is None:
            self._save_index()

    def reset_usage_summary(self):
        """Reset cache usage tracking for one model load."""
        self._used_cache_entries.clear()
        self._used_cache_files.clear()

    def _record_cache_use(self, cache_key):
        """Record that a cached simulation result was used."""
        if self._cache_file:
            self._used_cache_entries.add((self._cache_file, cache_key))
            self._used_cache_files.add(self._cache_file)

    def usage_summary(self):
        """Return cache usage details for the current model load."""
        cache_models = []
        for cache_file in sorted(self._used_cache_files):
            cache_name = os.path.basename(cache_file)
            if cache_name.endswith(".json"):
                cache_name = cache_name[:-5] + ".on"
            cache_models.append(cache_name)
        return {
            "used_entries": len(self._used_cache_entries),
            "cache_models": cache_models,
        }

    def begin_run(self):
        """Track which cache entries were used while building the current model."""
        self._run_stack.append(self._active_run_keys)
        self._active_run_keys = set()

    def end_run(self, success=True):
        """Finalize cache updates after a model run."""
        if self._active_run_keys is None:
            return

        active_run_keys = self._active_run_keys
        previous_run_keys = self._run_stack.pop() if self._run_stack else None
        self._active_run_keys = None

        if success:
            stale_keys = [key for key in self._index if key not in active_run_keys]
            for key in stale_keys:
                self._remove_entry(key)

        self._save_index()
        self._active_run_keys = previous_run_keys

    def _load_index(self):
        """Load the current model cache from disk."""
        if not self._cache_dir or not self._cache_file:
            return

        self._remove_legacy_cache_layout()
        self._index = {}
        self._index_dirty = False

        if os.path.exists(self._cache_file):
            try:
                with open(self._cache_file, "r") as f:
                    data = json.load(f)
                if data.get("version") == CACHE_VERSION:
                    self._index = data.get("entries", {})
                else:
                    try:
                        os.remove(self._cache_file)
                    except OSError:
                        pass
                    self._index = {}
            except (json.JSONDecodeError, IOError):
                self._index = {}

    def _save_index(self):
        """Save the current model cache to disk."""
        if not self._cache_dir or not self._cache_file or not self._index_dirty:
            return
        if not self._ensure_cache_dir():
            return

        serialized = self._serialized_cache()
        if os.path.exists(self._cache_file):
            try:
                with open(self._cache_file, "r") as f:
                    if f.read() == serialized:
                        self._index_dirty = False
                        return
            except IOError:
                pass

        try:
            with open(self._cache_file, "w") as f:
                f.write(serialized)
            self._index_dirty = False
        except IOError:
            pass

    def _compute_file_hash(self, filepath):
        """Compute SHA256 hash of a file's contents."""
        try:
            with open(filepath, "rb") as f:
                return hashlib.sha256(f.read()).hexdigest()
        except (IOError, OSError):
            return None

    def _resolve_local_module_file(self, current_file, root_dir, module_name, level=0):
        """Resolve a local Python module to a file within the model workspace."""
        if not module_name and level == 0:
            return None

        current_dir = os.path.dirname(current_file)
        if level > 0:
            base_dir = current_dir
            for _ in range(level - 1):
                base_dir = os.path.dirname(base_dir)
        else:
            base_dir = root_dir

        module_parts = [part for part in module_name.split(".") if part] if module_name else []
        candidate_base = os.path.join(base_dir, *module_parts) if module_parts else base_dir
        candidates = [candidate_base + ".py", os.path.join(candidate_base, "__init__.py")]

        for candidate in candidates:
            candidate = os.path.abspath(candidate)
            if candidate.startswith(os.path.abspath(root_dir) + os.sep) or candidate == os.path.abspath(root_dir):
                if os.path.exists(candidate):
                    return candidate

        return None

    def _collect_local_dependency_files(self, filepath, root_dir, visited=None):
        """Collect transitive local Python dependencies under the model workspace."""
        filepath = os.path.abspath(filepath)
        root_dir = os.path.abspath(root_dir)
        if visited is None:
            visited = set()
        if filepath in visited:
            return set()

        visited.add(filepath)
        dependencies = {filepath}

        try:
            with open(filepath, "r") as f:
                tree = ast.parse(f.read(), filename=filepath)
        except (OSError, SyntaxError, UnicodeDecodeError):
            return dependencies

        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                for alias in node.names:
                    dependency = self._resolve_local_module_file(filepath, root_dir, alias.name)
                    if dependency:
                        dependencies.update(self._collect_local_dependency_files(dependency, root_dir, visited))
            elif isinstance(node, ast.ImportFrom):
                base_dependency = self._resolve_local_module_file(filepath, root_dir, node.module or "", level=node.level)
                if base_dependency:
                    dependencies.update(self._collect_local_dependency_files(base_dependency, root_dir, visited))
                    package_dir = os.path.dirname(base_dependency) if os.path.basename(base_dependency) == "__init__.py" else None
                else:
                    package_dir = None

                if package_dir:
                    for alias in node.names:
                        submodule_candidate = self._resolve_local_module_file(
                            os.path.join(package_dir, "__init__.py"),
                            root_dir,
                            alias.name,
                        )
                        if submodule_candidate:
                            dependencies.update(self._collect_local_dependency_files(submodule_candidate, root_dir, visited))

        return dependencies

    def _compute_dependency_hash(self, dependency_files):
        """Compute a stable hash of a set of dependency files."""
        file_hashes = []
        for filepath in sorted(dependency_files):
            file_hash = self._compute_file_hash(filepath)
            if file_hash is None:
                return None
            file_hashes.append((filepath, file_hash))

        serialized = json.dumps(file_hashes, sort_keys=True).encode()
        return hashlib.sha256(serialized).hexdigest()

    def _model_root_dir(self):
        """Return the directory containing the active model and cache."""
        if self._cache_dir:
            return os.path.abspath(os.path.dirname(self._cache_dir))
        if self._cache_file:
            return os.path.abspath(os.path.dirname(os.path.dirname(self._cache_file)))
        return os.path.abspath(os.getcwd())

    def _is_local_dependency_file(self, filepath, root_dir):
        """Check whether a file is a local Python dependency under the model root."""
        if not filepath:
            return False

        abs_path = os.path.abspath(filepath)
        abs_root = os.path.abspath(root_dir)
        return abs_path.endswith(".py") and (
            abs_path == abs_root or abs_path.startswith(abs_root + os.sep)
        )

    def _resolve_dependency_paths(self, dependency_files, root_dir):
        """Resolve stored dependency paths back to absolute files."""
        resolved = set()
        for dependency in dependency_files or []:
            abs_path = os.path.abspath(os.path.join(root_dir, dependency))
            if not self._is_local_dependency_file(abs_path, root_dir):
                return None
            resolved.add(abs_path)
        return resolved

    def _relative_dependency_paths(self, dependency_files, root_dir):
        """Convert absolute dependency files to model-relative paths."""
        abs_root = os.path.abspath(root_dir)
        return [
            os.path.relpath(os.path.abspath(dependency), abs_root)
            for dependency in sorted(dependency_files)
        ]

    def _record_runtime_module_dependency(self, module, root_dir, dependency_files):
        """Record a dynamically imported local module file."""
        filepath = getattr(module, "__file__", None)
        if self._is_local_dependency_file(filepath, root_dir):
            dependency_files.add(os.path.abspath(filepath))

    def begin_function_execution(self, func):
        """Trace local Python modules imported while a simulation function runs."""
        func_id = self._get_function_id(func)
        module_name = self._function_modules.get(func_id)
        root_dir = self._model_root_dir()
        dependency_files = set()

        if module_name in self._import_hashes:
            source_file, _, dependency_paths, tracked_root_dir = self._import_hashes[module_name]
            root_dir = tracked_root_dir
            resolved = self._resolve_dependency_paths(dependency_paths, root_dir)
            if resolved is not None:
                dependency_files.update(resolved)
            elif source_file:
                dependency_files.add(os.path.abspath(source_file))

        original_import = builtins.__import__
        original_import_module = importlib.import_module
        original_reload = importlib.reload

        def traced_import(name, globals=None, locals=None, fromlist=(), level=0):
            module = original_import(name, globals, locals, fromlist, level)
            self._record_runtime_module_dependency(module, root_dir, dependency_files)
            if fromlist:
                parent_name = getattr(module, "__name__", "")
                for imported_name in fromlist:
                    if not isinstance(imported_name, str) or imported_name == "*":
                        continue
                    qualified_name = f"{parent_name}.{imported_name}" if parent_name else imported_name
                    imported_module = sys.modules.get(qualified_name)
                    if imported_module is not None:
                        self._record_runtime_module_dependency(imported_module, root_dir, dependency_files)
            return module

        def traced_import_module(name, package=None):
            module = original_import_module(name, package)
            self._record_runtime_module_dependency(module, root_dir, dependency_files)
            return module

        def traced_reload(module):
            reloaded_module = original_reload(module)
            self._record_runtime_module_dependency(reloaded_module, root_dir, dependency_files)
            return reloaded_module

        builtins.__import__ = traced_import
        importlib.import_module = traced_import_module
        importlib.reload = traced_reload

        return {
            "func_id": func_id,
            "dependency_files": dependency_files,
            "original_import": original_import,
            "original_import_module": original_import_module,
            "original_reload": original_reload,
        }

    def end_function_execution(self, trace_token, success):
        """Stop tracing imports for a simulation function execution."""
        builtins.__import__ = trace_token["original_import"]
        importlib.import_module = trace_token["original_import_module"]
        importlib.reload = trace_token["original_reload"]

        func_id = trace_token["func_id"]
        if success and trace_token["dependency_files"]:
            self._function_runtime_dependencies[func_id] = set(trace_token["dependency_files"])
        else:
            self._function_runtime_dependencies.pop(func_id, None)

    def _compute_entry_dependency_hash(self, entry):
        """Compute the current dependency hash for a stored cache entry."""
        dependency_files = entry.get("dependency_files")
        if dependency_files is None:
            return None

        resolved = self._resolve_dependency_paths(dependency_files, self._model_root_dir())
        if not resolved:
            return None
        return self._compute_dependency_hash(resolved)

    def _normalize_hash_value(self, value):
        """Normalize values so equivalent numeric inputs hash identically."""
        if isinstance(value, np.bool_):
            return bool(value)
        if isinstance(value, np.integer):
            return int(value)
        if isinstance(value, np.floating):
            value = float(value)
            if math.isfinite(value):
                return value
            return ("float", repr(value))
        if isinstance(value, tuple):
            return tuple(self._normalize_hash_value(item) for item in value)
        if isinstance(value, list):
            return tuple(self._normalize_hash_value(item) for item in value)
        if isinstance(value, dict):
            return tuple(sorted((key, self._normalize_hash_value(val)) for key, val in value.items()))
        return value

    def _compute_inputs_hash(self, inputs):
        """Compute a hash of the input parameter values."""
        try:
            hashable_inputs = []
            for inp in inputs:
                if hasattr(inp, "min") and hasattr(inp, "max") and hasattr(inp, "units"):
                    hashable_inputs.append(
                        (
                            self._normalize_hash_value(inp.min),
                            self._normalize_hash_value(inp.max),
                            tuple(sorted((key, self._normalize_hash_value(val)) for key, val in inp.units.items())),
                        )
                    )
                elif isinstance(inp, np.ndarray):
                    hashable_inputs.append(("ndarray", inp.tobytes(), inp.shape, str(inp.dtype)))
                elif isinstance(inp, (list, tuple)):
                    hashable_inputs.append(self._normalize_hash_value(inp))
                elif isinstance(inp, dict):
                    hashable_inputs.append(self._normalize_hash_value(inp))
                else:
                    hashable_inputs.append(self._normalize_hash_value(inp))

            serialized = pickle.dumps(tuple(hashable_inputs), protocol=pickle.HIGHEST_PROTOCOL)
            return hashlib.sha256(serialized).hexdigest()[:16]
        except Exception:
            return None

    def _inputs_repr(self, inputs):
        """Create a human-readable representation of inputs for the cache file."""
        parts = []
        for inp in inputs:
            if hasattr(inp, "min") and hasattr(inp, "max") and hasattr(inp, "id"):
                if inp.min == inp.max:
                    parts.append(f"{inp.id}={inp.min}")
                else:
                    parts.append(f"{inp.id}={inp.min}|{inp.max}")
            elif isinstance(inp, np.ndarray):
                parts.append(f"array{inp.shape}")
            else:
                s = str(inp)
                if len(s) > 20:
                    s = s[:17] + "..."
                parts.append(s)
        return ", ".join(parts) if parts else "(no inputs)"

    def _get_function_id(self, func):
        """Get a unique identifier for a function."""
        module = getattr(func, "__module__", "unknown")
        name = getattr(func, "__qualname__", getattr(func, "__name__", str(func)))
        return f"{module}.{name}"

    def _get_cache_key(self, simulation_id):
        """Create a cache key within the current model cache file."""
        return simulation_id

    def register_import(self, module, filepath=None, root_dir=None):
        """Register an imported module and track local dependency hashes."""
        module_name = module.__name__

        if filepath is None:
            filepath = getattr(module, "__file__", None)
        if filepath is None:
            return True
        if root_dir is None:
            root_dir = os.getcwd()
        filepath = os.path.abspath(filepath)
        root_dir = os.path.abspath(root_dir)

        dependency_files = self._collect_local_dependency_files(filepath, root_dir)
        new_hash = self._compute_dependency_hash(dependency_files)
        dependency_paths = self._relative_dependency_paths(dependency_files, root_dir)

        if module_name in self._import_hashes:
            old_filepath, old_hash, old_dependency_paths, old_root_dir = self._import_hashes[module_name]
            if (
                old_hash == new_hash
                and old_filepath == filepath
                and old_dependency_paths == dependency_paths
                and old_root_dir == root_dir
            ):
                return False

        self._invalidate_module_cache(module_name, new_hash)
        self._import_hashes[module_name] = (filepath, new_hash, dependency_paths, root_dir)

        for name, obj in inspect.getmembers(module, inspect.isfunction):
            self._function_modules[f"{module_name}.{name}"] = module_name

        return True

    def _invalidate_module_cache(self, module_name, new_source_hash):
        """Invalidate cached results for a module if its source changed."""
        keys_to_remove = []
        for cache_key, entry in self._index.items():
            if entry.get("module") != module_name:
                continue

            dependency_files = entry.get("dependency_files")
            if dependency_files:
                current_hash = self._compute_entry_dependency_hash(entry)
                if current_hash is None or entry.get("source_hash") != current_hash:
                    keys_to_remove.append(cache_key)
                continue

            if entry.get("source_hash") != new_source_hash:
                keys_to_remove.append(cache_key)

        for key in keys_to_remove:
            self._remove_entry(key)

        if keys_to_remove:
            self._flush_if_needed()

    def get(self, simulation_id, func, inputs):
        """
        Try to get a cached result for the function with given inputs.
        Returns (True, result) if found, (False, None) if not.
        """
        if not self._cache_file:
            return False, None

        inputs_hash = self._compute_inputs_hash(inputs)
        if inputs_hash is None:
            return False, None

        cache_key = self._get_cache_key(simulation_id)
        if cache_key in self._memory_cache:
            cached_inputs_hash, cached_result = self._memory_cache[cache_key]
            if cached_inputs_hash == inputs_hash:
                self._debug_log(f"{simulation_id}: memory HIT")
                self._record_cache_use(cache_key)
                if self._active_run_keys is not None:
                    self._active_run_keys.add(cache_key)
                return True, cached_result
            self._debug_log(f"{simulation_id}: memory STALE inputs")
            del self._memory_cache[cache_key]

        if cache_key in self._index:
            entry = self._index[cache_key]
            if entry.get("inputs_hash") != inputs_hash:
                self._debug_log(f"{simulation_id}: disk STALE inputs")
                return False, None
            current_hash = self._compute_entry_dependency_hash(entry)
            if current_hash is None:
                module_name = entry.get("module")
                if module_name in self._import_hashes:
                    _, current_hash, _, _ = self._import_hashes[module_name]
            if current_hash is None or entry.get("source_hash") != current_hash:
                self._debug_log(f"{simulation_id}: disk STALE source")
                self._remove_entry(cache_key)
                self._flush_if_needed()
                return False, None

            try:
                result = self._load_cached_result(entry)
                self._debug_log(f"{simulation_id}: disk HIT")
                self._record_cache_use(cache_key)
                self._memory_cache[cache_key] = (inputs_hash, result)
                if self._active_run_keys is not None:
                    self._active_run_keys.add(cache_key)
                return True, result
            except (KeyError, TypeError, ValueError):
                self._debug_log(f"{simulation_id}: disk CORRUPT")
                self._remove_entry(cache_key)
                self._flush_if_needed()

        self._debug_log(f"{simulation_id}: MISS")
        return False, None

    def _remove_entry(self, cache_key):
        """Remove a cache entry."""
        if cache_key in self._index:
            del self._index[cache_key]
            self._index_dirty = True
        if cache_key in self._memory_cache:
            del self._memory_cache[cache_key]

    def set(self, simulation_id, func, inputs, result):
        """Store a result in the cache."""
        if not self._cache_file:
            return
        if not self._ensure_cache_dir():
            return

        func_id = self._get_function_id(func)
        inputs_hash = self._compute_inputs_hash(inputs)
        if inputs_hash is None:
            return

        cache_key = self._get_cache_key(simulation_id)
        module_name = self._function_modules.get(func_id)
        source_hash = None
        source_file = None
        dependency_files = None
        root_dir = None
        if module_name and module_name in self._import_hashes:
            source_file, source_hash, dependency_files, root_dir = self._import_hashes[module_name]

        runtime_dependency_files = self._function_runtime_dependencies.get(func_id)
        if runtime_dependency_files:
            if root_dir is None:
                root_dir = self._model_root_dir()
            combined_dependency_files = set(runtime_dependency_files)
            resolved_dependency_files = self._resolve_dependency_paths(dependency_files, root_dir)
            if resolved_dependency_files:
                combined_dependency_files.update(resolved_dependency_files)
            if source_file:
                combined_dependency_files.add(os.path.abspath(source_file))
            source_hash = self._compute_dependency_hash(combined_dependency_files)
            dependency_files = self._relative_dependency_paths(combined_dependency_files, root_dir)

        try:
            cache_entry = self._build_cache_entry(
                simulation_id,
                func,
                module_name,
                source_file,
                source_hash,
                dependency_files,
                inputs_hash,
                inputs,
                result,
            )
        except TypeError:
            return

        if self._index.get(cache_key) != cache_entry:
            self._index[cache_key] = cache_entry
            self._index_dirty = True

        self._memory_cache[cache_key] = (inputs_hash, result)
        if self._active_run_keys is not None:
            self._active_run_keys.add(cache_key)

        self._flush_if_needed()

    def clear(self):
        """Clear all cached results (both memory and disk)."""
        self._memory_cache.clear()
        self._function_runtime_dependencies.clear()
        if self._cache_dir and os.path.exists(self._cache_dir):
            self._clear_cache_dir()
        self._index.clear()
        self._index_dirty = False

    def clear_all(self):
        """Clear everything including import tracking."""
        self.clear()
        self._import_hashes.clear()
        self._function_modules.clear()

    def stats(self):
        """Return cache statistics."""
        cache_size = 0
        if self._cache_file and os.path.exists(self._cache_file):
            try:
                cache_size = os.path.getsize(self._cache_file)
            except OSError:
                cache_size = 0

        return {
            "disk_entries": len(self._index),
            "memory_entries": len(self._memory_cache),
            "tracked_imports": len(self._import_hashes),
            "cache_size_bytes": cache_size,
            "cache_dir": self._cache_dir,
            "cache_file": self._cache_file,
        }
