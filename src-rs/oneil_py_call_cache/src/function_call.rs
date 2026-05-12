//! Row types inside a cache document: imports, parameter/test buckets, and individual calls.

use oneil_output::Value;
use oneil_python::PythonEvalError;
use oneil_shared::symbols::PyFunctionName;
use serde::{Deserialize, Serialize};

use crate::value::CacheValue;

/// One cached call: function name, inputs, and output value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionCall {
    /// Name of the Python function.
    pub function: PyFunctionName,
    /// Argument values passed to the function.
    pub inputs: Vec<CacheValue>,
    /// Return value of the function.
    pub output: FunctionCallResult,
}

/// The result of a single function call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FunctionCallResult {
    /// The function call succeeded.
    Success(CacheValue),
    /// The function call failed.
    Failure(PythonEvalError),
}

impl From<Result<Value, PythonEvalError>> for FunctionCallResult {
    /// Maps `Err` into [`FunctionCallResult::Failure`]; on `Ok`, converts [`Value`] to [`CacheValue`].
    fn from(value: Result<Value, PythonEvalError>) -> Self {
        match value {
            Ok(v) => Self::Success(CacheValue::from(v)),
            Err(e) => Self::Failure(e),
        }
    }
}

impl From<FunctionCallResult> for Result<Value, PythonEvalError> {
    /// Converts a successful cache row to [`Value`], or returns why the call failed or conversion broke.
    fn from(value: FunctionCallResult) -> Self {
        match value {
            FunctionCallResult::Success(cache_value) => Ok(Value::from(cache_value)),
            FunctionCallResult::Failure(err) => Err(err),
        }
    }
}
