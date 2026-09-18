//! Loading Python code and extracting callables.

use std::{
    ffi::CString,
    iter,
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
};

use indexmap::{IndexMap, IndexSet};
use oneil_shared::{paths::PythonPath, symbols::PyFunctionName};
use pyo3::{
    prelude::*,
    types::{PyDict, PyList, PyTuple},
    wrap_pymodule,
};

use crate::{
    error::LoadPythonImportError,
    function::{PythonFunction, PythonModule},
    py_compat::oneil_python_module,
    source_hash::calculate_source_hash,
};

pub fn load_python_import(
    path: &PythonPath,
    source: &str,
) -> Result<PythonModule, LoadPythonImportError> {
    // get the module directory from the path
    let module_directory = path
        .as_path()
        .parent()
        // if the parent is an empty string, use the current directory
        .map(|p| if p == "" { Path::new(".") } else { p })
        .unwrap_or_else(|| Path::new("."))
        .canonicalize()
        .expect("path should be directory that exists");

    // convert the path and reserved module name to C strings
    let path_str = path.as_path().to_string_lossy();
    let path_cstr = CString::new(path_str.as_bytes()).expect("path should not have a null byte");
    let module_name_cstr = CString::new(IMPORTED_MODULE_NAME)
        .expect("reserved module name should not have a null byte");

    // convert the source to a C string
    let source_cstr = match CString::new(source) {
        Ok(cstr) => cstr,
        Err(_null_error) => return Err(LoadPythonImportError::SourceHasNullByte),
    };

    let _load_guard = lock_python_loads();
    let functions = Python::attach(|py| {
        insert_oneil_module_into_python(py)?;
        // Bind stdlib `inspect` before the user directory is on `sys.path`.
        let inspect_module = PyModule::import(py, "inspect")?;
        let _import_state = isolate_import_state(py, &module_directory)?;

        // load the code module
        start_tracking_imports(py, &module_directory)?;
        let code_module = PyModule::from_code(py, &source_cstr, &path_cstr, &module_name_cstr)?;
        let imports = stop_tracking_imports(py)?;

        // get the functions from the code module
        let functions = code_module
            .dict()
            .iter()
            .filter(|(key, value)| !key.to_string().starts_with("__") && value.is_callable())
            .map(|(key, value)| {
                let name = key.to_string();
                let docs = get_doc_string(&value, &inspect_module);
                let line_no = get_line_no(&value, &inspect_module);
                let value = value.unbind();
                Ok((
                    PyFunctionName::from(name),
                    PythonFunction::new(value, docs, line_no, module_directory.clone()),
                ))
            })
            .collect::<PyResult<IndexMap<_, _>>>()?;

        let module_docs = get_doc_string(&code_module, &inspect_module);

        Ok::<_, PyErr>((module_docs, functions, imports))
    });

    // return the functions
    match functions {
        Ok((module_docs, functions, mut imports)) => {
            // add the `requirements.txt` path to the imports if it exists
            let maybe_requirements_txt_path = maybe_requirements_txt_path(&module_directory);
            imports.extend(maybe_requirements_txt_path);

            // calculate the source hash
            let source_paths = iter::once(path.as_path())
                .chain(imports.iter().map(|path| path.as_path()))
                .collect::<Vec<_>>();
            let hash = calculate_source_hash(source_paths)?;

            // return the Python module
            Ok(PythonModule::new(module_docs, functions, imports, hash))
        }
        Err(e) => Err(LoadPythonImportError::CouldNotLoadPythonModule(e)),
    }
}

/// Python `__name__` for Oneil-loaded source.
///
/// A reserved identifier is used so a user file such as `inspect.py` does not
/// replace the standard library module this loader needs for `getdoc`.
const IMPORTED_MODULE_NAME: &str = "_oneil_import";

static PYTHON_LOAD_LOCK: Mutex<()> = Mutex::new(());

/// Serializes isolated `sys.path` / `sys.modules` snapshots so loads and calls do not overlap.
pub(crate) fn lock_python_loads() -> MutexGuard<'static, ()> {
    PYTHON_LOAD_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Snapshots `sys.path`, puts `module_directory` on it, and drops only local
