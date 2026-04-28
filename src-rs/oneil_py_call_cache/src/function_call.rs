//! Row types inside a cache document: imports, parameter/test buckets, and individual calls.

use oneil_output::Value;
use oneil_python::PythonEvalError;
use serde::{Deserialize, Serialize};

use crate::{CachedFunctionName, value::CacheValue};

/// One cached call: function name, inputs, and output value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionCall {
    /// Name of the Python function.
    pub function: CachedFunctionName,
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
    Failure(FunctionCallError),
}

impl From<Result<Value, PythonEvalError>> for FunctionCallResult {
    /// Maps `Err` into [`FunctionCallResult::Failure`]; on `Ok`, converts [`Value`] to [`CacheValue`].
    fn from(value: Result<Value, PythonEvalError>) -> Self {
        match value {
            Ok(v) => Self::Success(CacheValue::from(v)),
            Err(e) => Self::Failure(e.into()),
        }
    }
}

impl From<FunctionCallResult> for Result<Value, PythonEvalError> {
    /// Converts a successful cache row to [`Value`], or returns why the call failed or conversion broke.
    fn from(value: FunctionCallResult) -> Self {
        match value {
            FunctionCallResult::Success(cache_value) => Ok(Value::from(cache_value)),
            FunctionCallResult::Failure(err) => Err(PythonEvalError::from(err)),
        }
    }
}

/// The error type for a function call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "error")]
#[serde(rename_all = "snake_case")]
pub enum FunctionCallError {
    /// The function call failed due to a Python error.
    PythonError {
        /// The error message.
        message: String,
        /// The traceback of the error.
        traceback: Option<String>,
    },
    /// The function call failed due to an invalid return value.
    InvalidReturnValue {
        /// The representation of the return value.
        value_repr: String,
    },
}

impl From<PythonEvalError> for FunctionCallError {
    fn from(value: PythonEvalError) -> Self {
        match value {
            PythonEvalError::PyErr { message, traceback } => {
                Self::PythonError { message, traceback }
            }
            PythonEvalError::InvalidReturnValue { value_repr } => {
                Self::InvalidReturnValue { value_repr }
            }
        }
    }
}

impl From<FunctionCallError> for PythonEvalError {
    fn from(value: FunctionCallError) -> Self {
        match value {
            FunctionCallError::PythonError { message, traceback } => {
                Self::PyErr { message, traceback }
            }
            FunctionCallError::InvalidReturnValue { value_repr } => {
                Self::InvalidReturnValue { value_repr }
            }
        }
    }
}
