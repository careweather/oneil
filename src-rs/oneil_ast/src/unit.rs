use crate::{
    atom::{IdentifierNode, NumberNode},
    node::Node,
};

/// Represents a unit expression
#[derive(Debug, Clone, PartialEq)]
pub enum UnitExpr {
    BinaryOp {
        op: UnitOpNode,
        left: Box<UnitExprNode>,
        right: Box<UnitExprNode>,
    },
    Unit {
        identifier: IdentifierNode,
        exponent: Option<NumberNode>,
    },
}

impl UnitExpr {
    pub fn binary_op(op: UnitOpNode, left: UnitExprNode, right: UnitExprNode) -> Self {
        let left = Box::new(left);
        let right = Box::new(right);
        Self::BinaryOp { op, left, right }
    }

    pub fn unit(identifier: IdentifierNode, exponent: Option<NumberNode>) -> Self {
        Self::Unit {
            identifier,
            exponent,
        }
    }
}

pub type UnitExprNode = Node<UnitExpr>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnitOp {
    Multiply,
    Divide,
}

impl UnitOp {
    pub fn multiply() -> Self {
        Self::Multiply
    }

    pub fn divide() -> Self {
        Self::Divide
    }
}

pub type UnitOpNode = Node<UnitOp>;
