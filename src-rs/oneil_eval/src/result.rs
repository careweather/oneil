use std::{collections::HashMap, path::PathBuf};

use oneil_shared::span::Span;

use crate::value::{Unit, Value};

#[derive(Debug, Clone)]
pub struct Model {
    pub path: PathBuf,
    pub submodels: HashMap<String, Model>,
    pub parameters: HashMap<String, Parameter>,
    pub tests: HashMap<String, Test>,
}

#[derive(Debug, Clone)]
pub struct Test {
    pub expr_span: Span,
    pub value: Value,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub ident: String,
    pub label: String,
    pub value: Value,
    pub unit: Unit,
    pub is_db: bool,
    pub is_performance: bool,
    pub trace: bool,
    pub dependency_results: HashMap<String, Value>,
}
