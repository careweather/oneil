//! The unified model intermediate representation for the Oneil frontend.
//!
//! [`InstancedModel`] is a node in the instance tree. It plays two roles:
//!
//! 1. **Template state** (resolver output, cached per file). The
//!    resolver fills in `parameters`, `tests`, `python_imports`, the
//!    `references` and `aliases` maps, and creates `submodels` entries
//!    with [`SubmodelImport::stub`]ped child instances. No cross-file
//!    recursion has happened yet.
//! 2. **Built state** (build pass output, cached per unit). The build
//!    pass clones the template, recursively builds each child subtree
//!    in place of the stubs, and applies the file's own design
//!    contributions.
//!
//! Graph-level diagnostics (cycle, contribution, and validation
//! errors) live on [`crate::InstanceGraph`], not on individual
//! `InstancedModel`s, so the per-file template stays a clean content
//! carrier in both states.

use indexmap::{IndexMap, map::Entry};
use oneil_ir as ir;
use oneil_shared::{
    InstancePath,
    labels::SectionLabel,
    paths::{ModelPath, PythonPath},
    symbols::{ParameterName, ReferenceName, TestIndex},
};

use super::imports::{AliasImport, ReferenceImport, SubmodelImport};
use crate::error::DesignResolutionError;

/// A contribution-time diagnostic produced by a design overlay.
///
/// Paired with the originating `apply` statement so the runtime can
/// surface a generic per-apply notice against the file that issued
/// the failing apply, alongside the precise diagnostic at the design
/// file's assignment span.
///
/// `host_path` identifies the affected instance as a path from the
/// containing graph's root. The runtime resolves it against the graph
/// to recover the host instance for filtering / span-collection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContributionDiagnostic {
    /// Path from the graph's root to the affected instance.
    pub host_path: InstancePath,
    /// The primary diagnostic, spanned at the assignment in the
    /// design file that introduced the failing contribution.
    pub error: DesignResolutionError,
    /// Path of the design file the error's primary span belongs to.
    pub design_file: ModelPath,
    /// The `apply X to Y` statement that brought this contribution
    /// in, when one exists. `None` for synthetic applies (the CLI
    /// design-as-root path) where no `apply` statement has a span.
    pub applied_via: Option<ir::DesignApplication>,
}

impl ContributionDiagnostic {
    /// Convenience constructor used by the apply pipeline at the site
    /// where the diagnostic is produced.
    #[must_use]
    pub const fn new(
        host_path: InstancePath,
        error: DesignResolutionError,
        design_file: ModelPath,
        applied_via: Option<ir::DesignApplication>,
    ) -> Self {
        Self {
            host_path,
            error,
            design_file,
            applied_via,
        }
    }
}

/// A node in the instance tree.
///
/// See module-level docs for the template / built state distinction.
#[derive(Debug, Clone, PartialEq)]
pub struct InstancedModel {
    path: ModelPath,
    python_imports: IndexMap<PythonPath, ir::PythonImport>,
    /// Owned child subtrees declared via `submodel foo as bar`.
    /// Keyed by alias (`bar`).
    submodels: IndexMap<ReferenceName, SubmodelImport>,
    /// Cross-file references declared via `reference x: ./foo.on`.
    /// The actual instance lives in the containing graph's
    /// `reference_pool`, keyed by the import's `path`.
    references: IndexMap<ReferenceName, ReferenceImport>,
    /// Local aliases for extraction-block submodels
    /// (`submodel a as cs [b.c]` → `cs` aliases the descent path `b.c`
    /// within the owned subtree of this instance).
    aliases: IndexMap<ReferenceName, AliasImport>,
    parameters: IndexMap<ParameterName, ir::Parameter>,
    tests: IndexMap<TestIndex, ir::Test>,
    note: Option<ir::Note>,
    /// Named sections in source order. Each section carries an optional note and an
    /// ordered item list referencing parameters/tests by ID.
    sections: IndexMap<SectionLabel, ir::Section>,
}

