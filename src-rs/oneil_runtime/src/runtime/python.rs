//! Python import and evaluation for the runtime (when the `python` feature is enabled).

use std::path::Path;

use indexmap::IndexSet;
use oneil_eval::EvalError;
use oneil_python::LoadPythonImportError;
use oneil_python::function::PythonFunctionMap;
use oneil_shared::load_result::LoadResult;
use oneil_shared::span::Span;

use super::Runtime;
use crate::output::{self, error::RuntimeErrors, ir};

impl Runtime {
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
        path: impl AsRef<Path>,
    ) -> (Option<IndexSet<&str>>, RuntimeErrors) {
        let path = path.as_ref();
        self.load_python_import_internal(path);

        let is_success = self
            .python_import_cache
            .get_entry(path)
            .is_some_and(LoadResult::is_success);

        let errors = if is_success {
            RuntimeErrors::new()
        } else {
            self.get_python_import_errors(path)
        };

        let names_opt = self
            .python_import_cache
            .get_entry(path)
            .and_then(LoadResult::value)
            .map(|functions| functions.get_function_names().collect());

        (names_opt, errors)
    }

    pub(super) fn load_python_import_internal(
        &mut self,
        path: impl AsRef<Path>,
    ) -> &LoadResult<PythonFunctionMap, LoadPythonImportError> {
        let path = path.as_ref();

        // load the source code from the file
        let Ok(source) = self.load_source(path) else {
            self.python_import_cache
                .insert(path.to_path_buf(), LoadResult::failure());

            return self
                .python_import_cache
                .get_entry(path)
                .expect("it was just inserted");
        };

        // load the Python module and return the set of functions
        let functions_result = oneil_python::load_python_import(path, source);

        // insert the result into the cache
        match functions_result {
            Ok(functions) => self
                .python_import_cache
                .insert(path.to_path_buf(), LoadResult::success(functions)),

            Err(e) => self.python_import_cache.insert(
                path.to_path_buf(),
                LoadResult::partial(PythonFunctionMap::default(), e),
            ),
        }

        // return the cached result
        self.python_import_cache
            .get_entry(path)
            .expect("entry was inserted in this function for the requested path")
    }

    /// Evaluates a Python function by path and identifier.
    pub(super) fn evaluate_python_function(
        &self,
        python_path: &ir::PythonPath,
        identifier: &ir::Identifier,
        identifier_span: Span,
        args: Vec<(output::Value, Span)>,
    ) -> Option<Result<output::Value, Box<EvalError>>> {
        let python_functions = self
            .python_import_cache
            .get_entry(python_path.as_ref())?
            .value()
            .expect("should not be trying to evaluate a Python function if the import failed");

        let function = python_functions.get_function(identifier.as_str())?;

        let eval_result = oneil_python::evaluate_python_function(
            function,
            identifier.as_str(),
            identifier_span,
            args,
        );

        Some(eval_result.map_err(|e| {
            Box::new(EvalError::PythonEvalError {
                function_name: e.function_name,
                identifier_span: e.identifier_span,
                message: e.message,
            })
        }))
    }
}
