//! Declaration constructs for the AST
//!
//! This module contains structures for representing declarations in Oneil programs,
//! including imports, model usage, parameters, and tests.

use crate::{naming::IdentifierNode, node::Node, parameter::ParameterNode, test::TestNode};

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
    /// (boxed because of large size of `ParameterNode`)
    Parameter(Box<ParameterNode>),
    /// Test declaration for verifying model behavior
    /// (boxed because of large size of `TestNode`)
    Test(Box<TestNode>),
}

/// A node containing a declaration
pub type DeclNode = Node<Decl>;

impl Decl {
    /// Creates an import declaration
    #[must_use]
    pub const fn import(path: ImportNode) -> Self {
        Self::Import(path)
    }

    /// Creates a model usage declaration
    #[must_use]
    pub const fn use_model(use_model: UseModelNode) -> Self {
        Self::UseModel(use_model)
    }

    /// Creates a parameter declaration
    #[must_use]
    pub fn parameter(parameter: ParameterNode) -> Self {
        Self::Parameter(Box::new(parameter))
    }

    /// Creates a test declaration
    #[must_use]
    pub fn test(test: TestNode) -> Self {
        Self::Test(Box::new(test))
    }
}

/// An import declaration that specifies a module to include
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Import {
    path: Node<String>,
}

/// A node containing an import declaration
pub type ImportNode = Node<Import>;

impl Import {
    /// Creates a new import with the given path
    #[must_use]
    pub const fn new(path: Node<String>) -> Self {
        Self { path }
    }

    /// Returns the import path as a string slice
    #[must_use]
    pub const fn path(&self) -> &Node<String> {
        &self.path
    }
}

/// A model usage declaration that references another model
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseModel {
    model_name: IdentifierNode,
    subcomponents: Vec<IdentifierNode>,
    alias: Option<IdentifierNode>,
}

/// A node containing a model usage declaration
pub type UseModelNode = Node<UseModel>;

impl UseModel {
    /// Creates a new model usage declaration
    #[must_use]
    pub const fn new(
        model_name: IdentifierNode,
        subcomponents: Vec<IdentifierNode>,
        alias: Option<IdentifierNode>,
    ) -> Self {
        Self {
            model_name,
            subcomponents,
            alias,
        }
    }

    /// Returns the name of the model being used
    #[must_use]
    pub const fn model_name(&self) -> &IdentifierNode {
        &self.model_name
    }

    /// Returns the list of subcomponents being used
    #[must_use]
    pub fn subcomponents(&self) -> &[IdentifierNode] {
        &self.subcomponents
    }

    /// Returns the optional alias for the model usage
    #[must_use]
    pub const fn alias(&self) -> Option<&IdentifierNode> {
        self.alias.as_ref()
    }
}
