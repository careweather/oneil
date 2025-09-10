//! Constructors and builders for the AST for testing. The main purpose of this
//! module is to remove the need to deal with spans.
//!
//! This module treats spans as unimportant, and therefore should not be used
//! when testing span related functionality.
//!
//! If the span is important, it is better to construct the node directly.

use oneil_ast::{self as ast, AstSpan};

/// Generates a span for testing purposes
///
/// The span is intentionally random in order to discourage any
/// use of the spans for testing.
fn unimportant_span() -> AstSpan {
    use rand::Rng;
    let mut rng = rand::rng();

    let start = usize::from(rng.random::<u16>());
    let length = usize::from(rng.random::<u16>());
    let whitespace_length = usize::from(rng.random::<u16>());

    AstSpan::new(start, length, whitespace_length)
}

// SIMPLE CONSTRUCTORS

pub fn empty_model_node() -> ast::ModelNode {
    let model = ast::Model::new(None, vec![], vec![]);
    ast::Node::new(&unimportant_span(), model)
}

pub fn identifier_node(identifier: &'static str) -> ast::Node<ast::Identifier> {
    let identifier = ast::Identifier::new(identifier.to_string());
    ast::Node::new(&unimportant_span(), identifier)
}

pub fn directory_name_node(directory_name: &'static str) -> ast::Node<ast::Directory> {
    let directory = ast::Directory::Name(directory_name.to_string());
    ast::Node::new(&unimportant_span(), directory)
}

pub fn literal_number_node(number: f64) -> ast::Node<ast::Literal> {
    let literal = ast::Literal::Number(number);
    ast::Node::new(&unimportant_span(), literal)
}

pub fn literal_string_node(string: &'static str) -> ast::Node<ast::Literal> {
    let literal = ast::Literal::String(string.to_string());
    ast::Node::new(&unimportant_span(), literal)
}

pub fn literal_boolean_node(boolean: bool) -> ast::Node<ast::Literal> {
    let literal = ast::Literal::Boolean(boolean);
    ast::Node::new(&unimportant_span(), literal)
}

pub fn literal_number_expr_node(number: f64) -> ast::Node<ast::Expr> {
    let literal_node = literal_number_node(number);
    let expr = ast::Expr::Literal(literal_node);
    ast::Node::new(&unimportant_span(), expr)
}

pub fn literal_string_expr_node(string: &'static str) -> ast::Node<ast::Expr> {
    let literal_node = literal_string_node(string);
    let expr = ast::Expr::Literal(literal_node);
    ast::Node::new(&unimportant_span(), expr)
}

pub fn literal_boolean_expr_node(boolean: bool) -> ast::Node<ast::Expr> {
    let literal_node = literal_boolean_node(boolean);
    let expr = ast::Expr::Literal(literal_node);
    ast::Node::new(&unimportant_span(), expr)
}

pub fn binary_op_node(op: ast::BinaryOp) -> ast::BinaryOpNode {
    ast::Node::new(&unimportant_span(), op)
}

pub fn binary_op_expr_node(
    op: ast::BinaryOpNode,
    left: ast::ExprNode,
    right: ast::ExprNode,
) -> ast::Node<ast::Expr> {
    let expr = ast::Expr::binary_op(op, left, right);
    ast::Node::new(&unimportant_span(), expr)
}

pub fn unary_op_node(op: ast::UnaryOp) -> ast::UnaryOpNode {
    ast::Node::new(&unimportant_span(), op)
}

pub fn unary_op_expr_node(op: ast::UnaryOpNode, expr: ast::ExprNode) -> ast::Node<ast::Expr> {
    let expr = ast::Expr::unary_op(op, expr);
    ast::Node::new(&unimportant_span(), expr)
}

pub fn function_call_expr_node(
    name: ast::IdentifierNode,
    args: Vec<ast::ExprNode>,
) -> ast::Node<ast::Expr> {
    let expr = ast::Expr::function_call(name, args);
    ast::Node::new(&unimportant_span(), expr)
}

