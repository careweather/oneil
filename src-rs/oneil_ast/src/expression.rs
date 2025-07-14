use crate::{
    atom::{BooleanNode, IdentifierNode, NumberNode, StrNode},
    node::Node,
};

/// An expression in the Oneil language
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    BinaryOp {
        op: BinaryOpNode,
        left: Box<ExprNode>,
        right: Box<ExprNode>,
    },

    UnaryOp {
        op: UnaryOpNode,
        expr: Box<ExprNode>,
    },

    FunctionCall {
        name: IdentifierNode,
        args: Vec<ExprNode>,
    },

    Variable(VariableNode),

    Literal(LiteralNode),
}

impl Expr {
    pub fn binary_op(op: BinaryOpNode, left: ExprNode, right: ExprNode) -> Self {
        Self::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn unary_op(op: UnaryOpNode, expr: ExprNode) -> Self {
        Self::UnaryOp {
            op,
            expr: Box::new(expr),
        }
    }

    pub fn function_call(name: IdentifierNode, args: Vec<ExprNode>) -> Self {
        Self::FunctionCall { name, args }
    }

    pub fn variable(var: VariableNode) -> Self {
        Self::Variable(var)
    }

    pub fn literal(lit: LiteralNode) -> Self {
        Self::Literal(lit)
    }
}

pub type ExprNode = Node<Expr>;

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

impl BinaryOp {
    pub fn add() -> Self {
        Self::Add
    }

    pub fn sub() -> Self {
        Self::Sub
    }

    pub fn true_sub() -> Self {
        Self::TrueSub
    }

    pub fn mul() -> Self {
        Self::Mul
    }

    pub fn div() -> Self {
        Self::Div
    }

    pub fn true_div() -> Self {
        Self::TrueDiv
    }

    pub fn modulo() -> Self {
        Self::Mod
    }

    pub fn pow() -> Self {
        Self::Pow
    }

    pub fn less_than() -> Self {
        Self::LessThan
    }

    pub fn less_than_eq() -> Self {
        Self::LessThanEq
    }

    pub fn greater_than() -> Self {
        Self::GreaterThan
    }

    pub fn greater_than_eq() -> Self {
        Self::GreaterThanEq
    }

    pub fn eq() -> Self {
        Self::Eq
    }

    pub fn not_eq() -> Self {
        Self::NotEq
    }

    pub fn and() -> Self {
        Self::And
    }

    pub fn or() -> Self {
        Self::Or
    }

    pub fn min_max() -> Self {
        Self::MinMax
    }
}

pub type BinaryOpNode = Node<BinaryOp>;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum UnaryOp {
    Neg,
    Not,
}

impl UnaryOp {
    pub fn neg() -> Self {
        Self::Neg
    }

    pub fn not() -> Self {
        Self::Not
    }
}

pub type UnaryOpNode = Node<UnaryOp>;

#[derive(Debug, Clone, PartialEq)]
pub enum Variable {
    Identifier(IdentifierNode),
    Accessor {
        parent: IdentifierNode,
        component: Box<VariableNode>,
    },
}

impl Variable {
    pub fn identifier(id: IdentifierNode) -> Self {
        Self::Identifier(id)
    }

    pub fn accessor(parent: IdentifierNode, component: VariableNode) -> Self {
        Self::Accessor {
            parent,
            component: Box::new(component),
        }
    }
}

pub type VariableNode = Node<Variable>;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(NumberNode),
    String(StrNode),
    Boolean(BooleanNode),
}

impl Literal {
    pub fn number(num: NumberNode) -> Self {
        Self::Number(num)
    }

    pub fn string(str: StrNode) -> Self {
        Self::String(str)
    }

    pub fn boolean(bool: BooleanNode) -> Self {
        Self::Boolean(bool)
    }
}

pub type LiteralNode = Node<Literal>;
