//! Declaration constructs for the AST
//!
//! This module contains structures for representing declarations in Oneil programs,
//! including imports, model usage, parameters, and tests.

use std::ops::Deref;

use crate::{
    naming::{DirectoryNode, IdentifierNode},
    node::Node,
    parameter::ParameterNode,
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
    directory_path: Vec<DirectoryNode>,
    model: ModelInfoNode,
    submodel_list: Option<SubmodelListNode>,
}

/// A node containing a model usage declaration
pub type UseModelNode = Node<UseModel>;

impl UseModel {
    /// Creates a new model usage declaration
    #[must_use]
    pub const fn new(
        directory_path: Vec<DirectoryNode>,
        model: ModelInfoNode,
        submodel_list: Option<SubmodelListNode>,
    ) -> Self {
        Self {
            directory_path,
            model,
            submodel_list,
        }
    }

    /// Returns the directory path for the model usage
    #[must_use]
    pub const fn directory_path(&self) -> &[DirectoryNode] {
        self.directory_path.as_slice()
    }

    /// Returns the model info being used
    #[must_use]
    pub const fn model_info(&self) -> &ModelInfoNode {
        &self.model
    }

    /// Returns the list of submodels being used
    #[must_use]
    pub fn submodels(&self) -> Option<&SubmodelListNode> {
        self.submodel_list.as_ref()
    }

    /// Returns the relative path of the model
    #[must_use]
    pub fn get_model_relative_path(&self) -> String {
        let mut path = self
            .directory_path
            .iter()
            .map(|d| d.as_str())
            .collect::<Vec<_>>();
        path.push(self.model.top_component.as_str());
        path.join("/")
    }
}

/// A collection of imported model info
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelInfo {
    top_component: IdentifierNode,
    subcomponents: Vec<IdentifierNode>,
    alias: Option<IdentifierNode>,
}

/// A node containing model info
pub type ModelInfoNode = Node<ModelInfo>;

impl ModelInfo {
    /// Creates a new model info
    #[must_use]
    pub const fn new(
        top_component: IdentifierNode,
        subcomponents: Vec<IdentifierNode>,
        alias: Option<IdentifierNode>,
    ) -> Self {
        Self {
            top_component,
            subcomponents,
            alias,
        }
    }

    /// Returns the top component of the model info
    #[must_use]
    pub const fn top_component(&self) -> &IdentifierNode {
        &self.top_component
    }

    /// Returns the list of subcomponents of the model
    #[must_use]
    pub const fn subcomponents(&self) -> &[IdentifierNode] {
        self.subcomponents.as_slice()
    }

    /// Returns the optional alias of the submodel
    #[must_use]
    pub const fn alias(&self) -> Option<&IdentifierNode> {
        self.alias.as_ref()
    }
}

/// A collection of submodel information nodes
///
/// `SubmodelList` represents a list of submodels that are being used or imported
/// as part of a model usage declaration. Each submodel in the list contains
/// information about the model's top component, subcomponents, and optional alias.
///
/// This is a newtype wrapper around `Vec<ModelInfoNode>` that provides
/// semantic meaning to the collection and implements `Deref` for convenient
/// access to the underlying vector methods.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmodelList(Vec<ModelInfoNode>);

/// A node containing a submodel list
pub type SubmodelListNode = Node<SubmodelList>;

impl SubmodelList {
    /// Creates a new submodel list
    #[must_use]
    pub const fn new(submodel_list: Vec<ModelInfoNode>) -> Self {
        Self(submodel_list)
    }
}

impl Deref for SubmodelList {
    type Target = Vec<ModelInfoNode>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
