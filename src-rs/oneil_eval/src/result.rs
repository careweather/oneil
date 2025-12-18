use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use oneil_shared::span::Span;

use crate::value::{SizedUnit, Value};

#[derive(Debug, Clone)]
pub struct Model {
    pub path: PathBuf,
    pub submodels: HashMap<String, Model>,
    pub parameters: HashMap<String, Parameter>,
    pub tests: Vec<Test>,
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
    pub unit: Option<SizedUnit>,
    pub is_performance: bool,
    pub trace: TraceLevel,
    pub dependencies: HashSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TraceLevel {
    None,
    Trace,
    Debug,
}