impl InstancedModel {
    /// Creates a new instance from already-prepared maps.
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "each map type is a distinct import category"
    )]
    pub fn new(
        path: ModelPath,
        python_imports: IndexMap<PythonPath, ir::PythonImport>,
        submodels: IndexMap<ReferenceName, SubmodelImport>,
        references: IndexMap<ReferenceName, ReferenceImport>,
        aliases: IndexMap<ReferenceName, AliasImport>,
        parameters: IndexMap<ParameterName, ir::Parameter>,
        tests: IndexMap<TestIndex, ir::Test>,
        note: Option<ir::Note>,
    ) -> Self {
        Self {
            path,
            python_imports,
            submodels,
            references,
            aliases,
            parameters,
            tests,
            note,
            sections: IndexMap::new(),
        }
    }

    /// Empty instance carrying just a path. Used as the template stub
    /// for a freshly-declared submodel before the build pass populates
    /// its content.
    #[must_use]
    pub fn empty_for(path: ModelPath) -> Self {
        Self {
            path,
            python_imports: IndexMap::new(),
            submodels: IndexMap::new(),
            references: IndexMap::new(),
            aliases: IndexMap::new(),
            parameters: IndexMap::new(),
            tests: IndexMap::new(),
            note: None,
            sections: IndexMap::new(),
        }
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    /// Returns the model file path this instance was lowered from.
    #[must_use]
    pub const fn path(&self) -> &ModelPath {
        &self.path
    }

    /// Returns the Python imports for this instance.
    #[must_use]
    pub const fn python_imports(&self) -> &IndexMap<PythonPath, ir::PythonImport> {
        &self.python_imports
    }

    /// Looks up a submodel by alias.
    #[must_use]
    pub fn get_submodel(&self, alias: &ReferenceName) -> Option<&SubmodelImport> {
        self.submodels.get(alias)
    }

    /// Returns all submodels keyed by alias.
    #[must_use]
    pub const fn submodels(&self) -> &IndexMap<ReferenceName, SubmodelImport> {
        &self.submodels
    }

    /// Mutable view of the submodels map.
    pub(crate) const fn submodels_mut(&mut self) -> &mut IndexMap<ReferenceName, SubmodelImport> {
        &mut self.submodels
    }

    /// Looks up a reference by alias.
    #[must_use]
    pub fn get_reference(&self, name: &ReferenceName) -> Option<&ReferenceImport> {
        self.references.get(name)
    }

    /// Returns all references keyed by alias.
    #[must_use]
    pub const fn references(&self) -> &IndexMap<ReferenceName, ReferenceImport> {
        &self.references
    }

    /// Mutable view of the references map.
    #[expect(dead_code, reason = "mutation accessor for future graph-level passes")]
    pub(crate) const fn references_mut(&mut self) -> &mut IndexMap<ReferenceName, ReferenceImport> {
        &mut self.references
    }

    /// Looks up an alias by name.
    #[must_use]
    pub fn get_alias(&self, name: &ReferenceName) -> Option<&AliasImport> {
        self.aliases.get(name)
    }

    /// Returns all `with`-extracted aliases keyed by alias.
    #[must_use]
    pub const fn aliases(&self) -> &IndexMap<ReferenceName, AliasImport> {
        &self.aliases
    }

    /// Mutable view of the aliases map.
    #[expect(dead_code, reason = "mutation accessor for future graph-level passes")]
    pub(crate) const fn aliases_mut(&mut self) -> &mut IndexMap<ReferenceName, AliasImport> {
        &mut self.aliases
    }

    /// Looks up a parameter by name.
    #[must_use]
    pub fn get_parameter(&self, name: &ParameterName) -> Option<&ir::Parameter> {
        self.parameters.get(name)
    }

    /// Returns all parameters.
    #[must_use]
    pub const fn parameters(&self) -> &IndexMap<ParameterName, ir::Parameter> {
        &self.parameters
    }

    /// Mutable view of the parameters map. Used by the build pass to
    /// link RHS expressions and apply design overlays in place.
    pub(crate) const fn parameters_mut(&mut self) -> &mut IndexMap<ParameterName, ir::Parameter> {
        &mut self.parameters
    }

    /// Returns all tests.
    #[must_use]
    pub const fn tests(&self) -> &IndexMap<TestIndex, ir::Test> {
        &self.tests
    }

    /// Mutable view of the tests map. Used by the pre-validation
    /// classification pass to reclassify variables in test expressions.
    pub(crate) const fn tests_mut(&mut self) -> &mut IndexMap<TestIndex, ir::Test> {
        &mut self.tests
    }

    /// Returns named sections in source order.
    ///
    /// Each section carries an optional note and an ordered list of
    /// parameter/test IDs that can be looked up in
    /// [`parameters`](Self::parameters) / [`tests`](Self::tests).
    #[must_use]
    pub const fn sections(&self) -> &IndexMap<SectionLabel, ir::Section> {
        &self.sections
    }

    /// Sets the section metadata, replacing any previously stored sections.
    ///
    /// Called by the resolver after parameters and tests have been resolved.
    pub fn set_sections(&mut self, sections: IndexMap<SectionLabel, ir::Section>) {
        self.sections = sections;
    }

    /// Moves `name` into `label`'s section, removing it from any prior section
    /// and updating the parameter's own `section_label` field.
    ///
    /// If `label` does not yet exist on this model it is created with `note`.
    /// Already-existing sections keep their existing note.
    pub(crate) fn place_parameter_in_section(
        &mut self,
        name: &ParameterName,
        label: &SectionLabel,
        note: Option<ir::Note>,
    ) {
        if let Some(parameter) = self.parameters.get_mut(name) {
            parameter.set_section_label(Some(label.clone()));
        }
        for section in self.sections.values_mut() {
            section.items_mut().retain(|item| match item {
                ir::SectionItem::Parameter(n) => n != name,
                ir::SectionItem::Test(_) => true,
            });
        }
        match self.sections.entry(label.clone()) {
            Entry::Occupied(mut entry) => {
                let items = entry.get_mut().items_mut();
                if !items
                    .iter()
                    .any(|item| matches!(item, ir::SectionItem::Parameter(n) if n == name))
                {
                    items.push(ir::SectionItem::Parameter(name.clone()));
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(ir::Section::new(
                    note,
                    vec![ir::SectionItem::Parameter(name.clone())],
                ));
            }
        }
    }

    /// Returns the model-level documentation note, if any.
    #[must_use]
    pub const fn note(&self) -> Option<&ir::Note> {
        self.note.as_ref()
    }

    // ── Mutating helpers (used by the resolver / build pass) ──────────────────

    /// Adds a Python import.
    pub fn add_python_import(&mut self, path: PythonPath, import: ir::PythonImport) {
        self.python_imports.insert(path, import);
    }

    /// Adds a reference declaration.
    pub fn add_reference(&mut self, name: ReferenceName, import: ReferenceImport) {
        self.references.insert(name, import);
    }

    /// Adds a submodel declaration.
    pub fn add_submodel(&mut self, alias: ReferenceName, import: SubmodelImport) {
        self.submodels.insert(alias, import);
    }

    /// Adds a `with`-extracted alias.
    pub fn add_alias(&mut self, alias: ReferenceName, import: AliasImport) {
        self.aliases.insert(alias, import);
    }

    /// Adds a parameter.
    pub fn add_parameter(&mut self, name: ParameterName, parameter: ir::Parameter) {
        self.parameters.insert(name, parameter);
    }

    /// Adds a test.
    pub fn add_test(&mut self, index: TestIndex, test: ir::Test) {
        self.tests.insert(index, test);
    }

    /// Sets the model-level documentation note.
    pub fn set_note(&mut self, note: ir::Note) {
        self.note = Some(note);
    }
}
