//! Loading Python code and extracting callables.

use std::{
    ffi::CString,
    iter,
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex},
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

    // get the module name from the file stem so directory components and `..`
    // do not produce an invalid dotted module name
    let module_name = python_module_name(path);

    // convert the path and module name to C strings
    let path_str = path.as_path().to_string_lossy();
    let path_cstr = CString::new(path_str.as_bytes()).expect("path should not have a null byte");
    let module_name_cstr =
        CString::new(module_name).expect("module name should not have a null byte");

    // convert the source to a C string
    let source_cstr = match CString::new(source) {
        Ok(cstr) => cstr,
        Err(_null_error) => return Err(LoadPythonImportError::SourceHasNullByte),
    };

    let functions = Python::attach(|py| {
        insert_oneil_module_into_python(py)?;
        add_module_directory_to_sys_path(py, &module_directory)?;

        // load the code module
        start_tracking_imports(py, &module_directory)?;
        let code_module = PyModule::from_code(py, &source_cstr, &path_cstr, &module_name_cstr)?;
        let imports = stop_tracking_imports(py)?;

        // get the inspect module
        let inspect_module = PyModule::import(py, "inspect")?;

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
                    PythonFunction::new(value, docs, line_no),
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

/// Returns the Python `__name__` used when executing imported source.
///
/// Directory components and `..` are omitted so `../testing/helpers.py` loads as `helpers`.
fn python_module_name(path: &PythonPath) -> String {
    path.as_path()
        .file_stem()
        .and_then(|stem| stem.to_str())
        .expect("python path should have a utf-8 file stem")
        .to_string()
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

// this adds the module directory to the sys.path so that the module can import
// other modules in the module directory
fn add_module_directory_to_sys_path(py: Python<'_>, module_directory: &Path) -> PyResult<()> {
    let sys = PyModule::import(py, "sys")?;
    let path = sys.getattr("path")?.cast_into::<PyList>()?;
    path.insert(0, module_directory.as_os_str())?;
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
        file_path
            .components()
            .any(|component| matches!(component, Component::Normal(os_str) if os_str == "venv" || os_str == ".venv"))
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
    use super::{load_python_import, python_module_name};
    use oneil_shared::paths::PythonPath;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn module_name_is_file_stem() {
        let path = PythonPath::from_str_no_ext("helpers");
        assert_eq!(python_module_name(&path), "helpers");
    }

    #[test]
    fn module_name_ignores_directories() {
        let path = PythonPath::from_str_no_ext("testing/helpers");
        assert_eq!(python_module_name(&path), "helpers");
    }

    #[test]
    fn module_name_ignores_parent_directory() {
        let path = PythonPath::from_str_no_ext("../testing/helpers");
        assert_eq!(python_module_name(&path), "helpers");
    }

    /// Loads a `.py` file that `import`s another module in the same directory.
    #[test]
    fn loaded_module_can_import_sibling_python_files() {
        let dir = std::env::temp_dir().join(format!(
            "oneil-python-sibling-import-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should be after the unix epoch")
                .as_nanos()
        ));
        fs::create_dir_all(&dir).expect("temp directory should be creatable");

        let sibling_path = dir.join("util.py");
        fs::write(&sibling_path, "def double(x):\n    return x * 2\n")
            .expect("sibling python file should be writable");

        let main_path = dir.join("helpers.py");
        let source = "import util\n\ndef run(x):\n    return util.double(x)\n";
        fs::write(&main_path, source).expect("main python file should be writable");

        let python_path = PythonPath::from_path_with_ext(&main_path);
        let module = load_python_import(&python_path, source)
            .expect("python file should load while importing a sibling module");

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

        fs::remove_dir_all(&dir).expect("temp directory should be removable");
    }
}
