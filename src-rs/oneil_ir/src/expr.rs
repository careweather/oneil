//! Expression system for mathematical and logical operations in Oneil.

use oneil_shared::{
    paths::PythonPath,
    span::Span,
    symbols::{
        BuiltinFunctionName, BuiltinValueName, ParameterName, PyFunctionName, ReferenceName,
    },
};

use crate::CompositeUnit;

/// Abstract syntax tree for mathematical and logical expressions.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum Expr {
    /// Comparison operation with left and right operands, supporting chaining.
    ComparisonOp {
        /// Span of the entire comparison expression.
        span: Span,
        /// The comparison operator to apply.
        op: ComparisonOp,
        /// The left-hand operand.
        left: Box<Self>,
        /// The right-hand operand.
        right: Box<Self>,
        /// Chained comparison operations (order matters).
        rest_chained: Vec<(ComparisonOp, Self)>,
    },
    /// Binary operation combining two expressions with an operator.
    BinaryOp {
        /// Span of the expression.
        span: Span,
        /// The binary operator to apply.
        op: BinaryOp,
        /// The left-hand operand.
        left: Box<Self>,
        /// The right-hand operand.
        right: Box<Self>,
    },
    /// Fallback expression: evaluate `left`, then use `right` if needed (`?`).
    Fallback {
        /// Span of the entire expression.
        span: Span,
        /// Operand evaluated first.
        left: Box<Self>,
        /// Operand used when the fallback applies.
        right: Box<Self>,
    },
    /// Unary operation applied to a single expression.
    UnaryOp {
        /// Span of the expression.
        span: Span,
        /// The unary operator to apply.
        op: UnaryOp,
        /// The operand expression.
        expr: Box<Self>,
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
        args: Vec<Self>,
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
    /// Unit cast expression: (expr : unit)
    UnitCast {
        /// Span of the entire unit cast expression.
        span: Span,
        /// The expression to cast.
        expr: Box<Self>,
        /// The unit to cast to.
        unit: CompositeUnit,
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

    /// Creates a fallback expression (`left ? right`).
    #[must_use]
    pub fn fallback(span: Span, left: Self, right: Self) -> Self {
        Self::Fallback {
            span,
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
    pub const fn builtin_variable(span: Span, ident_span: Span, ident: BuiltinValueName) -> Self {
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
        reference_name: ReferenceName,
        reference_span: Span,
        parameter_name: ParameterName,
        parameter_span: Span,
    ) -> Self {
        Self::Variable {
            span,
            variable: Variable::external(
                reference_name,
                reference_span,
                parameter_name,
                parameter_span,
            ),
        }
    }

    /// Creates a literal expression.
    #[must_use]
    pub const fn literal(span: Span, value: Literal) -> Self {
        Self::Literal { span, value }
    }

    /// Creates a unit cast expression.
    #[must_use]
    pub fn unit_cast(span: Span, expr: Self, unit: CompositeUnit) -> Self {
        Self::UnitCast {
            span,
            expr: Box::new(expr),
            unit,
        }
    }

    /// Walks every `Variable` in this expression in pre-order and invokes
    /// `f` on each. Read-only counterpart to [`Self::walk_variables_mut`];
    /// used by the post-walk validation pass to inspect resolved variables
    /// without mutating them.
    pub fn walk_variables<F: FnMut(&Variable)>(&self, f: &mut F) {
        match self {
            Self::ComparisonOp {
                left,
                right,
                rest_chained,
                ..
            } => {
                left.walk_variables(f);
                right.walk_variables(f);
                for (_, expr) in rest_chained {
                    expr.walk_variables(f);
                }
            }
            Self::BinaryOp { left, right, .. } | Self::Fallback { left, right, .. } => {
                left.walk_variables(f);
                right.walk_variables(f);
            }
            Self::UnaryOp { expr, .. } | Self::UnitCast { expr, .. } => {
                expr.walk_variables(f);
            }
            Self::FunctionCall { args, .. } => {
                for arg in args {
                    arg.walk_variables(f);
                }
            }
            Self::Variable { variable, .. } => f(variable),
            Self::Literal { .. } => {}
        }
    }

    /// Walks every `Variable` in this expression in pre-order and invokes
    /// `f` on each. Used by the pre-validation classification pass to
    /// reclassify raw identifiers as Parameter / Builtin / External.
    pub fn walk_variables_mut<F: FnMut(&mut Variable)>(&mut self, f: &mut F) {
        match self {
            Self::ComparisonOp {
                left,
                right,
                rest_chained,
                ..
            } => {
                left.walk_variables_mut(f);
                right.walk_variables_mut(f);
                for (_, expr) in rest_chained {
                    expr.walk_variables_mut(f);
                }
            }
            Self::BinaryOp { left, right, .. } | Self::Fallback { left, right, .. } => {
                left.walk_variables_mut(f);
                right.walk_variables_mut(f);
            }
            Self::UnaryOp { expr, .. } | Self::UnitCast { expr, .. } => {
                expr.walk_variables_mut(f);
            }
            Self::FunctionCall { args, .. } => {
                for arg in args {
                    arg.walk_variables_mut(f);
                }
            }
            Self::Variable { variable, .. } => f(variable),
            Self::Literal { .. } => {}
        }
    }

    /// Visits the expression with a visitor in pre-order
    /// (parent nodes are visited before their children).
    #[must_use]
    pub fn pre_order_visit<V: ExprVisitor>(&self, visitor: V) -> V {
        match self {
            Self::ComparisonOp {
                span,
                op,
                left,
                right,
                rest_chained,
            } => {
                let visitor = visitor.visit_comparison_op(span, op, left, right, rest_chained);
                let visitor = left.pre_order_visit(visitor);
                let visitor = right.pre_order_visit(visitor);
                rest_chained.iter().fold(visitor, |visitor, (_op, expr)| {
                    expr.pre_order_visit(visitor)
                })
            }
            Self::BinaryOp {
                span,
                op,
                left,
                right,
            } => {
                let visitor = visitor.visit_binary_op(span, op, left, right);
                let visitor = left.pre_order_visit(visitor);
                right.pre_order_visit(visitor)
            }
            Self::Fallback { span, left, right } => {
                let visitor = visitor.visit_fallback(span, left, right);
                let visitor = left.pre_order_visit(visitor);
                right.pre_order_visit(visitor)
            }
            Self::UnaryOp { span, op, expr } => {
                let visitor = visitor.visit_unary_op(span, op, expr);
                expr.pre_order_visit(visitor)
            }
            Self::FunctionCall {
                span,
                name_span,
                name,
                args,
            } => {
                let visitor = visitor.visit_function_call(span, name_span, name, args);
                args.iter()
                    .fold(visitor, |visitor, arg| arg.pre_order_visit(visitor))
            }
            Self::Variable { span, variable } => visitor.visit_variable(span, variable),
            Self::Literal { span, value } => visitor.visit_literal(span, value),
            Self::UnitCast { span, expr, unit } => {
                let visitor = visitor.visit_unit_cast(span, expr, unit);
                expr.pre_order_visit(visitor)
            }
        }
    }

    /// Visits the expression with a visitor in post-order
    /// (parent nodes are visited after their children).
    #[must_use]
    pub fn post_order_visit<V: ExprVisitor>(&self, visitor: V) -> V {
        match self {
            Self::ComparisonOp {
                span,
                op,
                left,
                right,
                rest_chained,
            } => {
                let visitor = left.post_order_visit(visitor);
                let visitor = right.post_order_visit(visitor);
                let visitor = rest_chained.iter().fold(visitor, |visitor, (_op, expr)| {
                    expr.post_order_visit(visitor)
                });
                visitor.visit_comparison_op(span, op, left, right, rest_chained)
            }
            Self::BinaryOp {
                span,
                op,
                left,
                right,
            } => {
                let visitor = left.post_order_visit(visitor);
                let visitor = right.post_order_visit(visitor);
                visitor.visit_binary_op(span, op, left, right)
            }
            Self::Fallback { span, left, right } => {
                let visitor = left.post_order_visit(visitor);
                let visitor = right.post_order_visit(visitor);
                visitor.visit_fallback(span, left, right)
            }
            Self::UnaryOp { span, op, expr } => {
                let visitor = expr.post_order_visit(visitor);
                visitor.visit_unary_op(span, op, expr)
            }
            Self::FunctionCall {
                span,
                name_span,
                name,
                args,
            } => {
                let visitor = args
                    .iter()
                    .fold(visitor, |visitor, arg| arg.post_order_visit(visitor));

                visitor.visit_function_call(span, name_span, name, args)
            }
            Self::Variable { span, variable } => visitor.visit_variable(span, variable),
            Self::Literal { span, value } => visitor.visit_literal(span, value),
            Self::UnitCast { span, expr, unit } => {
                let visitor = expr.post_order_visit(visitor);
                visitor.visit_unit_cast(span, expr, unit)
            }
        }
    }
}

/// Binary operators for mathematical and logical operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BinaryOp {
    /// Addition: `a + b`
    Add,
    /// Subtraction: `a - b`
    Sub,
    /// Escaped subtraction: `a -- b`
    EscapedSub,
    /// Multiplication: `a * b`
    Mul,
    /// Division: `a / b`
    Div,
    /// Escaped division: `a // b`
    EscapedDiv,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnaryOp {
    /// Negation: `-a`
    Neg,
    /// Logical NOT: `!a`
    Not,
}

/// Function names for built-in and imported functions.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum FunctionName {
    /// Built-in mathematical function.
    Builtin(BuiltinFunctionName, Span),
    /// Function imported from a Python module.
    Imported {
        /// The path to the Python module.
        python_path: PythonPath,
        /// The name of the function.
        name: PyFunctionName,
        /// The span of the function name.
        name_span: Span,
    },
}

impl FunctionName {
    /// Creates a reference to a built-in function.
    #[must_use]
    pub const fn builtin(name: BuiltinFunctionName, name_span: Span) -> Self {
        Self::Builtin(name, name_span)
    }

    /// Creates a reference to an imported Python function.
    #[must_use]
    pub const fn imported(python_path: PythonPath, name: PyFunctionName, name_span: Span) -> Self {
        Self::Imported {
            python_path,
            name,
            name_span,
        }
    }
}

/// Variable references in expressions.
///
/// Variables carry no positional information. The pre-validation
/// classification pass classifies a raw identifier as `Parameter`,
/// `External`, or `Builtin` based on the host instance's binding
/// scope; eval resolves variables against the active scope at force
/// time (with overlay parameters pushing the anchor scope first via
/// [`crate::DesignProvenance::anchor_path`]).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum Variable {
    /// Built-in variable.
    Builtin {
        /// The identifier of the builtin.
        ident: BuiltinValueName,
        /// Span of the builtin identifier.
        ident_span: Span,
    },
    /// Bare-name parameter reference, evaluated against the active scope.
    Parameter {
        /// The parameter name.
        parameter_name: ParameterName,
        /// Span of the parameter identifier.
        parameter_span: Span,
    },
    /// `p.r` reference: parameter `p` on the instance reached through
    /// reference name `r` from the active scope. The source-text spelling
    /// `p.r` is subscript-style (parameter first, reference / submodel
    /// second), opposite of OOP-style `r.p`.
    ///
    /// The resolved model path is **not** stored here; it is looked up from
    /// the live `InstancedModel::references` / `submodels` / `aliases` maps
    /// during the post-build validation pass (and at eval time for
    /// the lookup itself).
    External {
        /// The reference name of the model as it appears in the source.
        reference_name: ReferenceName,
        /// Span of the referenced model identifier.
        reference_span: Span,
        /// The identifier of the parameter in that model.
        parameter_name: ParameterName,
        /// Span of the parameter identifier.
        parameter_span: Span,
    },
}

impl Variable {
    /// Creates a built-in variable reference.
    #[must_use]
    pub const fn builtin(ident: BuiltinValueName, ident_span: Span) -> Self {
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
        reference_name: ReferenceName,
        reference_span: Span,
        parameter_name: ParameterName,
        parameter_span: Span,
    ) -> Self {
        Self::External {
            reference_name,
            reference_span,
            parameter_name,
            parameter_span,
        }
    }
}

/// Literal values that can appear in expressions.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
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

#[expect(
    unused_variables,
    reason = "the default implementations ignore node data"
)]
/// Visitor trait for traversing and transforming expressions.
pub trait ExprVisitor: Sized {
    /// Visits a comparison operation expression.
    #[must_use]
    fn visit_comparison_op(
        self,
        span: &Span,
        op: &ComparisonOp,
        left: &Expr,
        right: &Expr,
        rest_chained: &[(ComparisonOp, Expr)],
    ) -> Self {
        self
    }

    /// Visits a binary operation expression.
    #[must_use]
    fn visit_binary_op(self, span: &Span, op: &BinaryOp, left: &Expr, right: &Expr) -> Self {
        self
    }

    /// Visits a fallback expression (`left ? right`).
    #[must_use]
    fn visit_fallback(self, span: &Span, left: &Expr, right: &Expr) -> Self {
        self
    }

    /// Visits a unary operation expression.
    #[must_use]
    fn visit_unary_op(self, span: &Span, op: &UnaryOp, expr: &Expr) -> Self {
        self
    }

    /// Visits a function call expression.
    #[must_use]
    fn visit_function_call(
        self,
        span: &Span,
        name_span: &Span,
        name: &FunctionName,
        args: &[Expr],
    ) -> Self {
        self
    }

    /// Visits a variable reference expression.
    #[must_use]
    fn visit_variable(self, span: &Span, variable: &Variable) -> Self {
        self
    }

    /// Visits a literal value expression.
    #[must_use]
    fn visit_literal(self, span: &Span, value: &Literal) -> Self {
        self
    }

    /// Visits a unit cast expression.
    #[must_use]
    fn visit_unit_cast(self, span: &Span, expr: &Expr, unit: &CompositeUnit) -> Self {
        self
    }
}
