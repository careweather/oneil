//! Declaration constructs for the AST

// TODO: rename `Import` to `ImportPython`
use std::{ops::Deref, path::PathBuf};

use oneil_shared::paths::{DesignPath, ModelPath};

use crate::{
    debug_info::TraceLevelNode,
    naming::{DirectoryNode, IdentifierNode, ParameterLabelNode, RenderNameNode},
    node::Node,
    note::NoteNode,
    parameter::{LimitsNode, ParameterNode, ParameterValueNode, PerformanceMarkerNode},
    test::TestNode,
};

/// A declaration in an Oneil program
#[derive(Debug, Clone, PartialEq)]
pub enum Decl {
    /// Import declaration for including other modules
    Import(ImportNode),

    /// Submodel declaration for importing other models
    Submodel(SubmodelDeclNode),

    /// Declares that this file is a design file for another model (`design <name>`).
    DesignTarget(DesignTargetNode),

    /// Applies a design file to a specific reference path (`apply <file> to <ref>(.<ref>)*
    /// [\[ nested \]]`).
    ApplyDesign(ApplyDesignNode),

    /// Parameter assignment in a design file (`id(.<ref>)* = expr`, no label preamble).
    DesignParameter(DesignParameterNode),

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

    /// Creates a submodel declaration
    #[must_use]
    pub const fn submodel(submodel: SubmodelDeclNode) -> Self {
        Self::Submodel(submodel)
    }

    /// Creates a design target declaration
    #[must_use]
    pub const fn design_target(node: DesignTargetNode) -> Self {
        Self::DesignTarget(node)
    }

    /// Creates an `apply` declaration
    #[must_use]
    pub const fn apply_design(node: ApplyDesignNode) -> Self {
        Self::ApplyDesign(node)
    }

