//! Expression system for mathematical and logical operations in Oneil.

use oneil_shared::span::Span;

use crate::{
    ParameterName,
    reference::{Identifier, ModelPath},
};

/// Abstract syntax tree for mathematical and logical expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Comparison operation with left and right operands, supporting chaining.
    ComparisonOp {
        /// Span of the entire comparison expression.
        span: Span,
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
        /// Span of the expression.
        span: Span,
        /// The binary operator to apply.
        op: BinaryOp,
        /// The left-hand operand.
        left: Box<Expr>,
        /// The right-hand operand.
        right: Box<Expr>,
    },
    /// Unary operation applied to a single expression.
    UnaryOp {
        /// Span of the expression.
        span: Span,
        /// The unary operator to apply.
        op: UnaryOp,
        /// The operand expression.
        expr: Box<Expr>,
    },
    /// Function call with a name and argument list.
    FunctionCall {
        /// Span of the entire call expression.
        span: Span,
        /// Span of the function name itself.
        name_span: Span,
        /// The name of the function to call.
        name: FunctionName,
        /// The arguments to pass to the function.
        args: Vec<Expr>,
    },
    /// Variable reference (local, parameter, or external).
    Variable {
        /// Span of the entire variable expression.
        span: Span,
        /// The resolved variable.
        variable: Variable,
    },
    /// Constant literal value.
    Literal {
        /// Span of the literal.
        span: Span,
        /// The literal value.
        value: Literal,
    },
}

impl Expr {
    /// Creates a comparison operation expression.
    #[must_use]
    pub fn comparison_op(
        span: Span,
        op: ComparisonOp,
        left: Self,
        right: Self,
        rest_chained: Vec<(ComparisonOp, Self)>,
    ) -> Self {
        Self::ComparisonOp {
            span,
            op,
            left: Box::new(left),
            right: Box::new(right),
            rest_chained,
        }
    }

    /// Creates a binary operation expression.
    #[must_use]
    pub fn binary_op(span: Span, op: BinaryOp, left: Self, right: Self) -> Self {
        Self::BinaryOp {
            span,
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// Creates a unary operation expression.
    #[must_use]
    pub fn unary_op(span: Span, op: UnaryOp, expr: Self) -> Self {
        Self::UnaryOp {
            span,
            op,
            expr: Box::new(expr),
        }
    }

    /// Creates a function call expression.
    #[must_use]
    pub const fn function_call(
        span: Span,
        name_span: Span,
        name: FunctionName,
        args: Vec<Self>,
    ) -> Self {
        Self::FunctionCall {
            span,
            name_span,
            name,
            args,
        }
    }

    /// Creates a built-in variable reference.
    #[must_use]
    pub const fn builtin_variable(span: Span, ident_span: Span, ident: Identifier) -> Self {
        Self::Variable {
            span,
            variable: Variable::builtin(ident, ident_span),
        }
    }

    /// Creates a parameter variable reference.
    #[must_use]
    pub const fn parameter_variable(
        span: Span,
        parameter_span: Span,
        parameter_name: ParameterName,
    ) -> Self {
        Self::Variable {
            span,
            variable: Variable::parameter(parameter_name, parameter_span),
        }
    }

    /// Creates an external variable reference.
    #[must_use]
    pub const fn external_variable(
        span: Span,
        model: ModelPath,
        model_span: Span,
        parameter_name: ParameterName,
        parameter_span: Span,
    ) -> Self {
        Self::Variable {
            span,
            variable: Variable::external(model, model_span, parameter_name, parameter_span),
        }
    }

    /// Creates a literal expression.
    #[must_use]
    pub const fn literal(span: Span, value: Literal) -> Self {
        Self::Literal { span, value }
    }

    /// Returns the span of this expression.
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Self::ComparisonOp { span, .. }
            | Self::BinaryOp { span, .. }
            | Self::UnaryOp { span, .. }
            | Self::FunctionCall { span, .. }
            | Self::Variable { span, .. }
            | Self::Literal { span, .. } => *span,
        }
    }

    /// Returns the span of the function name if this is a function call.
    #[must_use]
    pub const fn function_name_span(&self) -> Option<Span> {
        match self {
            Self::FunctionCall { name_span, .. } => Some(*name_span),
            _ => None,
        }
    }
}

/// Binary operators for mathematical and logical operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    /// Addition: `a + b`
    Add,
    /// Subtraction: `a - b`
    Sub,
    /// True subtraction: `a -- b`
    TrueSub,
    /// Multiplication: `a * b`
    Mul,
    /// Division: `a / b`
    Div,
    /// True division: `a // b`
    TrueDiv,
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
    Builtin {
        /// The identifier of the builtin.
        ident: Identifier,
        /// Span of the builtin identifier.
        ident_span: Span,
    },
    /// Parameter defined in the current model.
    Parameter {
        /// The parameter name.
        parameter_name: ParameterName,
        /// Span of the parameter identifier.
        parameter_span: Span,
    },
    /// Parameter defined in another model.
    External {
        /// The model where the parameter is defined.
        model: ModelPath,
        /// Span of the referenced model identifier.
        model_span: Span,
        /// The identifier of the parameter in that model.
        parameter_name: ParameterName,
        /// Span of the parameter identifier.
        parameter_span: Span,
    },
}

impl Variable {
    /// Creates a built-in variable reference.
    #[must_use]
    pub const fn builtin(ident: Identifier, ident_span: Span) -> Self {
        Self::Builtin { ident, ident_span }
    }

    /// Creates a parameter variable reference.
    #[must_use]
    pub const fn parameter(parameter_name: ParameterName, parameter_span: Span) -> Self {
        Self::Parameter {
            parameter_name,
            parameter_span,
        }
    }

    /// Creates an external variable reference.
    #[must_use]
    pub const fn external(
        model: ModelPath,
        model_span: Span,
        parameter_name: ParameterName,
        parameter_span: Span,
    ) -> Self {
        Self::External {
            model,
            model_span,
            parameter_name,
            parameter_span,
        }
    }

    /// Returns the span of the referenced parameter identifier.
    #[must_use]
    pub const fn parameter_span(&self) -> Span {
        match self {
            Self::Builtin { ident_span, .. } => *ident_span,
            Self::Parameter { parameter_span, .. } => *parameter_span,
            Self::External { parameter_span, .. } => *parameter_span,
        }
    }

    /// Returns the span of the referenced model identifier, if any.
    #[must_use]
    pub const fn model_span(&self) -> Option<Span> {
        match self {
            Self::External { model_span, .. } => Some(*model_span),
            _ => None,
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
