//! Row types inside a cache document: imports, parameter/test buckets, and individual calls.

use std::collections::BTreeSet;

use oneil_output::Value;
use oneil_python::PythonEvalError;
use oneil_shared::paths::ModelPath;
use serde::{Deserialize, Serialize};

/// One cached call: function name, inputs, and output value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionCall {
    /// The root models that use this function
    /// when evaluating themselves or their submodels.
    ///
    /// When this is empty, this function call should
    /// be deleted from the cache.
    pub root_models: BTreeSet<ModelPath>,
    /// Argument values passed to the function.
    pub inputs: Vec<Value>,
    /// Return value of the function.
    pub output: FunctionCallResult,
}

/// The result of a single function call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FunctionCallResult {
    /// The function call succeeded.
    Success(Value),
    /// The function call failed.
    Failure(PythonEvalError),
}

impl From<Result<Value, PythonEvalError>> for FunctionCallResult {
    /// Maps `Err` into [`FunctionCallResult::Failure`]; on `Ok`, stores the successful [`Value`].
    fn from(value: Result<Value, PythonEvalError>) -> Self {
        match value {
            Ok(v) => Self::Success(v),
            Err(e) => Self::Failure(e),
        }
    }
}

impl From<FunctionCallResult> for Result<Value, PythonEvalError> {
    /// Converts a successful cache row to [`Value`], or returns why the call failed or conversion broke.
    fn from(value: FunctionCallResult) -> Self {
        match value {
            FunctionCallResult::Success(cache_value) => Ok(cache_value),
            FunctionCallResult::Failure(err) => Err(err),
        }
    }
}
