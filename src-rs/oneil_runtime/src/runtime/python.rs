//! Python import and evaluation for the runtime (when the `python` feature is enabled).

use oneil_output::EvalError;
use oneil_python::{PythonEvalError, PythonFunction, function::PythonModule};

use crate::error::PythonImportError;
use oneil_shared::{paths::PythonPath, span::Span, symbols::PyFunctionName};

use super::Runtime;
use crate::output::{self, error::RuntimeErrors};

impl Runtime {
    /// Looks up the documentation string for a Python import.
    #[must_use]
    pub fn lookup_python_import_docs(&self, path: &PythonPath) -> Option<&str> {
        self.python_import_cache
            .get_entry(path)?
            .as_ref()
            .ok()?
            .get_docs()
    }

    /// Looks up a Python function by path and name.
    #[must_use]
    pub fn lookup_python_function(
        &self,
        python_path: &PythonPath,
        name: &PyFunctionName,
    ) -> Option<&PythonFunction> {
        self.python_import_cache
            .get_entry(python_path)?
            .as_ref()
            .ok()?
            .get_function(name)
    }

    /// Loads a Python module from a file path and returns the set of callable names.
    ///
    /// Source is read from the file and passed to the Python loader. Results are
    /// cached; subsequent calls for the same path return the cached result.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeErrors`] (via [`get_python_import_errors`](super::Runtime::get_python_import_errors)) if the file could not be read or Python failed to load the module.
    pub fn load_python_import(
        &mut self,
        path: &PythonPath,
    ) -> Result<&PythonModule, RuntimeErrors> {
        self.load_python_import_internal(path);

        let names_opt = self
            .python_import_cache
            .get_entry(path)
            .and_then(|result| result.as_ref().ok());

        let errors = self.get_python_import_errors(path);

        names_opt.ok_or(errors)
    }

    pub(super) fn load_python_import_internal(
        &mut self,
        path: &PythonPath,
    ) -> &Result<PythonModule, PythonImportError> {
        // load the source code from the file
        let Ok(source) = self.load_source_internal(&path.into()) else {
            self.python_import_cache
                .insert(path.clone(), Err(PythonImportError::HasSourceError));

            return self
                .python_import_cache
                .get_entry(path)
                .expect("it was just inserted");
        };

        // load the Python module and return the set of functions
        let functions_result = oneil_python::load_python_import(path, source);

        // insert the result into the cache
        match functions_result {
            Ok(functions) => self.python_import_cache.insert(path.clone(), Ok(functions)),

            Err(e) => self
                .python_import_cache
                .insert(path.clone(), Err(PythonImportError::LoadFailed(e))),
        }

        // return the cached result
        self.python_import_cache
            .get_entry(path)
            .expect("entry was inserted in this function for the requested path")
    }

    /// Evaluates a Python function by path and identifier.
    pub(super) fn evaluate_python_function(
        &self,
        python_path: &PythonPath,
        identifier: &PyFunctionName,
        function_call_span: Span,
        args: Vec<(output::Value, Span)>,
    ) -> Option<Result<output::Value, Box<EvalError>>> {
        let python_functions = self
            .python_import_cache
            .get_entry(python_path)?
            .as_ref()
            .expect("should not be trying to evaluate a Python function if the import failed");

        let function = python_functions.get_function(identifier)?;

        let eval_result = oneil_python::evaluate_python_function(function, args);

        Some(eval_result.map_err(|e| match e {
            PythonEvalError::PyErr { message, traceback } => Box::new(EvalError::PythonEvalError {
                function_name: identifier.clone(),
                function_call_span,
                message,
                traceback,
            }),
            PythonEvalError::InvalidReturnValue { value_repr } => {
                Box::new(EvalError::InvalidPythonReturnValue {
                    function_name: identifier.clone(),
                    function_call_span,
                    value_repr,
                })
            }
        }))
    }
}
