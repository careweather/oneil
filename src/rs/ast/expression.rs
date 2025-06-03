use super::literal::Literal;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    BinaryOp {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },
    Literal(Literal),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    TrueSub,
    Mul,
    Div,
    TrueDiv,
    Mod,
    Pow,
    LessThan,
    LessThanEq,
    GreaterThan,
    GreaterThanEq,
    Eq,
    NotEq,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}
