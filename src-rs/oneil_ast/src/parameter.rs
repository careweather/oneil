//! Parameter constructs for the AST
//!
//! This module contains structures for representing parameters in Oneil programs,
//! including parameter definitions, values, limits, and performance markers.

use crate::{
    debug_info::TraceLevelNode,
    expression::ExprNode,
    naming::{IdentifierNode, LabelNode},
    node::Node,
    note::NoteNode,
    unit::UnitExprNode,
};

/// A parameter in an Oneil program
///
/// Parameters are used to define the values of variables in the model.
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    label: LabelNode,
    ident: IdentifierNode,
    value: ParameterValueNode,
    limits: Option<LimitsNode>,
    performance_marker: Option<PerformanceMarkerNode>,
    trace_level: Option<TraceLevelNode>,
    note: Option<NoteNode>,
}

/// A node containing a parameter definition
pub type ParameterNode = Node<Parameter>;

impl Parameter {
    /// Creates a new parameter with the given components
    pub fn new(
        label: LabelNode,
        ident: IdentifierNode,
        value: ParameterValueNode,
        limits: Option<LimitsNode>,
        performance_marker: Option<PerformanceMarkerNode>,
        trace_level: Option<TraceLevelNode>,
        note: Option<NoteNode>,
    ) -> Self {
        Self {
            label,
            ident,
            value,
            limits,
            performance_marker,
            trace_level,
            note,
        }
    }

    /// Returns the label of this parameter
    pub fn label(&self) -> &LabelNode {
        &self.label
    }

    /// Returns the identifier of this parameter
    pub fn ident(&self) -> &IdentifierNode {
        &self.ident
    }

    /// Returns the value of this parameter
    pub fn value(&self) -> &ParameterValueNode {
        &self.value
    }

    /// Returns the optional limits for this parameter
    pub fn limits(&self) -> Option<&LimitsNode> {
        self.limits.as_ref()
    }

    /// Returns the optional performance marker for this parameter
    pub fn performance_marker(&self) -> Option<&PerformanceMarkerNode> {
        self.performance_marker.as_ref()
    }

    /// Returns the optional trace level for this parameter
    pub fn trace_level(&self) -> Option<&TraceLevelNode> {
        self.trace_level.as_ref()
    }

    /// Returns the optional note attached to this parameter
    pub fn note(&self) -> Option<&NoteNode> {
        self.note.as_ref()
    }
}

/// A value assigned to a parameter.
///
/// Parameter values can be either simple expressions or piecewise expressions
/// that evaluate to different values based on conditions.
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterValue {
    /// A simple parameter value with an expression and optional unit
    Simple(ExprNode, Option<UnitExprNode>),
    /// A piecewise parameter value with multiple conditional parts and optional unit
    Piecewise(Vec<PiecewisePartNode>, Option<UnitExprNode>),
}

/// A node containing a parameter value
pub type ParameterValueNode = Node<ParameterValue>;

impl ParameterValue {
    /// Creates a simple parameter value with an expression and optional unit
    pub fn simple(expr: ExprNode, unit: Option<UnitExprNode>) -> Self {
        Self::Simple(expr, unit)
    }

    /// Creates a piecewise parameter value with parts and optional unit
    pub fn piecewise(parts: Vec<PiecewisePartNode>, unit: Option<UnitExprNode>) -> Self {
        Self::Piecewise(parts, unit)
    }
}

/// Parameter limits that constrain the allowed values
///
/// Limits can be either continuous (with min/max bounds) or discrete
/// (with a specific set of allowed values).
#[derive(Debug, Clone, PartialEq)]
pub enum Limits {
    /// Continuous limits with minimum and maximum values
    Continuous {
        /// The minimum value
        min: ExprNode,
        /// The maximum value
        max: ExprNode,
    },
    /// Discrete limits with a list of allowed values
    Discrete {
        /// The list of allowed values
        values: Vec<ExprNode>,
    },
}

/// A node containing parameter limits
pub type LimitsNode = Node<Limits>;

impl Limits {
    /// Creates continuous limits with minimum and maximum values
    pub fn continuous(min: ExprNode, max: ExprNode) -> Self {
        Self::Continuous { min, max }
    }

    /// Creates discrete limits with a list of allowed values
    pub fn discrete(values: Vec<ExprNode>) -> Self {
        Self::Discrete { values }
    }
}

/// A performance marker for optimization purposes
#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceMarker;

/// A node containing a performance marker
pub type PerformanceMarkerNode = Node<PerformanceMarker>;

impl PerformanceMarker {
    /// Creates a new performance marker
    pub fn new() -> Self {
        Self
    }
}

/// A part of a piecewise parameter value with an expression and condition
#[derive(Debug, Clone, PartialEq)]
pub struct PiecewisePart {
    expr: ExprNode,
    if_expr: ExprNode,
}

/// A node containing a piecewise part
pub type PiecewisePartNode = Node<PiecewisePart>;

impl PiecewisePart {
    /// Creates a new piecewise part with expression and condition
    pub fn new(expr: ExprNode, if_expr: ExprNode) -> Self {
        Self { expr, if_expr }
    }

    /// Returns the expression value for this piecewise part
    pub fn expr(&self) -> &ExprNode {
        &self.expr
    }

    /// Returns the condition expression for this piecewise part
    pub fn if_expr(&self) -> &ExprNode {
        &self.if_expr
    }
}
