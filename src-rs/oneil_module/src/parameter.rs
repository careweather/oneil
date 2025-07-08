use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

use crate::{debug_info::TraceLevel, expr::Expr, reference::Identifier, unit::CompositeUnit};

#[derive(Debug, Clone, PartialEq)]
pub struct ParameterCollection {
    parameters: HashMap<Identifier, Parameter>,
}

impl ParameterCollection {
    pub fn new(parameters: HashMap<Identifier, Parameter>) -> Self {
        Self { parameters }
    }

    // TODO: add methods for getting performance parameters, evaluation order, etc.
}

impl Deref for ParameterCollection {
    type Target = HashMap<Identifier, Parameter>;

    fn deref(&self) -> &Self::Target {
        &self.parameters
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    dependencies: HashSet<Identifier>,
    ident: Identifier,
    value: ParameterValue,
    limits: Limits,
    is_performance: bool,
    trace_level: TraceLevel,
}

impl Parameter {
    pub fn new(
        dependencies: HashSet<Identifier>,
        ident: Identifier,
        value: ParameterValue,
        limits: Limits,
        is_performance: bool,
        trace_level: TraceLevel,
    ) -> Self {
        Self {
            dependencies,
            ident,
            value,
            limits,
            is_performance,
            trace_level,
        }
    }

    /// Returns a reference to the set of parameter dependencies.
    ///
    /// Dependencies are the identifiers of other parameters that this parameter
    /// depends on for its value calculation.
    pub fn dependencies(&self) -> &HashSet<Identifier> {
        &self.dependencies
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParameterValue {
    Simple(Expr, Option<CompositeUnit>),
    Piecewise(Vec<PiecewiseExpr>, Option<CompositeUnit>),
}

impl ParameterValue {
    pub fn simple(expr: Expr, unit: Option<CompositeUnit>) -> Self {
        Self::Simple(expr, unit)
    }

    pub fn piecewise(exprs: Vec<PiecewiseExpr>, unit: Option<CompositeUnit>) -> Self {
        Self::Piecewise(exprs, unit)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PiecewiseExpr {
    expr: Expr,
    if_expr: Expr,
}

impl PiecewiseExpr {
    pub fn new(expr: Expr, if_expr: Expr) -> Self {
        Self { expr, if_expr }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Limits {
    Default,
    Continuous { min: Expr, max: Expr },
    Discrete { values: Vec<Expr> },
}

impl Limits {
    pub fn default() -> Self {
        Self::Default
    }
}

impl Default for Limits {
    fn default() -> Self {
        Self::Default
    }
}

impl Limits {
    pub fn continuous(min: Expr, max: Expr) -> Self {
        Self::Continuous { min, max }
    }

    pub fn discrete(values: Vec<Expr>) -> Self {
        Self::Discrete { values }
    }
}
