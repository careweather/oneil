//! Python import and evaluation for the runtime (when the `python` feature is enabled).

use std::path::Path;

use oneil_eval::EvalError;
use oneil_python::LoadPythonImportError;
use oneil_python::function::PythonFunctionMap;
use oneil_shared::load_result::LoadResult;
use oneil_shared::span::Span;

use super::Runtime;
use crate::output::{self, ir};

impl Runtime {
    /// Loads a Python module from a file path and returns the set of callable names.
    ///
    /// Source is read from the file and passed to the Python loader. Results are
    /// cached; subsequent calls for the same path return the cached result.
    ///
    /// # Errors
    ///
    /// Returns an [`OneilError`] if the file could not be read or Python failed
    /// to load the module.
    #[expect(
        clippy::missing_panics_doc,
        reason = "the panic only happens if an internal invariant is violated"
    )]
    pub fn load_python_import(
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