pub fn identifier_variable_node(name: &'static str) -> ast::Node<ast::Variable> {
    let identifier = ast::Identifier::new(name.to_string());
    let identifier_node = ast::Node::new(&unimportant_span(), identifier);
    let variable = ast::Variable::Identifier(identifier_node);
    ast::Node::new(&unimportant_span(), variable)
}

pub fn model_parameter_variable_node(
    reference_model: &'static str,
    parameter: &'static str,
) -> ast::Node<ast::Variable> {
    let reference_model_node = identifier_node(reference_model);
    let parameter_node = identifier_node(parameter);
    let variable = ast::Variable::ModelParameter {
        reference_model: reference_model_node,
        parameter: parameter_node,
    };
    ast::Node::new(&unimportant_span(), variable)
}

pub fn variable_expr_node(variable_node: ast::VariableNode) -> ast::Node<ast::Expr> {
    let expr = ast::Expr::Variable(variable_node);
    ast::Node::new(&unimportant_span(), expr)
}

pub fn parenthesized_expr_node(expr: ast::ExprNode) -> ast::Node<ast::Expr> {
    let expr = ast::Expr::Parenthesized { expr };
    ast::Node::new(&unimportant_span(), expr)
}

pub fn comparison_op_node(op: ast::ComparisonOp) -> ast::ComparisonOpNode {
    ast::Node::new(&unimportant_span(), op)
}

pub fn comparison_op_expr_node(
    op: ast::ComparisonOpNode,
    left: ast::ExprNode,
    right: ast::ExprNode,
    rest_chained: impl IntoIterator<Item = (ast::ComparisonOpNode, ast::ExprNode)>,
) -> ast::Node<ast::Expr> {
    let expr = ast::Expr::ComparisonOp {
        op,
        left,
        right,
        rest_chained: rest_chained.into_iter().collect(),
    };
    ast::Node::new(&unimportant_span(), expr)
}

pub fn simple_parameter_value_node(expr: ast::ExprNode) -> ast::Node<ast::ParameterValue> {
    let value = ast::ParameterValue::Simple(expr, None);
    ast::Node::new(&unimportant_span(), value)
}

pub fn continuous_limits_node(min: f64, max: f64) -> ast::Node<ast::Limits> {
    let min_node = literal_number_expr_node(min);
    let max_node = literal_number_expr_node(max);
    let limits = ast::Limits::Continuous {
        min: min_node,
        max: max_node,
    };
    ast::Node::new(&unimportant_span(), limits)
}

pub fn discrete_limits_node(values: impl IntoIterator<Item = f64>) -> ast::Node<ast::Limits> {
    let values = values.into_iter().map(literal_number_expr_node).collect();
    let limits = ast::Limits::Discrete { values };
    ast::Node::new(&unimportant_span(), limits)
}

pub fn unit_node(identifier: &'static str) -> ast::Node<ast::UnitExpr> {
    let identifier = identifier_node(identifier);
    let unit = ast::UnitExpr::Unit {
        identifier,
        exponent: None,
    };
    ast::Node::new(&unimportant_span(), unit)
}

pub fn unit_with_exponent_node(
    identifier: &'static str,
    exponent: f64,
) -> ast::Node<ast::UnitExpr> {
    let identifier = identifier_node(identifier);
    let exponent = ast::UnitExponent::new(exponent);
    let exponent_node = ast::Node::new(&unimportant_span(), exponent);
    let unit = ast::UnitExpr::Unit {
        identifier,
        exponent: Some(exponent_node),
    };
    ast::Node::new(&unimportant_span(), unit)
}

pub fn unit_binary_op_node(
    op: ast::UnitOp,
    left: ast::UnitExprNode,
    right: ast::UnitExprNode,
) -> ast::Node<ast::UnitExpr> {
    let op_node = ast::Node::new(&unimportant_span(), op);
    let unit = ast::UnitExpr::BinaryOp {
        op: op_node,
        left,
        right,
    };
    ast::Node::new(&unimportant_span(), unit)
}

pub fn parenthesized_unit_node(expr: ast::UnitExprNode) -> ast::Node<ast::UnitExpr> {
    let unit = ast::UnitExpr::Parenthesized { expr };
    ast::Node::new(&unimportant_span(), unit)
}

