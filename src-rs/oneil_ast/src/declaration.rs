use crate::{
    atom::IdentifierNode, expression::ExprNode, node::Node, parameter::ParameterNode,
    test::TestNode,
};

/// A declaration in an Oneil program
///
/// Declarations are top-level constructs that define imports, model usage,
/// parameters, and tests.
#[derive(Debug, Clone, PartialEq)]
pub enum Decl {
    Import(ImportNode),

    UseModel(UseModelNode),

    Parameter(ParameterNode),
    Test(TestNode),
}

impl Decl {
    pub fn import(path: ImportNode) -> Self {
        Self::Import(path)
    }

    pub fn use_model(use_model: UseModelNode) -> Self {
        Self::UseModel(use_model)
    }

    pub fn parameter(parameter: ParameterNode) -> Self {
        Self::Parameter(parameter)
    }

    pub fn test(test: TestNode) -> Self {
        Self::Test(test)
    }
}

pub type DeclNode = Node<Decl>;

#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    pub path: String,
}

impl Import {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

pub type ImportNode = Node<Import>;

#[derive(Debug, Clone, PartialEq)]
pub struct UseModel {
    pub model_name: IdentifierNode,
    pub subcomponents: Vec<IdentifierNode>,
    pub inputs: Option<Vec<ModelInputNode>>,
    pub as_name: Option<IdentifierNode>,
}

impl UseModel {
    pub fn new(
        model_name: IdentifierNode,
        subcomponents: Vec<IdentifierNode>,
        inputs: Option<Vec<ModelInputNode>>,
        as_name: Option<IdentifierNode>,
    ) -> Self {
        Self {
            model_name,
            subcomponents,
            inputs,
            as_name,
        }
    }
}

pub type UseModelNode = Node<UseModel>;

#[derive(Debug, Clone, PartialEq)]
pub struct ModelInput {
    pub name: IdentifierNode,
    pub value: ExprNode,
}

impl ModelInput {
    pub fn new(name: IdentifierNode, value: ExprNode) -> Self {
        Self { name, value }
    }
}

pub type ModelInputNode = Node<ModelInput>;
