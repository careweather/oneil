use super::expression::Expr;
use super::note::Note;
use super::unit::UnitExpr;

/// A parameter in an Oneil program
///
/// Parameters are used to define the values of variables in the model.
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    name: String,
    ident: String,
    value: ParameterValue,
    limits: Limits,
    is_performance: bool,
    trace_level: TraceLevel,
    note: Option<Note>,
}

/// A value assigned to a parameter.
///
/// Parameter values can be either simple expressions or piecewise expressions
/// that evaluate to different values based on conditions.
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterValue {
    Simple(Expr, UnitExpr),
    Piecewise(PiecewiseExpr, UnitExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Limits {
    Continuous { min: Expr, max: Expr },
    Discrete { values: Vec<Expr> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TraceLevel {
    None,
    Trace,
    Debug,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PiecewisePart {
    pub expr: Expr,
    pub if_expr: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PiecewiseExpr {
    pub parts: Vec<PiecewisePart>,
}
