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
    pub name: LabelNode,
    pub ident: IdentifierNode,
    pub value: ParameterValueNode,
    pub limits: Option<LimitsNode>,
    pub is_performance: Node<bool>,
    pub trace_level: TraceLevelNode,
    pub note: Option<NoteNode>,
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
    pub parts: Vec<PiecewisePartNode>,
}

pub type PiecewiseExprNode = Node<PiecewiseExpr>;

impl PiecewiseExpr {
    pub fn new(parts: Vec<PiecewisePartNode>) -> Self {
        Self { parts }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PiecewisePart {
    pub expr: ExprNode,
    pub if_expr: ExprNode,
}

pub type PiecewisePartNode = Node<PiecewisePart>;

impl PiecewisePart {
    pub fn new(expr: ExprNode, if_expr: ExprNode) -> Self {
        Self { expr, if_expr }
    }
}
