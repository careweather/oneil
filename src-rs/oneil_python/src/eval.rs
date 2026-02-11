use oneil_output::Value;
use oneil_shared::span::Span;

use crate::PythonFunction;

pub fn evaluate_python_function(
    function: &PythonFunction,
    identifier: &str,
    identifier_span: Span,
    args: Vec<(Value, oneil_shared::span::Span)>,
) -> Result<Value, ()> {
    todo!()
}
