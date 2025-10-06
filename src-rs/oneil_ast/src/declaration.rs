//! Declaration constructs for the AST

// TODO: rename `Import` to `ImportPython` and `UseModel` to `ImportModel`
use std::ops::Deref;

use crate::{
    naming::{DirectoryNode, IdentifierNode},
    node::Node,
    parameter::ParameterNode,
    test::TestNode,
};

/// A declaration in an Oneil program
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
    pub const fn parameter(parameter: ParameterNode) -> Self {
        Self::Parameter(parameter)
    }

    /// Creates a test declaration
    #[must_use]
    pub const fn test(test: TestNode) -> Self {
        Self::Test(test)
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
    model_kind: ModelKind,
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
        model_kind: ModelKind,
    ) -> Self {
        Self {
            directory_path,
            model,
            submodel_list,
            model_kind,
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
    pub const fn submodels(&self) -> Option<&SubmodelListNode> {
        self.submodel_list.as_ref()
    }

    /// Returns the kind of model being used
    #[must_use]
    pub const fn model_kind(&self) -> ModelKind {
        self.model_kind
    }

    /// Returns the relative path of the model
    #[must_use]
    pub fn get_model_relative_path(&self) -> String {
        let mut path = self
            .directory_path
            .iter()
            .map(|d| d.node_value().as_str())
            .collect::<Vec<_>>();
        path.push(
            self.model
                .node_value()
                .top_component()
                .node_value()
                .as_str(),
        );
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

    /// Returns the top component of the model
    #[must_use]
    pub const fn top_component(&self) -> &IdentifierNode {
        &self.top_component
    }

    /// Returns the list of subcomponents of the model
    #[must_use]
    pub const fn subcomponents(&self) -> &[IdentifierNode] {
        self.subcomponents.as_slice()
    }

    /// Returns the calculated name of the model
    ///
    /// This is the name of the last subcomponent, or the name of the top
    /// component if there are no subcomponents.
    ///
    /// ## Examples
    ///
    /// ```oneil
    /// # name: `baz`
    /// use foo/bar.baz as qux
    ///
    /// # name: `foo`
    /// ref foo as bar
    ///
    /// # name: `bar`
    /// use foo/bar
    ///
    /// # name: `foo`
    /// ref foo
    /// ```
    #[must_use]
    pub fn get_model_name(&self) -> &IdentifierNode {
        self.subcomponents.last().unwrap_or(&self.top_component)
    }

    /// Returns the calculated alias of the model
    ///
    /// This is the given alias if one is provided. Otherwise, it is the model
    /// name
    ///
    /// ## Examples
    ///
    /// ```oneil
    /// # alias: `qux`
    /// use foo/bar.baz as qux
    ///
    /// # alias: `bar`
    /// ref foo as bar
    ///
    /// # alias: `bar`
    /// use foo/bar
    ///
    /// # alias: `foo`
    /// ref foo
    /// ```
    #[must_use]
    pub fn get_alias(&self) -> &IdentifierNode {
        self.alias.as_ref().unwrap_or_else(|| self.get_model_name())
    }
}

/// A collection of submodel information nodes
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

/// The kind of model being used
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelKind {
    /// The model is being used for reference
    Reference,
    /// The model is being used as a submodel
    Submodel,
}

impl ModelKind {
    /// Returns the reference model kind
    #[must_use]
    pub const fn reference() -> Self {
        Self::Reference
    }

    /// Returns the submodel model kind
    #[must_use]
    pub const fn submodel() -> Self {
        Self::Submodel
    }
}