pub fn unit_one_node() -> ast::Node<ast::UnitExpr> {
    let unit = ast::UnitExpr::UnitOne;
    ast::Node::new(&unimportant_span(), unit)
}

// BUILDERS

pub struct ModelNodeBuilder {
    note: Option<ast::NoteNode>,
    decls: Vec<ast::DeclNode>,
    sections: Vec<ast::SectionNode>,
}

impl ModelNodeBuilder {
    pub fn new() -> Self {
        Self {
            note: None,
            decls: vec![],
            sections: vec![],
        }
    }

    pub fn with_submodel(mut self, submodel: &'static str) -> Self {
        let use_model_name = ast::Identifier::new(submodel.to_string());
        let use_model_name_node = ast::Node::new(&unimportant_span(), use_model_name);
        let use_model_info = ast::ModelInfo::new(use_model_name_node, vec![], None);
        let use_model_info_node = ast::Node::new(&unimportant_span(), use_model_info);
        let use_model =
            ast::UseModel::new(vec![], use_model_info_node, None, ast::ModelKind::Submodel);
        let use_model_node = ast::Node::new(&unimportant_span(), use_model);
        let decl = ast::Decl::use_model(use_model_node);
        let decl_node = ast::Node::new(&unimportant_span(), decl);

        self.decls.push(decl_node);
        self
    }

    pub fn with_reference(mut self, reference: &'static str) -> Self {
        let reference_name = ast::Identifier::new(reference.to_string());
        let reference_name_node = ast::Node::new(&unimportant_span(), reference_name);
        let reference_info = ast::ModelInfo::new(reference_name_node, vec![], None);
        let reference_info_node = ast::Node::new(&unimportant_span(), reference_info);
        let reference =
            ast::UseModel::new(vec![], reference_info_node, None, ast::ModelKind::Reference);
        let reference_node = ast::Node::new(&unimportant_span(), reference);
        let decl = ast::Decl::use_model(reference_node);
        let decl_node = ast::Node::new(&unimportant_span(), decl);

        self.decls.push(decl_node);
        self
    }

    pub fn with_section(mut self, section: &'static str, decls: Vec<ast::DeclNode>) -> Self {
        let section_label = ast::Label::new(section.to_string());
        let section_label_node = ast::Node::new(&unimportant_span(), section_label);
        let section_header = ast::SectionHeader::new(section_label_node);
        let section_header_node = ast::Node::new(&unimportant_span(), section_header);

        let section = ast::Section::new(section_header_node, None, decls);
        let section_node = ast::Node::new(&unimportant_span(), section);

        self.sections.push(section_node);
        self
    }

    pub fn build(self) -> ast::ModelNode {
        let model = ast::Model::new(self.note, self.decls, self.sections);
        ast::Node::new(&unimportant_span(), model)
    }
}

pub struct ImportModelNodeBuilder {
    directory_path: Vec<ast::DirectoryNode>,
    model_info: ModelInfoNodeBuilder,
    submodel_list: Option<Vec<ast::ModelInfoNode>>,
    kind: Option<ast::ModelKind>,
}

impl ImportModelNodeBuilder {
    pub fn new() -> Self {
        Self {
            directory_path: vec![],
            model_info: ModelInfoNodeBuilder::new(),
            submodel_list: None,
            kind: None,
        }
    }

    pub fn with_directory_path(
        mut self,
        directory_path: impl IntoIterator<Item = &'static str>,
    ) -> Self {
        self.directory_path = directory_path
            .into_iter()
            .map(directory_name_node)
            .collect();
        self
    }

    pub fn with_top_component(mut self, name: &'static str) -> Self {
        self.model_info = self.model_info.with_top_component(name);
        self
    }

    pub fn with_alias(mut self, alias: &'static str) -> Self {
        self.model_info = self.model_info.with_alias(alias);
        self
    }

    pub fn with_subcomponents(
        mut self,
        subcomponents: impl IntoIterator<Item = &'static str>,
    ) -> Self {
        self.model_info = self.model_info.with_subcomponents(subcomponents);
        self
    }

