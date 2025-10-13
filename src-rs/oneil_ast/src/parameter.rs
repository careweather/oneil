//! Parameter constructs for the AST

use crate::{
    debug_info::TraceLevelNode,
    expression::ExprNode,
    naming::{IdentifierNode, LabelNode},
    node::Node,
    note::NoteNode,
    unit::UnitExprNode,
};

/// A parameter in an Oneil program
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
    #[must_use]
    pub const fn new(
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
    #[must_use]
    pub const fn label(&self) -> &LabelNode {
        &self.label
    }

    /// Returns the identifier of this parameter
    #[must_use]
    pub const fn ident(&self) -> &IdentifierNode {
        &self.ident
    }

    /// Returns the value of this parameter
    #[must_use]
    pub const fn value(&self) -> &ParameterValueNode {
        &self.value
    }

    /// Returns the optional limits for this parameter
    #[must_use]
    pub const fn limits(&self) -> Option<&LimitsNode> {
        self.limits.as_ref()
    }

    /// Returns the optional performance marker for this parameter
    #[must_use]
    pub const fn performance_marker(&self) -> Option<&PerformanceMarkerNode> {
        self.performance_marker.as_ref()
    }

    /// Returns the optional trace level for this parameter
    #[must_use]
    pub const fn trace_level(&self) -> Option<&TraceLevelNode> {
        self.trace_level.as_ref()
    }

    /// Returns the optional note attached to this parameter
    #[must_use]
    pub const fn note(&self) -> Option<&NoteNode> {
        self.note.as_ref()
    }
}

/// A value assigned to a parameter.
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
    #[must_use]
    pub const fn simple(expr: ExprNode, unit: Option<UnitExprNode>) -> Self {
        Self::Simple(expr, unit)
    }

    /// Creates a piecewise parameter value with parts and optional unit
    #[must_use]
    pub const fn piecewise(parts: Vec<PiecewisePartNode>, unit: Option<UnitExprNode>) -> Self {
        Self::Piecewise(parts, unit)
    }
}

/// Parameter limits that constrain the allowed values
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
    #[must_use]
    pub const fn continuous(min: ExprNode, max: ExprNode) -> Self {
        Self::Continuous { min, max }
    }

    /// Creates discrete limits with a list of allowed values
    #[must_use]
    pub const fn discrete(values: Vec<ExprNode>) -> Self {
        Self::Discrete { values }
    }
}

/// A performance marker for optimization purposes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PerformanceMarker;

/// A node containing a performance marker
pub type PerformanceMarkerNode = Node<PerformanceMarker>;

impl PerformanceMarker {
    /// Creates a new performance marker
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for PerformanceMarker {
    fn default() -> Self {
        Self::new()
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
    #[must_use]
    pub const fn new(expr: ExprNode, if_expr: ExprNode) -> Self {
        Self { expr, if_expr }
    }

    /// Returns the expression value for this piecewise part
    #[must_use]
    pub const fn expr(&self) -> &ExprNode {
        &self.expr
    }

    /// Returns the condition expression for this piecewise part
    #[must_use]
    pub const fn if_expr(&self) -> &ExprNode {
        &self.if_expr
    }
}
