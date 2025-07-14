use crate::{
    atom::{IdentifierNode, LabelNode},
    debug_info::TraceLevelNode,
    expression::ExprNode,
    node::Node,
    note::NoteNode,
    unit::UnitExprNode,
};

/// A parameter in an Oneil program
///
/// Parameters are used to define the values of variables in the model.
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    name: LabelNode,
    ident: IdentifierNode,
    value: ParameterValueNode,
    limits: Option<LimitsNode>,
    is_performance: Node<bool>,
    trace_level: TraceLevelNode,
    note: Option<NoteNode>,
}

pub type ParameterNode = Node<Parameter>;

impl Parameter {
    pub fn new(
        name: LabelNode,
        ident: IdentifierNode,
        value: ParameterValueNode,
        limits: Option<LimitsNode>,
        is_performance: Node<bool>,
        trace_level: TraceLevelNode,
        note: Option<NoteNode>,
    ) -> Self {
        Self {
            name,
            ident,
            value,
            limits,
            is_performance,
            trace_level,
            note,
        }
    }

    pub fn name(&self) -> &LabelNode {
        &self.name
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

    pub fn is_performance(&self) -> &Node<bool> {
        &self.is_performance
    }

    pub fn trace_level(&self) -> &TraceLevelNode {
        &self.trace_level
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
    Piecewise(PiecewiseExprNode, Option<UnitExprNode>),
}

pub type ParameterValueNode = Node<ParameterValue>;

impl ParameterValue {
    pub fn simple(expr: ExprNode, unit: Option<UnitExprNode>) -> Self {
        Self::Simple(expr, unit)
    }

    pub fn piecewise(expr: PiecewiseExprNode, unit: Option<UnitExprNode>) -> Self {
        Self::Piecewise(expr, unit)
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
pub struct PiecewiseExpr {
    parts: Vec<PiecewisePartNode>,
}

pub type PiecewiseExprNode = Node<PiecewiseExpr>;

impl PiecewiseExpr {
    pub fn new(parts: Vec<PiecewisePartNode>) -> Self {
        Self { parts }
    }

    pub fn parts(&self) -> &[PiecewisePartNode] {
        &self.parts
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