/// sibling modules when finished.
///
/// Installed and virtualenv packages stay in `sys.modules` so a first `import
/// numpy` is not unloaded and later re-imported as a second copy.
pub(crate) fn isolate_import_state<'py>(
    py: Python<'py>,
    module_directory: &Path,
) -> PyResult<IsolatedImportState<'py>> {
    let sys = PyModule::import(py, "sys")?;
    let path = sys.getattr("path")?.cast_into::<PyList>()?;
    let modules = sys.getattr("modules")?.cast_into::<PyDict>()?;
    let path_snapshot = path.call_method0("copy")?;
    let evicted = evict_local_module_names(&modules, module_directory)?;
    path.insert(0, module_directory.as_os_str())?;

    Ok(IsolatedImportState {
        path,
        modules,
        path_snapshot,
        evicted,
        module_directory: module_directory.to_path_buf(),
    })
}

/// Removes `sys.modules` entries whose names match `.py` files in `module_directory`.
///
/// Non-local modules that used those names (stdlib or site-packages) are returned
/// so they can be put back after the load.
fn evict_local_module_names(
    modules: &Bound<'_, PyDict>,
    module_directory: &Path,
) -> PyResult<Vec<(String, Py<PyAny>)>> {
    let Ok(entries) = std::fs::read_dir(module_directory) else {
        return Ok(Vec::new());
    };

    let mut evicted = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "py")
            && let Some(stem) = path.file_stem().and_then(|stem| stem.to_str())
            && let Ok(Some(module)) = modules.get_item(stem)
        {
            if !is_local_non_venv_module(&module, module_directory) {
                evicted.push((stem.to_string(), module.unbind()));
            }
            let _ = modules.del_item(stem);
        }
    }
    Ok(evicted)
}

/// Drops the Oneil-loaded module and sibling imports from `sys.modules`.
fn drop_local_modules(modules: &Bound<'_, PyDict>, module_directory: &Path) -> PyResult<()> {
    let keys = modules
        .iter()
        .map(|(key, _value)| key.extract::<String>())
        .collect::<PyResult<Vec<_>>>()?;

    for key in keys {
        if key == IMPORTED_MODULE_NAME {
            let _ = modules.del_item(&key);
            continue;
        }
        if let Ok(Some(module)) = modules.get_item(&key)
            && is_local_non_venv_module(&module, module_directory)
        {
            let _ = modules.del_item(&key);
        }
    }
    Ok(())
}

/// Returns whether `module` was loaded from `module_directory` and is not inside a venv.
fn is_local_non_venv_module(module: &Bound<'_, PyAny>, module_directory: &Path) -> bool {
    module
        .getattr("__file__")
        .ok()
        .and_then(|file| file.extract::<String>().ok())
        .is_some_and(|file| {
            let file = PathBuf::from(file);
            file.starts_with(module_directory) && !has_venv_component(&file)
        })
}

/// Returns whether `path` contains a `venv` or `.venv` directory.
fn has_venv_component(path: &Path) -> bool {
    path.components().any(|component| {
        matches!(component, Component::Normal(os_str) if os_str == "venv" || os_str == ".venv")
    })
}

/// Restores `sys.path` and removes only local sibling modules from `sys.modules`.
pub(crate) struct IsolatedImportState<'py> {
    path: Bound<'py, PyList>,
    modules: Bound<'py, PyDict>,
    path_snapshot: Bound<'py, PyAny>,
    evicted: Vec<(String, Py<PyAny>)>,
    module_directory: PathBuf,
}

impl IsolatedImportState<'_> {
    /// Restores `sys.path`, drops local modules, and puts back evicted non-local modules.
    fn restore(&self) -> PyResult<()> {
        self.path.call_method0("clear")?;
        self.path.call_method1("extend", (&self.path_snapshot,))?;
        drop_local_modules(&self.modules, &self.module_directory)?;
        for (name, module) in &self.evicted {
            if !self.modules.contains(name)? {
                self.modules.set_item(name, module)?;
            }
        }
        Ok(())
    }
}

impl Drop for IsolatedImportState<'_> {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

fn insert_oneil_module_into_python(py: Python<'_>) -> PyResult<()> {
    // wrap the oneil_python_module into a Python module
    let oneil_module = wrap_pymodule!(oneil_python_module)(py);

    // Import and get sys.modules
    let sys = PyModule::import(py, "sys")?;
    let py_modules: Bound<'_, PyDict> = sys.getattr("modules")?.cast_into()?;

    // Insert oneil_python_module into sys.modules
    py_modules.set_item("oneil", oneil_module)?;

    Ok(())
}

