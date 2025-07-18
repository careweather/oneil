//! Declaration constructs for the AST
//!
//! This module contains structures for representing declarations in Oneil programs,
//! including imports, model usage, parameters, and tests.

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
    /// Import declaration for including other modules
    Import(ImportNode),

    /// Model usage declaration for referencing other models
    UseModel(UseModelNode),

    /// Parameter declaration for defining model parameters
    Parameter(ParameterNode),
    /// Test declaration for verifying model behavior
    Test(TestNode),
}

/// A node containing a declaration
pub type DeclNode = Node<Decl>;

impl Decl {
    /// Creates an import declaration
    pub fn import(path: ImportNode) -> Self {
        Self::Import(path)
    }

    /// Creates a model usage declaration
    pub fn use_model(use_model: UseModelNode) -> Self {
        Self::UseModel(use_model)
    }

    /// Creates a parameter declaration
    pub fn parameter(parameter: ParameterNode) -> Self {
        Self::Parameter(parameter)
    }

    /// Creates a test declaration
    pub fn test(test: TestNode) -> Self {
        Self::Test(test)
    }
}

/// An import declaration that specifies a module to include
#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    path: String,
}

/// A node containing an import declaration
pub type ImportNode = Node<Import>;

impl Import {
    /// Creates a new import with the given path
    pub fn new(path: String) -> Self {
        Self { path }
    }

    /// Returns the import path as a string slice
    pub fn path(&self) -> &str {
        &self.path
    }
}

/// A model usage declaration that references another model
#[derive(Debug, Clone, PartialEq)]
pub struct UseModel {
    model_name: IdentifierNode,
    subcomponents: Vec<IdentifierNode>,
    inputs: Option<ModelInputListNode>,
    alias: Option<IdentifierNode>,
}

/// A node containing a model usage declaration
pub type UseModelNode = Node<UseModel>;

impl UseModel {
    /// Creates a new model usage declaration
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

    /// Returns the name of the model being used
    pub fn model_name(&self) -> &IdentifierNode {
        &self.model_name
    }

    /// Returns the list of subcomponents being used
    pub fn subcomponents(&self) -> &[IdentifierNode] {
        &self.subcomponents
    }

    /// Returns the optional inputs for the model usage
    pub fn inputs(&self) -> Option<&ModelInputListNode> {
        self.inputs.as_ref()
    }

    /// Returns the optional alias for the model usage
    pub fn alias(&self) -> Option<&IdentifierNode> {
        self.alias.as_ref()
    }
}

/// A list of model inputs for a model usage declaration
#[derive(Debug, Clone, PartialEq)]
pub struct ModelInputList(Vec<ModelInputNode>);

/// A node containing a list of model inputs
pub type ModelInputListNode = Node<ModelInputList>;

impl ModelInputList {
    /// Creates a new model input list
    pub fn new(inputs: Vec<ModelInputNode>) -> Self {
        Self(inputs)
    }

    /// Returns a slice of the model inputs
    pub fn inputs(&self) -> &[ModelInputNode] {
        &self.0
    }
}

/// A single model input with an identifier and value
#[derive(Debug, Clone, PartialEq)]
pub struct ModelInput {
    ident: IdentifierNode,
    value: ExprNode,
}

/// A node containing a model input
pub type ModelInputNode = Node<ModelInput>;

impl ModelInput {
    /// Creates a new model input
    pub fn new(ident: IdentifierNode, value: ExprNode) -> Self {
        Self { ident, value }
    }

    /// Returns the identifier of the model input
    pub fn ident(&self) -> &IdentifierNode {
        &self.ident
    }

    /// Returns the value expression of the model input
    pub fn value(&self) -> &ExprNode {
        &self.value
    }
}
