use crate::{naming::IdentifierNode, node::Node};

/// Represents a unit expression
#[derive(Debug, Clone, PartialEq)]
pub enum UnitExpr {
    BinaryOp {
        op: UnitOpNode,
        left: Box<UnitExprNode>,
        right: Box<UnitExprNode>,
    },
    Parenthesized {
        expr: Box<UnitExprNode>,
    },
    Unit {
        identifier: IdentifierNode,
        exponent: Option<UnitExponentNode>,
    },
}

pub type UnitExprNode = Node<UnitExpr>;

impl UnitExpr {
    pub fn binary_op(op: UnitOpNode, left: UnitExprNode, right: UnitExprNode) -> Self {
        let left = Box::new(left);
        let right = Box::new(right);
        Self::BinaryOp { op, left, right }
    }

    pub fn parenthesized(expr: UnitExprNode) -> Self {
        Self::Parenthesized {
            expr: Box::new(expr),
        }
    }

    pub fn unit(identifier: IdentifierNode, exponent: Option<UnitExponentNode>) -> Self {
        Self::Unit {
            identifier,
            exponent,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnitOp {
    Multiply,
    Divide,
}

pub type UnitOpNode = Node<UnitOp>;

impl UnitOp {
    pub fn multiply() -> Self {
        Self::Multiply
    }

    pub fn divide() -> Self {
        Self::Divide
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    identifier: IdentifierNode,
    exponent: Option<f64>,
}

pub type UnitNode = Node<Unit>;

#[derive(Debug, Clone, PartialEq)]
pub struct UnitExponent(f64);

pub type UnitExponentNode = Node<UnitExponent>;

impl UnitExponent {
    pub fn new(value: f64) -> Self {
        Self(value)
    }
}
