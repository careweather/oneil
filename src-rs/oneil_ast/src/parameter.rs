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

pub type ParameterNode = Node<Parameter>;

impl Parameter {
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

    pub fn label(&self) -> &LabelNode {
        &self.label
    }

    pub fn ident(&self) -> &IdentifierNode {
        &self.ident
    }

    pub fn value(&self) -> &ParameterValueNode {
        &self.value
    }

    pub fn limits(&self) -> Option<&LimitsNode> {
        self.limits.as_ref()
    }

    pub fn performance_marker(&self) -> Option<&PerformanceMarkerNode> {
        self.performance_marker.as_ref()
    }

    pub fn trace_level(&self) -> Option<&TraceLevelNode> {
        self.trace_level.as_ref()
    }

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
    Simple(ExprNode, Option<UnitExprNode>),
    Piecewise(Vec<PiecewisePartNode>, Option<UnitExprNode>),
}

pub type ParameterValueNode = Node<ParameterValue>;

impl ParameterValue {
    pub fn simple(expr: ExprNode, unit: Option<UnitExprNode>) -> Self {
        Self::Simple(expr, unit)
    }

    pub fn piecewise(parts: Vec<PiecewisePartNode>, unit: Option<UnitExprNode>) -> Self {
        Self::Piecewise(parts, unit)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Limits {
    Continuous { min: ExprNode, max: ExprNode },
    Discrete { values: Vec<ExprNode> },
}

pub type LimitsNode = Node<Limits>;

impl Limits {
    pub fn continuous(min: ExprNode, max: ExprNode) -> Self {
        Self::Continuous { min, max }
    }

    pub fn discrete(values: Vec<ExprNode>) -> Self {
        Self::Discrete { values }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceMarker;

pub type PerformanceMarkerNode = Node<PerformanceMarker>;

impl PerformanceMarker {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PiecewisePart {
    expr: ExprNode,
    if_expr: ExprNode,
}

pub type PiecewisePartNode = Node<PiecewisePart>;

impl PiecewisePart {
    pub fn new(expr: ExprNode, if_expr: ExprNode) -> Self {
        Self { expr, if_expr }
    }

    pub fn expr(&self) -> &ExprNode {
        &self.expr
    }

    pub fn if_expr(&self) -> &ExprNode {
        &self.if_expr
    }
}
