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
    inputs: Option<ModelInputListNode>,
    alias: Option<IdentifierNode>,
}

pub type UseModelNode = Node<UseModel>;

impl UseModel {
    pub fn new(
        model_name: IdentifierNode,
        subcomponents: Vec<IdentifierNode>,
        inputs: Option<ModelInputListNode>,
        alias: Option<IdentifierNode>,
    ) -> Self {
        Self {
            model_name,
            subcomponents,
            inputs,
            alias,
        }
    }

    pub fn model_name(&self) -> &IdentifierNode {
        &self.model_name
    }

    pub fn subcomponents(&self) -> &[IdentifierNode] {
        &self.subcomponents
    }

    pub fn inputs(&self) -> Option<&ModelInputListNode> {
        self.inputs.as_ref()
    }

    pub fn alias(&self) -> Option<&IdentifierNode> {
        self.alias.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelInputList(Vec<ModelInputNode>);

pub type ModelInputListNode = Node<ModelInputList>;

impl ModelInputList {
    pub fn new(inputs: Vec<ModelInputNode>) -> Self {
        Self(inputs)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelInput {
    ident: IdentifierNode,
    value: ExprNode,
}

pub type ModelInputNode = Node<ModelInput>;

impl ModelInput {
    pub fn new(ident: IdentifierNode, value: ExprNode) -> Self {
        Self { ident, value }
    }

    pub fn ident(&self) -> &IdentifierNode {
        &self.ident
    }

    pub fn value(&self) -> &ExprNode {
        &self.value
    }
}
