//! Test support for evaluation tests.
//!
//! Provides [`TestExternalContext`] that implements [`ExternalEvaluationContext`]
//! with standard builtins included implicitly. In tests, construct an external
//! context with [`TestExternalContext::new`], then pass a mutable reference
//! to it when creating an [`EvalContext`].

use std::collections::HashMap;
use std::fmt;

use oneil_builtins as builtins;
use oneil_output::{self as output, EvalError, ModelEvalErrors, Unit, Value};
use oneil_shared::{
    EvalInstanceKey,
    load_result::LoadResult,
    paths::{ModelPath, PythonPath},
    span::Span,
    symbols::{BuiltinFunctionName, BuiltinValueName, PyFunctionName, UnitBaseName, UnitPrefix},
};

use crate::context::ExternalEvaluationContext;

/// Returns a [`ModelPath`] for use in tests (path without extension, e.g. `"test"` → `test.on`).
#[must_use]
pub fn test_model_path(s: &str) -> ModelPath {
    ModelPath::from_str_no_ext(s)
}

/// Handler for a stubbed imported (Python) function in tests.
type ImportedFunctionHandler =
    Box<dyn Fn(Vec<(Value, Span)>) -> Result<Value, Box<EvalError>> + Send>;

/// Test double for [`ExternalEvaluationContext`] with standard builtins included.
///
/// [`TestExternalContext::new`] creates a context that already has the standard
/// builtin values, functions, units, and prefixes from the [`std`] module.
/// Imported Python functions can be stubbed with
/// [`TestExternalContext::register_imported_function`].
pub struct TestExternalContext {
    builtin_ref: builtins::BuiltinRef,
    imported_functions: HashMap<(PythonPath, PyFunctionName), ImportedFunctionHandler>,
}

impl TestExternalContext {
    /// Creates a new test external context with standard builtins.
    #[must_use]
    pub fn new() -> Self {
        Self {
            builtin_ref: builtins::BuiltinRef::new(),
            imported_functions: HashMap::new(),
        }
    }

    /// Registers a stub for an imported Python function.
    ///
    /// When evaluation calls a matching imported function, `handler` is invoked with
    /// the evaluated arguments.
    pub fn register_imported_function(
        &mut self,
        python_path: PythonPath,
        name: PyFunctionName,
        handler: impl Fn(Vec<(Value, Span)>) -> Result<Value, Box<EvalError>> + Send + 'static,
    ) {
        self.imported_functions
            .insert((python_path, name), Box::new(handler));
    }
}

impl Default for TestExternalContext {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for TestExternalContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TestExternalContext")
            .field("builtin_ref", &self.builtin_ref)
            .field(
                "imported_functions",
                &self.imported_functions.keys().collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl ExternalEvaluationContext for TestExternalContext {
    fn lookup_builtin_variable(&self, name: &BuiltinValueName) -> Option<&Value> {
        self.builtin_ref.get_value(name)
    }

    fn evaluate_builtin_function(
        &self,
        name: &BuiltinFunctionName,
        name_span: Span,
        args: Vec<(Value, Span)>,
    ) -> Option<Result<Value, Vec<EvalError>>> {
        let builtin = self.builtin_ref.get_function(name)?;
        Some(builtin.call(name_span, args))
    }

    fn evaluate_imported_function(
        &mut self,
        _root_model: &ModelPath,
        python_path: &PythonPath,
        identifier: &PyFunctionName,
        _function_call_span: Span,
        args: Vec<(Value, Span)>,
    ) -> Option<Result<Value, Box<EvalError>>> {
        let handler = self
            .imported_functions
            .get(&(python_path.clone(), identifier.clone()))?;
        Some(handler(args))
    }

    fn lookup_unit(&self, name: &UnitBaseName) -> Option<&Unit> {
        self.builtin_ref.get_unit(name)
    }

    fn lookup_prefix(&self, name: &UnitPrefix) -> Option<f64> {
        self.builtin_ref.get_prefix(name)
    }

    fn get_preloaded_models(
        &self,
    ) -> impl Iterator<Item = (EvalInstanceKey, &LoadResult<output::Model, ModelEvalErrors>)> {
        std::iter::empty()
    }
}
