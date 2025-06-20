/// An expression in the Oneil language
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

    Variable(Vec<String>),

    Literal(Literal),
}

#[derive(Debug, Clone, PartialEq, Copy)]
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
    MinMax,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
}