fn start_tracking_imports(py: Python<'_>, module_directory: &Path) -> PyResult<()> {
    let builtins = PyModule::import(py, "builtins")?;
    let builtins_import_orig = builtins.getattr("__import__")?;

    let import_tracker = ImportTracker::new(module_directory, builtins_import_orig.unbind())?;

    builtins.setattr("__import__", import_tracker)?;

    Ok(())
}

fn stop_tracking_imports(py: Python<'_>) -> PyResult<IndexSet<PathBuf>> {
    let builtins = PyModule::import(py, "builtins")?;
    let builtins_import_orig = builtins.getattr("__import__")?;

    let import_tracker = builtins_import_orig.extract::<ImportTracker>()?;
    let (imports, builtins_import_orig) = import_tracker.into_imports_and_original_import_fn(py);

    builtins.setattr("__import__", builtins_import_orig)?;

    Ok(imports)
}

fn get_doc_string(
    value: &Bound<'_, PyAny>,
    inspect_module: &Bound<'_, PyModule>,
) -> Option<String> {
    inspect_module
        .call_method1("getdoc", (value,))
        .expect("getdoc should not fail")
        .extract::<Option<String>>()
        .expect("getdoc should return either a string or None")
}

fn get_line_no(value: &Bound<'_, PyAny>, inspect_module: &Bound<'_, PyModule>) -> Option<u32> {
    let result = inspect_module
        .call_method1("getsourcelines", (value,))
        // if the call fails, it was probably because
        // the source code could not be retrieved or the
        // function is a builtin function
        .ok()?;

    let (_, line_no) = result
        .extract::<(Vec<String>, u32)>()
        .expect("`getsourcelines` should return a tuple of a string and a u32");

    Some(line_no)
}

fn maybe_requirements_txt_path(module_directory: &Path) -> Option<PathBuf> {
    let requirements_txt_path = module_directory.join("requirements.txt");

    requirements_txt_path
        .exists()
        .then_some(requirements_txt_path)
}

/// Tracks imports made by a Python module.
///
/// Note that only imports from the module directory are tracked. This ensures
/// that imports from builtin libraries and other libraries like `numpy` are not
/// tracked.
#[pyclass(from_py_object)]
#[derive(Debug)]
struct ImportTracker {
    pub module_directory: PathBuf,
    pub imports: Arc<Mutex<IndexSet<PathBuf>>>,
    pub builtins_import_orig: Py<PyAny>,
}

impl ImportTracker {
    pub fn new(module_directory: &Path, builtins_import_orig: Py<PyAny>) -> PyResult<Self> {
        Ok(Self {
            module_directory: module_directory.to_path_buf(),
            imports: Arc::new(Mutex::new(IndexSet::new())),
            builtins_import_orig,
        })
    }

    pub fn into_imports_and_original_import_fn(
        self,
        py: Python<'_>,
    ) -> (IndexSet<PathBuf>, Py<PyAny>) {
        (
            self.imports
                .lock()
                .expect("imports should not be poisoned")
                .clone(),
            self.builtins_import_orig.clone_ref(py),
        )
    }

    pub fn is_local_path(&self, file_path: &Path) -> bool {
        file_path.starts_with(&self.module_directory)
    }

    pub fn is_local_venv_path(&self, file_path: &Path) -> bool {
        has_venv_component(file_path)
    }
}

#[expect(
    clippy::multiple_inherent_impl,
    reason = "this block is for Python, not Rust"
)]
#[pymethods]
impl ImportTracker {
    #[pyo3(signature = (*args, **kwargs))]
    fn __call__<'py>(
        &self,
        py: Python<'py>,
        args: &Bound<'py, PyTuple>,
        kwargs: Option<&Bound<'py, PyDict>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let result = self.builtins_import_orig.bind(py).call(args, kwargs)?;

        let result_module = result.cast::<PyModule>()?;

        // try to get the file path of the imported module
        //
        // built-in modules do not have a __file__ attribute, so we skip them
        if let Ok(file_path) = result_module.getattr("__file__")
            && let Ok(file_path) = file_path.extract::<String>()
        {
            let file_path = PathBuf::from(file_path);

            // if the file path starts with the module directory, add it to the
            // import list
            if self.is_local_path(&file_path) && !self.is_local_venv_path(&file_path) {
                self.imports
                    .lock()
                    .expect("imports should not be poisoned")
                    .insert(file_path);
            }
        }

        Ok(result)
    }
}