    /// Creates a design parameter line
    #[must_use]
    pub const fn design_parameter(node: DesignParameterNode) -> Self {
        Self::DesignParameter(node)
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

/// A submodel declaration that imports another model
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmodelDecl {
    directory_path: Vec<DirectoryNode>,
    model: ModelInfoNode,
    submodel_list: Option<SubmodelListNode>,
    model_kind: ModelKind,
}

/// A node containing a submodel declaration
pub type SubmodelDeclNode = Node<SubmodelDecl>;

impl SubmodelDecl {
    /// Creates a new submodel declaration
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

    /// Returns the directory path for the submodel
    #[must_use]
    pub const fn directory_path(&self) -> &[DirectoryNode] {
        self.directory_path.as_slice()
    }

    /// Returns the model info being imported
    #[must_use]
    pub const fn model_info(&self) -> &ModelInfoNode {
        &self.model
    }

    /// Returns the list of submodels being extracted
    #[must_use]
    pub const fn imported_submodels(&self) -> Option<&SubmodelListNode> {
        self.submodel_list.as_ref()
    }

    /// Returns the kind of model being imported
    #[must_use]
    pub const fn model_kind(&self) -> ModelKind {
        self.model_kind
    }

    /// Returns the relative path of the model
    #[must_use]
    pub fn get_model_relative_path(&self) -> ModelPath {
        let mut path = self
            .directory_path
            .iter()
            .map(|d| d.as_str())
            .collect::<Vec<_>>();
        path.push(self.model.top_component().as_str());

        let path = PathBuf::from(path.join("/"));

        ModelPath::from_path_no_ext(&path)
    }

    /// Returns the corresponding `.one` design file path for this submodel declaration.
    ///
    /// This mirrors [`get_model_relative_path`](Self::get_model_relative_path) but emits
    /// a [`DesignPath`] with a `.one` extension instead of a `.on` model path.  Used when
    /// the resolver needs to fall back to a design file when no `.on` model exists for
    /// this name.
    #[must_use]
    pub fn get_design_relative_path(&self) -> DesignPath {
        let mut path = self
            .directory_path
            .iter()
            .map(|d| d.as_str())
            .collect::<Vec<_>>();
        path.push(self.model.top_component().as_str());

        let path = PathBuf::from(path.join("/"));

        DesignPath::from_path_no_ext(&path)
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

    /// Returns the explicit alias if one was provided in the declaration.
    ///
    /// This returns `None` if no `as <alias>` was specified.
    #[must_use]
    pub const fn alias(&self) -> Option<&IdentifierNode> {
        self.alias.as_ref()
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
    /// submodel foo/bar.baz as qux
    ///
    /// # name: `foo`
    /// reference foo as bar
    ///
    /// # name: `bar`
    /// submodel foo/bar
    ///
    /// # name: `foo`
    /// reference foo
    /// ```
    #[must_use]
    pub fn get_model_name(&self) -> &IdentifierNode {
        self.subcomponents.last().unwrap_or(&self.top_component)
    }

    /// Returns the reference name of the model.
    ///
    /// This is the given alias if one is provided. Otherwise, it is the model
    /// name.
    ///
    /// ## Examples
    ///
    /// ```oneil
    /// # alias: `qux`
    /// submodel foo/bar.baz as qux
    ///
    /// # alias: `bar`
    /// reference foo as bar
    ///
    /// # alias: `bar`
    /// submodel foo/bar
    ///
    /// # alias: `foo`
    /// reference foo
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

/// Target model path in a `design [path/to/]<name>` declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignTarget {
    /// Optional directory path (e.g., `../models/`).
    directory_path: Vec<DirectoryNode>,
    /// The target model name.
    target: IdentifierNode,
}

/// AST node for a [`DesignTarget`].
pub type DesignTargetNode = Node<DesignTarget>;

impl DesignTarget {
    /// Creates a design target declaration with just a model name.
    #[must_use]
    pub const fn new(target: IdentifierNode) -> Self {
        Self {
            directory_path: Vec::new(),
            target,
        }
    }

    /// Creates a design target declaration with a directory path.
    #[must_use]
    pub const fn with_path(directory_path: Vec<DirectoryNode>, target: IdentifierNode) -> Self {
        Self {
            directory_path,
            target,
        }
    }

    /// Returns the directory path for the target model.
    #[must_use]
    pub const fn directory_path(&self) -> &[DirectoryNode] {
        self.directory_path.as_slice()
    }

    /// Returns the target model identifier.
    #[must_use]
    pub const fn target(&self) -> &IdentifierNode {
        &self.target
    }

    /// Returns the relative path of the target model.
    #[must_use]
    pub fn get_target_relative_path(&self) -> ModelPath {
        let mut path = self
            .directory_path
            .iter()
            .map(|d| d.as_str())
            .collect::<Vec<_>>();
        path.push(self.target.as_str());

        let path = PathBuf::from(path.join("/"));

        ModelPath::from_path_no_ext(&path)
    }
}

/// `apply [path/to/]<file> to <ref>(.<ref>)* [ '[' nested_applies ']' ]`.
///
/// Applies a design file to a specific reference path on the current model
/// (or design target). Nested applies appear in a `[ … ]` block and may
/// recursively address deeper references; nested entries omit the `apply`
/// keyword (they are parsed as `<file> to <ref>(.<ref>)*`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyDesign {
    /// Optional directory path (e.g., `../designs/`).
    directory_path: Vec<DirectoryNode>,
    /// Design file name (without extension).
    design_file: IdentifierNode,
    /// Non-empty path of reference segments identifying where the design lands.
    target: Vec<IdentifierNode>,
    /// Recursive applies attached under `target`. Each entry is itself an
    /// [`ApplyDesignNode`]; the resolver flattens these by concatenating the
    /// outer `target` with the nested entry's `target` before applying.
    nested_applies: Vec<ApplyDesignNode>,
}

/// AST node for an [`ApplyDesign`].
pub type ApplyDesignNode = Node<ApplyDesign>;

impl ApplyDesign {
    /// Creates an `apply` declaration with the given target path and (possibly empty)
    /// nested applies.
    #[must_use]
    pub const fn new(
        directory_path: Vec<DirectoryNode>,
        design_file: IdentifierNode,
        target: Vec<IdentifierNode>,
        nested_applies: Vec<ApplyDesignNode>,
    ) -> Self {
        Self {
            directory_path,
            design_file,
            target,
            nested_applies,
        }
    }

    /// Returns the directory path for the design file.
    #[must_use]
    pub const fn directory_path(&self) -> &[DirectoryNode] {
        self.directory_path.as_slice()
    }

    /// Design file name.
    #[must_use]
    pub const fn design_file(&self) -> &IdentifierNode {
        &self.design_file
    }

    /// Returns the non-empty `to <ref>(.<ref>)*` path identifying the apply target.
    #[must_use]
    pub const fn target(&self) -> &[IdentifierNode] {
        self.target.as_slice()
    }

    /// Returns nested applies declared under this target's `[ … ]` block.
    #[must_use]
    pub const fn nested_applies(&self) -> &[ApplyDesignNode] {
        self.nested_applies.as_slice()
    }

    /// Returns the relative path of the design file with the `.one` extension.
    #[must_use]
    pub fn get_design_relative_path(&self) -> DesignPath {
        let mut path = self
            .directory_path
            .iter()
            .map(|d| d.as_str())
            .collect::<Vec<_>>();
        path.push(self.design_file.as_str());

        let path = PathBuf::from(path.join("/"));

        DesignPath::from_path_no_ext(&path)
    }
}

/// `<id>(.<segment>)? = value` line allowed after `design` in design files.
///
/// When `instance_path` is `Some`, the parameter override applies to a
/// descendant instance reached by that single reference name.
/// For example, `mass.sat = 5 kg` overrides `mass` on the `sat` instance.
///
/// `performance_marker` and `trace_level` only take effect when this line
/// introduces a brand-new parameter (i.e. it is not present on the design
/// target); they are ignored for plain overrides.
///
/// `limits` in the full form set limits for new parameters, or adjust existing
/// limits on overrides.
#[derive(Debug, Clone, PartialEq)]
pub struct DesignParameter {
    ident: IdentifierNode,
    /// Single reference-name scoping the override (`None` for a flat override on the
    /// design target).
    instance_path: Option<IdentifierNode>,
    value: ParameterValueNode,
    performance_marker: Option<PerformanceMarkerNode>,
    trace_level: Option<TraceLevelNode>,
    note: Option<NoteNode>,
    /// Optional human-readable label (`None` for shorthand additions without a label prefix).
    label: Option<ParameterLabelNode>,
    /// Optional value limits. For additions, sets the parameter limits. For overrides, replaces
    /// the target limits when set. Present in both full form (`Label [Limits]: id = value`) and
    /// shorthand form (`id [Limits] = value`).
    limits: Option<LimitsNode>,
    /// Optional LaTeX render-name written as `{...}` after the `:` (only valid when label is present).
    render_name: Option<RenderNameNode>,
}

/// AST node for a [`DesignParameter`].
pub type DesignParameterNode = Node<DesignParameter>;

impl DesignParameter {
    /// Creates a design parameter line with the given (possibly absent) instance reference.
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "all fields are required for full construction"
    )]
    pub const fn new(
        ident: IdentifierNode,
        instance_path: Option<IdentifierNode>,
        value: ParameterValueNode,
        performance_marker: Option<PerformanceMarkerNode>,
        trace_level: Option<TraceLevelNode>,
        note: Option<NoteNode>,
        label: Option<ParameterLabelNode>,
        limits: Option<LimitsNode>,
        render_name: Option<RenderNameNode>,
    ) -> Self {
        Self {
            ident,
            instance_path,
            value,
            performance_marker,
            trace_level,
            note,
            label,
            limits,
            render_name,
        }
    }

    /// Parameter identifier being assigned.
    #[must_use]
    pub const fn ident(&self) -> &IdentifierNode {
        &self.ident
    }

    /// Single reference name scoping the override (`None` for a flat override).
    #[must_use]
    pub const fn instance_path(&self) -> Option<&IdentifierNode> {
        self.instance_path.as_ref()
    }

    /// Assigned value (expression or piecewise).
    #[must_use]
    pub const fn value(&self) -> &ParameterValueNode {
        &self.value
    }

    /// Output parameter marker (`$`); only meaningful for new parameters.
    #[must_use]
    pub const fn performance_marker(&self) -> Option<&PerformanceMarkerNode> {
        self.performance_marker.as_ref()
    }

    /// Trace level (`*` / `**`); only meaningful for new parameters.
    #[must_use]
    pub const fn trace_level(&self) -> Option<&TraceLevelNode> {
        self.trace_level.as_ref()
    }

    /// Optional trailing note.
    #[must_use]
    pub const fn note(&self) -> Option<&NoteNode> {
        self.note.as_ref()
    }

    /// Optional human-readable label (`None` for shorthand additions without a label prefix).
    #[must_use]
    pub const fn label(&self) -> Option<&ParameterLabelNode> {
        self.label.as_ref()
    }

    /// Optional value limits. For additions, sets
    /// the parameter limits. For overrides, replaces the target limits when set.
    #[must_use]
    pub const fn limits(&self) -> Option<&LimitsNode> {
        self.limits.as_ref()
    }

    /// Optional LaTeX render-name written as `{...}` after the `:` (only valid when label is present).
    #[must_use]
    pub const fn render_name(&self) -> Option<&RenderNameNode> {
        self.render_name.as_ref()
    }
}
