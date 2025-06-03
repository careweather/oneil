use super::expression::Expr;

#[derive(Debug, Clone, PartialEq)]
pub enum ParameterValue {
    Simple(Expr),
    Piecewise(PiecewiseExpr),
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
