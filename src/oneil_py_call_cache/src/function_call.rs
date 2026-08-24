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

#[cfg(test)]
mod tests {
    use oneil_output::{Number, Value};
    use oneil_python::PythonEvalError;
    use serde_json::json;

    use super::FunctionCallResult;

    #[test]
    fn from_ok_result_is_success() {
        let value = Value::Number(Number::Scalar(2.0));

        let result = FunctionCallResult::from(Ok(value.clone()));

        let FunctionCallResult::Success(got) = result else {
            panic!("Expected Success, got {result:?}");
        };
        assert_eq!(got, value);
    }

    #[test]
    fn from_err_result_is_failure() {
        let error = PythonEvalError::InvalidReturnValue {
            value_repr: "None".to_string(),
        };

        let result = FunctionCallResult::from(Err(error.clone()));

        let FunctionCallResult::Failure(got) = result else {
            panic!("Expected Failure, got {result:?}");
        };
        assert_eq!(got, error);
    }

    #[test]
    fn success_converts_back_to_ok() {
        let value = Value::Boolean(true);
        let result = FunctionCallResult::Success(value.clone());

        let converted = Result::<Value, PythonEvalError>::from(result);

        assert_eq!(converted, Ok(value));
    }

    #[test]
    fn failure_converts_back_to_err() {
        let error = PythonEvalError::PyErr {
            message: "boom".to_string(),
            traceback: None,
        };
        let result = FunctionCallResult::Failure(error.clone());

        let converted = Result::<Value, PythonEvalError>::from(result);

        assert_eq!(converted, Err(error));
    }

    #[test]
    fn success_serializes_as_untagged_value() {
        let result = FunctionCallResult::Success(Value::Number(Number::Scalar(2.0)));

        let json = serde_json::to_value(&result).expect("serialize");

        assert_eq!(json, json!(2.0));
    }

    #[test]
    fn failure_serializes_as_python_eval_error() {
        let result = FunctionCallResult::Failure(PythonEvalError::PyErr {
            message: "boom".to_string(),
            traceback: None,
        });

        let json = serde_json::to_value(&result).expect("serialize");

        assert_eq!(
            json,
            json!({
                "error": "py_err",
                "message": "boom",
                "traceback": null
            })
        );
    }

    #[test]
    fn failure_json_does_not_deserialize_as_success() {
        let json = json!({
            "error": "py_err",
            "message": "boom",
            "traceback": null
        });

        let result: FunctionCallResult = serde_json::from_value(json).expect("deserialize");

        let FunctionCallResult::Failure(PythonEvalError::PyErr { message, traceback }) = result
        else {
            panic!("Expected Failure(PyErr), got {result:?}");
        };
        assert_eq!(message, "boom");
        assert_eq!(traceback, None);
    }
}
