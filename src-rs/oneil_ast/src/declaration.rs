use crate::{
    expression::ExprNode, naming::IdentifierNode, node::Node, parameter::ParameterNode,
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

pub type DeclNode = Node<Decl>;

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

#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    path: String,
}

pub type ImportNode = Node<Import>;

impl Import {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UseModel {
    model_name: IdentifierNode,
    subcomponents: Vec<IdentifierNode>,
    inputs: Option<Vec<ModelInputNode>>,
    as_name: Option<IdentifierNode>,
}

pub type UseModelNode = Node<UseModel>;

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

    pub fn model_name(&self) -> &IdentifierNode {
        &self.model_name
    }

    pub fn subcomponents(&self) -> &[IdentifierNode] {
        &self.subcomponents
    }

    pub fn inputs(&self) -> Option<&[ModelInputNode]> {
        self.inputs.as_deref()
    }

    pub fn as_name(&self) -> Option<&IdentifierNode> {
        self.as_name.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelInput {
    name: IdentifierNode,
    value: ExprNode,
}

pub type ModelInputNode = Node<ModelInput>;

impl ModelInput {
    pub fn new(name: IdentifierNode, value: ExprNode) -> Self {
        Self { name, value }
    }

    pub fn name(&self) -> &IdentifierNode {
        &self.name
    }

    pub fn value(&self) -> &ExprNode {
        &self.value
    }
}
