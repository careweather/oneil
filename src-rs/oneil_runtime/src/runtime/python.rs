//! Python import and evaluation for the runtime (when the `python` feature is enabled).

use oneil_eval::CallsiteInfo;
use oneil_output::EvalError;
use oneil_python::{PythonEvalError, PythonFunction, function::PythonModule};

use crate::error::PythonImportError;
use oneil_shared::{
    paths::{ModelPath, PythonPath},
    span::Span,
    symbols::{ParameterName, PyFunctionName, TestIndex},
};

use super::Runtime;
use crate::cache::PythonCallCacheRecord;
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
        &mut self,
        python_path: &PythonPath,
        identifier: &PyFunctionName,
        function_call_span: Span,
        args: Vec<(output::Value, Span)>,
        callsite_info: &CallsiteInfo,
    ) -> Option<Result<output::Value, Box<EvalError>>> {
        let args_no_spans: Vec<_> = args.clone().into_iter().map(|(value, _)| value).collect();
        let cached_eval_result = match callsite_info {
            CallsiteInfo::Parameter(instance_key, parameter_name) => self
                .try_load_parameter_cache_entry(
                    &instance_key.model_path,
                    parameter_name,
                    python_path,
                    identifier,
                    &args_no_spans,
                ),
            CallsiteInfo::Test(instance_key, test_index) => self.try_load_test_cache_entry(
                &instance_key.model_path,
                *test_index,
                python_path,
                identifier,
                &args_no_spans,
            ),
            // caching does not apply to other callsites
            CallsiteInfo::Other => None,
        };

        let python_functions = self
            .python_import_cache
            .get_entry(python_path)?
            .as_ref()
            .expect("should not be trying to evaluate a Python function if the import failed");

        let function = python_functions.get_function(identifier)?;

        let eval_result = cached_eval_result
            .unwrap_or_else(|| oneil_python::evaluate_python_function(function, args));

        match callsite_info {
            CallsiteInfo::Parameter(instance_key, parameter_name) => {
                self.add_parameter_cache_entry(
                    &instance_key.model_path,
                    parameter_name,
                    python_path,
                    identifier,
                    &args_no_spans,
                    eval_result.clone(),
                );
            }
            CallsiteInfo::Test(instance_key, test_index) => {
                self.add_test_cache_entry(
                    &instance_key.model_path,
                    *test_index,
                    python_path,
                    identifier,
                    &args_no_spans,
                    eval_result.clone(),
                );
            }
            CallsiteInfo::Other => (),
        }

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

    fn try_load_parameter_cache_entry(
        &mut self,
        model_path: &ModelPath,
        parameter_name: &ParameterName,
        python_path: &PythonPath,
        function_name: &PyFunctionName,
        args: &[output::Value],
    ) -> Option<Result<output::Value, PythonEvalError>> {
        let python_module = self
            .python_import_cache
            .get_entry(python_path)
            .and_then(|result| result.as_ref().ok())
            .expect("should not be trying to load a parameter cache entry if the import failed");

        self.python_call_cache.validate_or_clear_python_import(
            model_path,
            python_path,
            python_module.get_hash(),
        );

        let calls = &self
            .python_call_cache
            .get_parameter_entry(model_path, parameter_name)?;

        calls
            .iter()
            .find(|call| call.function == *function_name && call.inputs == args)
            .map(|call| call.output.clone().into())
    }

    fn try_load_test_cache_entry(
        &mut self,
        model_path: &ModelPath,
        test_index: TestIndex,
        python_path: &PythonPath,
        function_name: &PyFunctionName,
        args: &[output::Value],
    ) -> Option<Result<output::Value, PythonEvalError>> {
        let python_module = self
            .python_import_cache
            .get_entry(python_path)
            .and_then(|result| result.as_ref().ok())
            .expect("should not be trying to load a test cache entry if the import failed");

        self.python_call_cache.validate_or_clear_python_import(
            model_path,
            python_path,
            python_module.get_hash(),
        );

        let calls = &self
            .python_call_cache
            .get_test_entry(model_path, test_index)?;

        calls
            .iter()
            .find(|call| call.function == *function_name && call.inputs == args)
            .map(|call| call.output.clone().into())
    }

    fn add_parameter_cache_entry(
        &mut self,
        model_path: &ModelPath,
        parameter_name: &ParameterName,
        python_path: &PythonPath,
        function_name: &PyFunctionName,
        args: &[output::Value],
        eval_result: Result<output::Value, PythonEvalError>,
    ) {
        let python_module = self
            .python_import_cache
            .get_entry(python_path)
            .and_then(|result| result.as_ref().ok())
            .expect("should not be trying to add a parameter cache entry if the import failed");

        self.python_call_replacement_cache.add_parameter_entry(
            PythonCallCacheRecord {
                model_path,
                python_path,
                function_name,
                args,
                eval_result,
                python_module,
            },
            parameter_name,
        );
    }

    fn add_test_cache_entry(
        &mut self,
        model_path: &ModelPath,
        test_index: TestIndex,
        python_path: &PythonPath,
        function_name: &PyFunctionName,
        args: &[output::Value],
        eval_result: Result<output::Value, PythonEvalError>,
    ) {
        let python_module = self
            .python_import_cache
            .get_entry(python_path)
            .and_then(|result| result.as_ref().ok())
            .expect("should not be trying to add a test cache entry if the import failed");

        self.python_call_replacement_cache.add_test_entry(
            PythonCallCacheRecord {
                model_path,
                python_path,
                function_name,
                args,
                eval_result,
                python_module,
            },
            test_index,
        );
    }
}
