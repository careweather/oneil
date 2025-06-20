/// Represents a unit expression
#[derive(Debug, Clone, PartialEq)]
pub enum UnitExpr {
    BinaryOp {
        op: UnitOp,
        left: Box<UnitExpr>,
        right: Box<UnitExpr>,
    },
    Unit {
        identifier: String,
        exponent: Option<f64>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnitOp {
    Multiply,
    Divide,
}
