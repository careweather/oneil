//! Import types for [`InstancedModel`](super::model::InstancedModel).
//!
//! `InstancedModel` exposes three child maps. This module defines the
//! value types stored in each:
//!
//! * [`SubmodelImport`] — owned child subtree (from `submodel foo as bar`).
//! * [`ReferenceImport`] — alias for a root-level instance shared via the
//!   containing graph's `reference_pool` (from `reference x: ./foo.on`).
//! * [`AliasImport`] — local alias for a relative path within the host's
//!   own subtree, created by an extraction item in the `[…]` block
//!   (e.g. `submodel a as cs [b.c]` extracts `b.c` into alias `cs`).
//!
//! Resolver vs. build-pass roles:
//!
//! * The resolver populates `references` and `aliases` fully from
//!   syntactic information, and creates each `SubmodelImport` with a
//!   stub `instance` that carries only the child's `ModelPath`.
//! * The build pass walks the submodel declarations, recursively builds
//!   each child's subtree, and replaces the stub `instance`.

use oneil_shared::{
    InstancePath,
    paths::ModelPath,
    span::Span,
    symbols::{ReferenceName, SubmodelName},
};

use super::model::InstancedModel;

/// A `reference` declaration — a cross-file pointer to another root model.
///
/// The actual `InstancedModel` lives in the containing graph's
/// `reference_pool`, keyed by [`Self::path`]; this struct only records
/// the alias's source span and target path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceImport {
    /// Source-level reference name (`foo` in `reference foo as bar`).
    pub name: ReferenceName,
    /// Span of the reference's alias identifier in source.
    pub name_span: Span,
    /// Explicit `as` alias when the declaration includes one; otherwise
    /// the map key is the model name and this is `None`.
    pub alias: Option<ReferenceName>,
    /// Span of [`Self::alias`] in source, if present.
    pub alias_span: Option<Span>,
    /// On-disk path of the referenced model file. Doubles as the lookup
    /// key into the containing graph's `reference_pool`.
    pub path: ModelPath,
}

impl ReferenceImport {
    /// Creates a new reference import.
    #[must_use]
    pub const fn new(
        name: ReferenceName,
        name_span: Span,
        alias: Option<ReferenceName>,
        alias_span: Option<Span>,
        path: ModelPath,
    ) -> Self {
        Self {
            name,
            name_span,
            alias,
            alias_span,
            path,
        }
    }
}

/// A `submodel` declaration — an owned child instance directly nested
/// under the parent in the instance tree.
#[derive(Debug, Clone, PartialEq)]
pub struct SubmodelImport {
    /// Source-level model name (`foo` in `submodel foo as bar`).
    pub name: SubmodelName,
    /// Span of the source-level model name in the file.
    pub name_span: Span,
    /// Explicit `as` alias when the declaration includes one; otherwise
    /// the map key is the model name and this is `None`.
    pub alias: Option<ReferenceName>,
    /// Span of [`Self::alias`] in source, if present.
    pub alias_span: Option<Span>,
    /// The owned child subtree. The resolver creates this as a path-only
    /// stub (see [`InstancedModel::empty_for`]); the build pass replaces
    /// it with the recursively-built subtree.
    pub instance: Box<InstancedModel>,
}

impl SubmodelImport {
    /// Creates a new submodel import with a path-only stub child.
    /// The build pass replaces the stub with the built subtree.
    #[must_use]
    pub fn stub(
        name: SubmodelName,
        name_span: Span,
        alias: Option<ReferenceName>,
        alias_span: Option<Span>,
        child_path: ModelPath,
    ) -> Self {
        Self {
            name,
            name_span,
            alias,
            alias_span,
            instance: Box::new(InstancedModel::empty_for(child_path)),
        }
    }
}

/// A `with`-extracted submodel — a *local alias* for an instance
/// already reachable via a chain of reference-name segments under the
/// host instance.
///
/// Aliases never introduce a new instance; they rename an existing
/// path. Eval resolves an alias by descending its [`Self::alias_path`]
/// from the host's absolute key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliasImport {
    /// Source-level submodel name (`foo` in `submodel foo as bar [x]`).
    pub source: SubmodelName,
    /// Span of the source-level submodel name in the file.
    pub name_span: Span,
    /// Explicit `as` alias from the extraction item when one was written;
    /// otherwise the map key is derived from the extracted path and this
    /// is `None`.
    pub alias: Option<ReferenceName>,
    /// Span of [`Self::alias`] in source, if present.
    pub alias_span: Option<Span>,
    /// Reference-name segments to descend from the host instance to
    /// reach the alias target.
    pub alias_path: InstancePath,
}

impl AliasImport {
    /// Creates a new alias import.
    #[must_use]
    pub const fn new(
        source: SubmodelName,
        name_span: Span,
        alias: Option<ReferenceName>,
        alias_span: Option<Span>,
        alias_path: InstancePath,
    ) -> Self {
        Self {
            source,
            name_span,
            alias,
            alias_span,
            alias_path,
        }
    }
}
