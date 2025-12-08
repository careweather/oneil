use std::{collections::HashMap, path::PathBuf};

use oneil_shared::span::Span;

use crate::value::{Unit, Value};

#[derive(Debug, Clone)]
pub struct Model {
    path: PathBuf,
    submodels: HashMap<String, Model>,
    parameters: HashMap<String, Parameter>,
    tests: HashMap<String, Test>,
}

#[derive(Debug, Clone)]
pub struct Test {
    expr_span: Span,
    value: Value,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    ident: String,
    label: String,
    value: Value,
    unit: Unit,
    is_performance: bool,
    trace: bool,
    dependency_results: HashMap<String, Value>,
}
