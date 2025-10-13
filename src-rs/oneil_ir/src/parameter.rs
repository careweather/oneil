//! Parameter definitions and management for Oneil model IR.

use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

use crate::{
    debug_info::TraceLevel,
    expr::ExprWithSpan,
    reference::{Identifier, IdentifierWithSpan},
    unit::CompositeUnit,
};

/// A collection of parameters that can be managed together.
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterCollection {
    parameters: HashMap<Identifier, Parameter>,
}

impl ParameterCollection {
    /// Creates a new parameter collection from a mapping of identifiers to parameters.
    #[must_use]
    pub const fn new(parameters: HashMap<Identifier, Parameter>) -> Self {
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

/// Represents a single parameter in an Oneil model.
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    dependencies: HashSet<Identifier>,
    ident: IdentifierWithSpan,
    value: ParameterValue,
    limits: Limits,
    is_performance: bool,
    trace_level: TraceLevel,
}

impl Parameter {
    /// Creates a new parameter with the specified properties.
    #[must_use]
    pub const fn new(
        dependencies: HashSet<Identifier>,
        ident: IdentifierWithSpan,
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
    #[must_use]
    pub const fn dependencies(&self) -> &HashSet<Identifier> {
        &self.dependencies
    }

    /// Returns the identifier of this parameter.
    #[must_use]
    pub const fn identifier(&self) -> &IdentifierWithSpan {
        &self.ident
    }

    /// Returns the value of this parameter.
    #[must_use]
    pub const fn value(&self) -> &ParameterValue {
        &self.value
    }

    /// Returns the limits of this parameter.
    #[must_use]
    pub const fn limits(&self) -> &Limits {
        &self.limits
    }

    /// Returns whether this parameter is a performance parameter.
    #[must_use]
    pub const fn is_performance(&self) -> bool {
        self.is_performance
    }

    /// Returns the trace level of this parameter.
    #[must_use]
    pub const fn trace_level(&self) -> TraceLevel {
        self.trace_level
    }
}

/// The value of a parameter, which can be either a simple expression or a piecewise function.
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterValue {
    /// A simple expression with an optional unit.
    Simple(ExprWithSpan, Option<CompositeUnit>),
    /// A piecewise function with multiple expressions and conditions.
    Piecewise(Vec<PiecewiseExpr>, Option<CompositeUnit>),
}

impl ParameterValue {
    /// Creates a simple parameter value from an expression and optional unit.
    #[must_use]
    pub const fn simple(expr: ExprWithSpan, unit: Option<CompositeUnit>) -> Self {
        Self::Simple(expr, unit)
    }

    /// Creates a piecewise parameter value from a list of expressions and conditions.
    #[must_use]
    pub const fn piecewise(exprs: Vec<PiecewiseExpr>, unit: Option<CompositeUnit>) -> Self {
        Self::Piecewise(exprs, unit)
    }
}

/// A single expression in a piecewise function with its associated condition.
#[derive(Debug, Clone, PartialEq)]
pub struct PiecewiseExpr {
    expr: ExprWithSpan,
    if_expr: ExprWithSpan,
}

impl PiecewiseExpr {
    /// Creates a new piecewise expression with a value and condition.
    #[must_use]
    pub const fn new(expr: ExprWithSpan, if_expr: ExprWithSpan) -> Self {
        Self { expr, if_expr }
    }

    /// Returns the expression value.
    #[must_use]
    pub const fn expr(&self) -> &ExprWithSpan {
        &self.expr
    }

    /// Returns the condition expression.
    #[must_use]
    pub const fn if_expr(&self) -> &ExprWithSpan {
        &self.if_expr
    }
}

/// Constraints on valid parameter values.
#[derive(Debug, Clone, PartialEq)]
pub enum Limits {
    /// No constraints on parameter values.
    Default,
    /// Continuous range with minimum and maximum values.
    Continuous {
        /// The minimum allowed value expression.
        min: ExprWithSpan,
        /// The maximum allowed value expression.
        max: ExprWithSpan,
    },
    /// Discrete set of allowed values.
    Discrete {
        /// Vector of expressions representing allowed values.
        values: Vec<ExprWithSpan>,
    },
}

impl Limits {
    /// Creates default limits (no constraints).
    #[must_use]
    pub const fn default() -> Self {
        Self::Default
    }

    /// Creates continuous limits with minimum and maximum expressions.
    #[must_use]
    pub const fn continuous(min: ExprWithSpan, max: ExprWithSpan) -> Self {
        Self::Continuous { min, max }
    }

    /// Creates discrete limits with a set of allowed values.
    #[must_use]
    pub const fn discrete(values: Vec<ExprWithSpan>) -> Self {
        Self::Discrete { values }
    }
}

impl Default for Limits {
    fn default() -> Self {
        Self::Default
    }
}
