//! Parameter definitions and management for Oneil model IR.

use std::collections::HashSet;

use oneil_shared::span::Span;

use crate::{debug_info::TraceLevel, expr::Expr, unit::CompositeUnit};

/// A name for a parameter.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParameterName(String);

impl ParameterName {
    /// Creates a new parameter name with the given name.
    #[must_use]
    pub const fn new(name: String) -> Self {
        Self(name)
    }

    /// Returns the parameter name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A label for a parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label(String);

impl Label {
    /// Creates a new label with the given name.
    #[must_use]
    pub const fn new(name: String) -> Self {
        Self(name)
    }

    /// Returns the label as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Represents a single parameter in an Oneil model.
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    dependencies: HashSet<ParameterName>,
    name: ParameterName,
    name_span: Span,
    span: Span,
    label: Label,
    value: ParameterValue,
    limits: Limits,
    is_performance: bool,
    trace_level: TraceLevel,
}

impl Parameter {
    /// Creates a new parameter with the specified properties.
    #[expect(clippy::too_many_arguments, reason = "this is a constructor")]
    #[must_use]
    pub const fn new(
        dependencies: HashSet<ParameterName>,
        name: ParameterName,
        name_span: Span,
        span: Span,
        label: Label,
        value: ParameterValue,
        limits: Limits,
        is_performance: bool,
        trace_level: TraceLevel,
    ) -> Self {
        Self {
            dependencies,
            name,
            name_span,
            span,
            label,
            value,
            limits,
            is_performance,
            trace_level,
        }
    }

    /// Returns a reference to the set of parameter dependencies.
    #[must_use]
    pub const fn dependencies(&self) -> &HashSet<ParameterName> {
        &self.dependencies
    }

    /// Returns the name of this parameter.
    #[must_use]
    pub const fn name(&self) -> &ParameterName {
        &self.name
    }

    /// Returns the span of this parameter's identifier.
    #[must_use]
    pub const fn name_span(&self) -> Span {
        self.name_span
    }

    /// Returns the span covering the entire parameter definition.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the label of this parameter.
    #[must_use]
    pub const fn label(&self) -> &Label {
        &self.label
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
    Simple(Expr, Option<CompositeUnit>),
    /// A piecewise function with multiple expressions and conditions.
    Piecewise(Vec<PiecewiseExpr>, Option<CompositeUnit>),
}

impl ParameterValue {
    /// Creates a simple parameter value from an expression and optional unit.
    #[must_use]
    pub const fn simple(expr: Expr, unit: Option<CompositeUnit>) -> Self {
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
    expr: Expr,
    if_expr: Expr,
}

impl PiecewiseExpr {
    /// Creates a new piecewise expression with a value and condition.
    #[must_use]
    pub const fn new(expr: Expr, if_expr: Expr) -> Self {
        Self { expr, if_expr }
    }

    /// Returns the expression value.
    #[must_use]
    pub const fn expr(&self) -> &Expr {
        &self.expr
    }

    /// Returns the condition expression.
    #[must_use]
    pub const fn if_expr(&self) -> &Expr {
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
        min: Box<Expr>,
        /// The maximum allowed value expression.
        max: Box<Expr>,
        /// The span of the expression representing the limit.
        limit_expr_span: Span,
    },
    /// Discrete set of allowed values.
    Discrete {
        /// Vector of expressions representing allowed values.
        values: Vec<Expr>,
        /// The span of the expression representing the limit.
        limit_expr_span: Span,
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
    pub fn continuous(min: Expr, max: Expr, limit_expr_span: Span) -> Self {
        Self::Continuous {
            min: Box::new(min),
            max: Box::new(max),
            limit_expr_span,
        }
    }

    /// Creates discrete limits with a set of allowed values.
    #[must_use]
    pub const fn discrete(values: Vec<Expr>, limit_expr_span: Span) -> Self {
        Self::Discrete {
            values,
            limit_expr_span,
        }
    }
}

impl Default for Limits {
    fn default() -> Self {
        Self::Default
    }
}
