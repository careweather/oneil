//! Test support for evaluation tests.
//!
//! Provides [`TestExternalContext`] that implements [`ExternalResolutionContext`]
//! with standard builtins included implicitly. In tests, construct an external
//! context with [`TestExternalContext::new`], then pass a mutable reference
//! to it when creating an [`EvalContext`].

use oneil_builtins as builtins;
use oneil_output::{self as output, EvalError, ModelEvalErrors, Unit, Value};
use oneil_shared::{
    EvalInstanceKey,
    load_result::LoadResult,
    paths::ModelPath,
    span::Span,
    symbols::{BuiltinFunctionName, BuiltinValueName, UnitBaseName, UnitPrefix},
};

#[cfg(feature = "python")]
use crate::context::CallsiteInfo;
use crate::context::ExternalEvaluationContext;

/// Returns a [`ModelPath`] for use in tests (path without extension, e.g. `"test"` → `test.on`).
#[must_use]
pub fn test_model_path(s: &str) -> ModelPath {
    ModelPath::from_str_no_ext(s)
}

/// Test double for [`ExternalResolutionContext`] with standard builtins included.
///
/// [`TestExternalContext::new`] creates a context that already has the standard
/// builtin values, functions, units, and prefixes from the [`std`] module.
#[derive(Debug)]
pub struct TestExternalContext {
    builtin_ref: builtins::BuiltinRef,
}

impl TestExternalContext {
    /// Creates a new test external context with standard builtins.
    #[must_use]
    pub fn new() -> Self {
        Self {
            builtin_ref: builtins::BuiltinRef::new(),
        }
    }
}

impl Default for TestExternalContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ExternalEvaluationContext for TestExternalContext {
    fn lookup_builtin_variable(&self, name: &BuiltinValueName) -> Option<&Value> {
        self.builtin_ref.get_value(name)
    }

    fn evaluate_builtin_function(
        &self,
        _name: &BuiltinFunctionName,
        _name_span: Span,
        _args: Vec<(Value, Span)>,
    ) -> Option<Result<Value, Vec<EvalError>>> {
        // TODO: figure out how to solve this - is this the right place for this?
        //       maybe we could move eval tests to snapshot tests?
        panic!("this is unused and causes errors because of circular dependencies")
    }

    #[cfg(feature = "python")]
    fn evaluate_imported_function(
        &mut self,
        _python_path: &oneil_shared::paths::PythonPath,
        _identifier: &oneil_shared::symbols::PyFunctionName,
        _function_call_span: Span,
        _args: Vec<(Value, Span)>,
        _callsite_info: &CallsiteInfo,
    ) -> Option<Result<Value, Box<EvalError>>> {
        // For now, we don't support imported functions in tests
        None
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
