//! Loading Python code and extracting callables.

use std::{ffi::CString, path::Path};

use indexmap::IndexMap;
use pyo3::prelude::*;

use crate::{PythonFunction, error::LoadPythonImportError};

pub fn load_python_import(
    path: &Path,
    source: &str,
) -> Result<IndexMap<String, PythonFunction>, LoadPythonImportError> {
    // get the module name from the path
    let path = path.to_string_lossy();
    let module_name = path.trim_end_matches(".py").replace('/', ".");

    // convert the path and module name to C strings
    let path_cstr = CString::new(path.as_bytes()).expect("path should not have a null byte");
    let module_name_cstr =
        CString::new(module_name).expect("module name should not have a null byte");

    // convert the source to a C string
    let source_cstr = match CString::new(source) {
        Ok(cstr) => cstr,
        Err(_null_error) => return Err(LoadPythonImportError::SourceHasNullByte),
    };

    let functions = Python::attach(|py| {
        // load the code module
        let code_module = PyModule::from_code(py, &source_cstr, &path_cstr, &module_name_cstr)?;

        // get the functions from the code module
        let functions = code_module
            .dict()
            .iter()
            .filter(|(key, value)| !key.to_string().starts_with("__") && value.is_callable())
            .map(|(key, value)| (key.to_string(), PythonFunction::new(value.unbind())))
            .collect::<IndexMap<_, _>>();

        Ok::<_, PyErr>(functions)
    });

    // return the functions
    match functions {
        Ok(functions) => Ok(functions),
        Err(e) => Err(LoadPythonImportError::CouldNotLoadPythonModule(e)),
    }
}
