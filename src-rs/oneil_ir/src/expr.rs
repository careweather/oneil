//! Expression system for mathematical and logical operations in Oneil.

use crate::{
    ParameterName,
    reference::{Identifier, ModelPath},
};

/// Abstract syntax tree for mathematical and logical expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Comparison operation with left and right operands, supporting chaining.
    ComparisonOp {
        /// The comparison operator to apply.
        op: ComparisonOp,
        /// The left-hand operand.
        left: Box<Expr>,
        /// The right-hand operand.
        right: Box<Expr>,
        /// Chained comparison operations (order matters).
        rest_chained: Vec<(ComparisonOp, Expr)>,
    },
    /// Binary operation combining two expressions with an operator.
    BinaryOp {
        /// The binary operator to apply.
        op: BinaryOp,
        /// The left-hand operand.
        left: Box<Expr>,
        /// The right-hand operand.
        right: Box<Expr>,
    },
    /// Unary operation applied to a single expression.
    UnaryOp {
        /// The unary operator to apply.
        op: UnaryOp,
        /// The operand expression.
        expr: Box<Expr>,
    },
    /// Function call with a name and argument list.
    FunctionCall {
        /// The name of the function to call.
        name: FunctionName,
        /// The arguments to pass to the function.
        args: Vec<Expr>,
    },
    /// Variable reference (local, parameter, or external).
    Variable(Variable),
    /// Constant literal value.
    Literal {
        /// The literal value.
        value: Literal,
    },
}

impl Expr {
    /// Creates a comparison operation expression.
    #[must_use]
    pub fn comparison_op(
        op: ComparisonOp,
        left: Self,
        right: Self,
        rest_chained: Vec<(ComparisonOp, Self)>,
    ) -> Self {
        Self::ComparisonOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
            rest_chained,
        }
    }

    /// Creates a binary operation expression.
    #[must_use]
    pub fn binary_op(op: BinaryOp, left: Self, right: Self) -> Self {
        Self::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// Creates a unary operation expression.
    #[must_use]
    pub fn unary_op(op: UnaryOp, expr: Self) -> Self {
        Self::UnaryOp {
            op,
            expr: Box::new(expr),
        }
    }

    /// Creates a function call expression.
    #[must_use]
    pub const fn function_call(name: FunctionName, args: Vec<Self>) -> Self {
        Self::FunctionCall { name, args }
    }

    /// Creates a built-in variable reference.
    #[must_use]
    pub const fn builtin_variable(ident: Identifier) -> Self {
        Self::Variable(Variable::Builtin(ident))
    }

    /// Creates a parameter variable reference.
    #[must_use]
    pub const fn parameter_variable(parameter_name: ParameterName) -> Self {
        Self::Variable(Variable::Parameter(parameter_name))
    }

    /// Creates an external variable reference.
    #[must_use]
    pub const fn external_variable(model: ModelPath, parameter_name: ParameterName) -> Self {
        Self::Variable(Variable::External {
            model,
            parameter_name,
        })
    }

    /// Creates a literal expression.
    #[must_use]
    pub const fn literal(value: Literal) -> Self {
        Self::Literal { value }
    }
}

/// Binary operators for mathematical and logical operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    /// Addition: `a + b`
    Add,
    /// Subtraction: `a - b`
    Sub,
    /// Multiplication: `a * b`
    Mul,
    /// Division: `a / b`
    Div,
    /// Modulo: `a % b`
    Mod,
    /// Exponentiation: `a ^ b`
    Pow,
    /// Logical AND: `a && b`
    And,
    /// Logical OR: `a || b`
    Or,
    /// Minimum/maximum: `a | b`
    MinMax,
}

/// Comparison operators for expressions.
///
/// Comparison operations support chaining for expressions like `a < b < c`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOp {
    /// Less than comparison: `a < b`
    LessThan,
    /// Less than or equal comparison: `a <= b`
    LessThanEq,
    /// Greater than comparison: `a > b`
    GreaterThan,
    /// Greater than or equal comparison: `a >= b`
    GreaterThanEq,
    /// Equality comparison: `a == b`
    Eq,
    /// Inequality comparison: `a != b`
    NotEq,
}

impl ComparisonOp {
    /// Creates a less than operator.
    #[must_use]
    pub const fn less_than() -> Self {
        Self::LessThan
    }

    /// Creates a less than or equal operator.
    #[must_use]
    pub const fn less_than_eq() -> Self {
        Self::LessThanEq
    }

    /// Creates a greater than operator.
    #[must_use]
    pub const fn greater_than() -> Self {
        Self::GreaterThan
    }

    /// Creates a greater than or equal operator.
    #[must_use]
    pub const fn greater_than_eq() -> Self {
        Self::GreaterThanEq
    }

    /// Creates an equality operator.
    #[must_use]
    pub const fn eq() -> Self {
        Self::Eq
    }

    /// Creates an inequality operator.
    #[must_use]
    pub const fn not_eq() -> Self {
        Self::NotEq
    }
}

/// Unary operators for single-operand operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    /// Negation: `-a`
    Neg,
    /// Logical NOT: `!a`
    Not,
}

/// Function names for built-in and imported functions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionName {
    /// Built-in mathematical function.
    Builtin(Identifier),
    /// Function imported from a Python module.
    Imported(Identifier),
}

impl FunctionName {
    /// Creates a reference to a built-in function.
    #[must_use]
    pub const fn builtin(name: Identifier) -> Self {
        Self::Builtin(name)
    }

    /// Creates a reference to an imported Python function.
    #[must_use]
    pub const fn imported(name: Identifier) -> Self {
        Self::Imported(name)
    }
}

/// Variable references in expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Variable {
    /// Built-in variable
    Builtin(Identifier),
    /// Parameter defined in the current model.
    Parameter(ParameterName),
    /// Parameter defined in another model.
    External {
        /// The model where the parameter is defined.
        model: ModelPath,
        /// The identifier of the parameter in that model.
        parameter_name: ParameterName,
    },
}

impl Variable {
    /// Creates a built-in variable reference.
    #[must_use]
    pub const fn builtin(ident: Identifier) -> Self {
        Self::Builtin(ident)
    }

    /// Creates a parameter variable reference.
    #[must_use]
    pub const fn parameter(parameter_name: ParameterName) -> Self {
        Self::Parameter(parameter_name)
    }

    /// Creates an external variable reference.
    #[must_use]
    pub const fn external(model: ModelPath, parameter_name: ParameterName) -> Self {
        Self::External {
            model,
            parameter_name,
        }
    }
}

/// Literal values that can appear in expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// Numeric literal (floating-point).
    Number(f64),
    /// String literal.
    String(String),
    /// Boolean literal.
    Boolean(bool),
}

impl Literal {
    /// Creates a numeric literal.
    #[must_use]
    pub const fn number(value: f64) -> Self {
        Self::Number(value)
    }

    /// Creates a string literal.
    #[must_use]
    pub const fn string(value: String) -> Self {
        Self::String(value)
    }

    /// Creates a boolean literal.
    #[must_use]
    pub const fn boolean(value: bool) -> Self {
        Self::Boolean(value)
    }
}