    pub fn with_submodels(
        mut self,
        submodels: impl IntoIterator<Item = ast::ModelInfoNode>,
    ) -> Self {
        self.submodel_list = Some(submodels.into_iter().collect());
        self
    }

    pub fn with_kind(mut self, kind: ast::ModelKind) -> Self {
        self.kind = Some(kind);
        self
    }

    pub fn build(self) -> ast::UseModelNode {
        let model_info = self.model_info.build();
        let kind = self.kind.expect("kind is required");
        let submodel_list = self.submodel_list.map(|submodel_list| {
            ast::Node::new(&unimportant_span(), ast::SubmodelList::new(submodel_list))
        });

        let use_model = ast::UseModel::new(self.directory_path, model_info, submodel_list, kind);

        ast::Node::new(&unimportant_span(), use_model)
    }

    pub fn build_as_decl_node(self) -> ast::DeclNode {
        let use_model = self.build();
        let decl = ast::Decl::use_model(use_model);
        ast::Node::new(&unimportant_span(), decl)
    }
}

pub struct ModelInfoNodeBuilder {
    top_component: Option<ast::IdentifierNode>,
    subcomponents: Vec<ast::IdentifierNode>,
    alias: Option<ast::IdentifierNode>,
}

impl ModelInfoNodeBuilder {
    pub fn new() -> Self {
        Self {
            top_component: None,
            alias: None,
            subcomponents: vec![],
        }
    }

    pub fn with_top_component(mut self, name: &'static str) -> Self {
        let name_identifier = identifier_node(name);
        self.top_component = Some(name_identifier);
        self
    }

    pub fn with_alias(mut self, alias: &'static str) -> Self {
        let alias_identifier = identifier_node(alias);
        self.alias = Some(alias_identifier);
        self
    }

    pub fn with_subcomponents(
        mut self,
        subcomponents: impl IntoIterator<Item = &'static str>,
    ) -> Self {
        let subcomponents = subcomponents.into_iter().map(identifier_node).collect();
        self.subcomponents = subcomponents;
        self
    }

    pub fn build(self) -> ast::ModelInfoNode {
        let model_info = ast::ModelInfo::new(
            self.top_component.expect("name is required"),
            self.subcomponents,
            self.alias,
        );

        ast::Node::new(&unimportant_span(), model_info)
    }
}

pub struct ImportPythonNodeBuilder;

impl ImportPythonNodeBuilder {
    pub fn build(path: &'static str) -> ast::ImportNode {
        let path = ast::Node::new(&unimportant_span(), path.to_string());
        let import = ast::Import::new(path);
        ast::Node::new(&unimportant_span(), import)
    }
}

pub struct ParameterNodeBuilder {
    label: Option<ast::LabelNode>,
    ident: Option<ast::IdentifierNode>,
    value: Option<ast::ParameterValueNode>,
    limits: Option<ast::LimitsNode>,
    performance_marker: Option<ast::PerformanceMarkerNode>,
    trace_level: Option<ast::TraceLevelNode>,
    note: Option<ast::NoteNode>,
}

impl ParameterNodeBuilder {
    pub fn new() -> Self {
        Self {
            label: None,
            ident: None,
            value: None,
            limits: None,
            performance_marker: None,
            trace_level: None,
            note: None,
        }
    }

    pub fn with_ident_and_label(mut self, ident_and_label: &'static str) -> Self {
        let ident = ast::Identifier::new(ident_and_label.to_string());
        let ident_node = ast::Node::new(&unimportant_span(), ident);
        self.ident = Some(ident_node);

        let label = ast::Label::new(ident_and_label.to_string());
        let label_node = ast::Node::new(&unimportant_span(), label);
        self.label = Some(label_node);

        self
    }

    pub fn with_number_value(mut self, value: f64) -> Self {
        let number_node = literal_number_expr_node(value);
        let value = ast::ParameterValue::Simple(number_node, None);
        let value_node = ast::Node::new(&unimportant_span(), value);

        self.value = Some(value_node);
        self
    }