impl Clone for ImportTracker {
    fn clone(&self) -> Self {
        Python::attach(|py| Self {
            module_directory: self.module_directory.clone(),
            imports: Arc::clone(&self.imports),
            builtins_import_orig: self.builtins_import_orig.clone_ref(py),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{IMPORTED_MODULE_NAME, load_python_import};
    use crate::function::PythonModule;
    use oneil_shared::paths::PythonPath;
    use oneil_shared::symbols::PyFunctionName;
    use pyo3::Python;
    use pyo3::types::{PyAnyMethods, PyDict, PyDictMethods, PyModule};
    use std::fs;
    use std::path::PathBuf;

    fn fixture_dir(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join(name)
    }

    fn load_fixture(path: PathBuf) -> PythonModule {
        let source = fs::read_to_string(&path).expect("fixture should be readable");
        load_python_import(&PythonPath::from_path_with_ext(&path), &source)
            .expect("fixture should load")
    }

    fn call_int(module: &PythonModule, name: &str) -> i64 {
        Python::attach(|py| {
            let function = module
                .get_function(&PyFunctionName::from(name))
                .expect("function should exist");
            function
                .call(py, &[])
                .expect("function should run")
                .extract()
                .expect("function should return an int")
        })
    }

    fn call_str(module: &PythonModule, name: &str) -> String {
        Python::attach(|py| {
            let function = module
                .get_function(&PyFunctionName::from(name))
                .expect("function should exist");
            function
                .call(py, &[])
                .expect("function should run")
                .extract()
                .expect("function should return a string")
        })
    }

    /// Loads a `.py` file that `import`s another module in the same directory.
    #[test]
    fn loaded_module_can_import_sibling_python_files() {
        let dir = fixture_dir("sibling_import");
        let sibling_path = dir.join("util.py");
        let module = load_fixture(dir.join("helpers.py"));

        assert!(
            module
                .get_function_names()
                .any(|name| name.as_str() == "run"),
            "loaded module should expose `run`"
        );

        let sibling_canonical = sibling_path
            .canonicalize()
            .expect("sibling python file should exist");
        assert!(
            module.get_imports().iter().any(|path| {
                path == &sibling_canonical
                    || path.canonicalize().ok().as_ref() == Some(&sibling_canonical)
            }),
            "sibling python file should be tracked as a local import"
        );
    }

    /// Sibling imports inside `run()` still resolve after `sys.path` is restored.
    #[test]
    fn call_resolves_sibling_imports_inside_run() {
        let module = load_fixture(fixture_dir("sibling_import").join("helpers.py"));
        assert_eq!(call_int(&module, "late_run"), 42);
    }

    /// Same-stem modules keep their own sibling imports, including imports inside `run()`.
    #[test]
    fn same_stem_modules_use_their_own_sibling_imports() {
        let a = load_fixture(fixture_dir("same_stem/a").join("helpers.py"));
        let b = load_fixture(fixture_dir("same_stem/b").join("helpers.py"));

        assert_eq!(call_int(&a, "run"), 1);
        assert_eq!(call_int(&b, "run"), 2);
        assert_eq!(call_int(&a, "run"), 1);
    }

    /// Installed and stdlib modules stay in `sys.modules` after a load.
    #[test]
    fn installed_modules_remain_cached_after_load() {
        let module = load_fixture(fixture_dir("installed_modules").join("helpers.py"));
        assert_eq!(call_int(&module, "circle_area"), 3);
        assert!(
            sys_modules_contains("math"),
            "stdlib/site-packages modules should stay cached"
        );
        assert_eq!(call_int(&module, "late_math"), 3);
    }

    fn sys_modules_contains(name: &str) -> bool {
        Python::attach(|py| {
            let sys = PyModule::import(py, "sys").expect("sys should import");
            let modules = sys
                .getattr("modules")
                .expect("sys.modules should exist")
                .cast_into::<PyDict>()
                .expect("sys.modules should be a dict");
            modules.contains(name).expect("contains should not fail")
        })
    }

    /// A user `inspect.py` does not replace the standard library `inspect` module.
    #[test]
    fn user_inspect_module_does_not_shadow_stdlib() {
        assert_ne!(IMPORTED_MODULE_NAME, "inspect");

        let module = load_fixture(fixture_dir("inspect_name").join("inspect.py"));
        assert_eq!(call_str(&module, "tagged"), "user");
        assert_eq!(
            module.get_docs(),
            Some("A user module whose file stem matches the stdlib `inspect` module.")
        );
    }
}
