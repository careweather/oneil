#![cfg_attr(doc, doc = include_str!("../README.md"))]
//! Frontend for the Oneil programming language.
//!
//! Two passes, each owned by its own submodule:
//!
//! 1. **Resolution** ([`mod@resolver`], invoked via [`load_model`] /
//!    [`load_model_list`]): loads each `.on` / `.one` file, resolves
//!    local identifiers, units, imports, and design overlays, and
//!    produces a cached file-static [`InstancedModel`] template plus
//!    declarative design metadata for every file in the dependency
//!    tree. Resolver-time diagnostics live in
//!    [`ResolutionErrorCollection`].
//!
//! 2. **Instancing** ([`mod@instance`], invoked via
//!    [`build_unit_graph`] + [`apply_designs`], or the legacy one-shot
//!    [`build_instance_graph`]): walks the cached templates from a
//!    root and constructs the user-rooted instance tree, recursively
//!    inlining each referenced unit's cached subtree, applying design
//!    contributions, and detecting cross-file cycles. Returns a
//!    fully-built [`InstanceGraph`] whose buckets hold graph-time
//!    diagnostics ([`CompilationCycleError`],
//!    [`ContributionDiagnostic`], [`InstanceValidationError`]).
//!
//! See `docs/decisions/2026-04-24-two-pass-instance-graph.md` and
//! `docs/architecture/design-overlays.md`.

use indexmap::IndexMap;
use oneil_ast as ast;
use oneil_ir as ir;
use oneil_shared::paths::ModelPath;

mod context;
pub mod error;
pub mod instance;
mod resolver;

#[cfg(test)]
mod test;

pub use crate::context::{
    AstLoadingFailedError, ExternalResolutionContext, ModelResolutionResult,
    PythonImportLoadingFailedError,
};
pub use crate::error::{DesignResolutionError, ResolutionErrorCollection};
pub use crate::instance::{
    AliasImport, ApplyDesign, BuiltinLookup, CompilationCycleError, CompilationUnit,
    ContributionDiagnostic, CycleMember, CycleStackFrame, HostLocation, InstanceGraph,
    InstanceValidationError, InstanceValidationErrorKind, InstancedModel, ModelDesignInfo,
    OverlayParameterValue, ReferenceImport, SubmodelImport, UnitGraphCache, apply_designs,
    build_instance_graph, build_unit_graph, build_unit_graph_for, classify_variables,
};

use crate::context::ResolutionContext;
pub use crate::resolver::collect_design_target_path;

/// Loads a single model and all its dependencies.
///
/// Returns a per-path [`ModelResolutionResult`] map containing the lowered
/// template, design metadata, and resolution errors for every file in the
/// dependency tree.
pub fn load_model<E>(
    model_path: &ModelPath,
    external_context: &mut E,
) -> IndexMap<ModelPath, ModelResolutionResult>
where
    E: ExternalResolutionContext,
{
    load_model_list(&[model_path], external_context)
}

/// Loads multiple models and all their dependencies.
pub fn load_model_list<E>(
    model_paths: &[&ModelPath],
    external_context: &mut E,
) -> IndexMap<ModelPath, ModelResolutionResult>
where
    E: ExternalResolutionContext,
{
    let mut resolution_context = ResolutionContext::new(external_context);

    for model_path in model_paths {
        resolver::load_model(model_path, &mut resolution_context);
    }

    resolution_context.into_result()
}

/// Resolves an expression as if it were in the context of the given model.
///
/// # Errors
///
/// Returns the errors that occurred during variable resolution.
pub fn resolve_expr_in_model<E>(
    expr_ast: &ast::ExprNode,
    model_path: &ModelPath,
    external_context: &mut E,
) -> Result<ir::Expr, Vec<error::VariableResolutionError>>
where
    E: ExternalResolutionContext,
{
    let mut resolution_context = ResolutionContext::with_preloaded_models(external_context);
    resolution_context.push_active_model(model_path);

    resolver::resolve_expr(expr_ast, &resolution_context)
}