    pub fn with_piecewise_values(
        mut self,
        values_and_conds: impl IntoIterator<Item = (ast::ExprNode, ast::ExprNode)>,
    ) -> Self {
        let values = values_and_conds
            .into_iter()
            .map(|(expr_node, if_expr_node)| ast::PiecewisePart::new(expr_node, if_expr_node))
            .map(|piecewise_part| ast::Node::new(&unimportant_span(), piecewise_part))
            .collect();

        let value = ast::ParameterValue::Piecewise(values, None);
        let value_node = ast::Node::new(&unimportant_span(), value);
        self.value = Some(value_node);

        self
    }

    pub fn with_dependent_parameter_values(
        mut self,
        parameters: impl IntoIterator<Item = &'static str>,
    ) -> Self {
        let mut parameters = parameters.into_iter().collect::<Vec<_>>();

        let ident_a = parameters
            .pop()
            .expect("parameters must have at least one parameter");
        let ident_a_node = identifier_variable_node(ident_a);
        let ident_a_node = variable_expr_node(ident_a_node);

        let final_expr_node =
            parameters
                .into_iter()
                .fold(ident_a_node, |final_expr_node, parameter| {
                    let parameter_node = identifier_variable_node(parameter);
                    let parameter_node = variable_expr_node(parameter_node);
                    let binary_op_node = binary_op_node(ast::BinaryOp::Add);
                    binary_op_expr_node(binary_op_node, final_expr_node, parameter_node)
                });

        let value = ast::ParameterValue::Simple(final_expr_node, None);
        let value_node = ast::Node::new(&unimportant_span(), value);

        self.value = Some(value_node);
        self
    }

    pub fn with_continuous_limit_vars(
        mut self,
        min_var: &'static str,
        max_var: &'static str,
    ) -> Self {
        let min_var_node = identifier_variable_node(min_var);
        let min_var_node = variable_expr_node(min_var_node);
        let max_var_node = identifier_variable_node(max_var);
        let max_var_node = variable_expr_node(max_var_node);
        let limits = ast::Limits::Continuous {
            min: min_var_node,
            max: max_var_node,
        };
        let limits_node = ast::Node::new(&unimportant_span(), limits);
        self.limits = Some(limits_node);

        self
    }

    pub fn build(self) -> ast::ParameterNode {
        let label = self.label.expect("label is required");
        let ident = self.ident.expect("ident is required");
        let value = self.value.expect("value is required");
        let limits = self.limits;
        let performance_marker = self.performance_marker;
        let trace_level = self.trace_level;
        let note = self.note;

        let parameter = ast::Parameter::new(
            label,
            ident,
            value,
            limits,
            performance_marker,
            trace_level,
            note,
        );
        ast::Node::new(&unimportant_span(), parameter)
    }
}

pub struct TestNodeBuilder {
    trace_level: Option<ast::TraceLevelNode>,
    expr: Option<ast::ExprNode>,
    note: Option<ast::NoteNode>,
}

impl TestNodeBuilder {
    pub fn new() -> Self {
        Self {
            trace_level: None,
            expr: None,
            note: None,
        }
    }

    pub fn with_boolean_expr(mut self, expr: bool) -> Self {
        let expr_node = literal_boolean_expr_node(expr);
        self.expr = Some(expr_node);
        self
    }

    pub fn with_variable_expr(mut self, expr: &'static str) -> Self {
        let variable_node = identifier_variable_node(expr);
        let expr_node = variable_expr_node(variable_node);
        self.expr = Some(expr_node);
        self
    }

    pub fn with_debug_trace_level(mut self) -> Self {
        let trace_level = ast::TraceLevel::Debug;
        let trace_level_node = ast::Node::new(&unimportant_span(), trace_level);
        self.trace_level = Some(trace_level_node);
        self
    }

    pub fn build(self) -> ast::TestNode {
        let note = self.note;
        let trace_level = self.trace_level;
        let expr = self.expr.expect("expr is required");

        let test = ast::Test::new(trace_level, expr, note);
        ast::Node::new(&unimportant_span(), test)
    }
}
