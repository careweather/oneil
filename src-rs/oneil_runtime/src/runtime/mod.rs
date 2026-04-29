//! Runtime for the Oneil programming language.
//!
//! The runtime is split into submodules by concern: source loading, AST, IR,
//! evaluation, analysis, builtins, utilities, and (optionally) Python.

#![allow(
    clippy::multiple_inherent_impl,
    reason = "this allows the runtime to be split up into its different functionionalities"
)]

mod analysis;
mod ast;
mod builtin;
mod error;
mod eval;
mod ir;
mod source;
mod util;

#[cfg(feature = "python")]
mod python;

#[cfg(feature = "python")]
use std::path::PathBuf;

use crate::cache::{AstCache, EvalCache, SourceCache};
#[cfg(feature = "python")]
use crate::cache::{PythonCallCache, PythonImportCache};
use indexmap::IndexMap;
use oneil_builtins::BuiltinRef;
use oneil_frontend::instance::graph::{ModelDesignInfo, UnitGraphCache};
use oneil_frontend::{BuiltinLookup, InstanceGraph};
use oneil_shared::paths::ModelPath;

/// Adapter wiring the runtime's builtin table into the frontend's
/// [`BuiltinLookup`] trait. Passed into
/// [`oneil_analysis::validate_instance_graph`] so the pre-validation
/// classification step can reclassify variables as builtins / parameters
/// against per-instance binding scopes.
pub struct RuntimeBuiltinLookup<'r> {
    pub runtime: &'r Runtime,
}

impl BuiltinLookup for RuntimeBuiltinLookup<'_> {
    fn has_builtin_value(&self, name: &str) -> bool {
        self.runtime.builtins.has_builtin_value(name)
    }
}

/// Runtime for the Oneil programming language.
///
/// The runtime manages caches for source files, ASTs, lowered templates,
/// and evaluation results, and provides methods to load and process Oneil models.
#[derive(Debug)]
pub struct Runtime {
    #[cfg(feature = "python")]
    cache_dir: PathBuf,
    source_cache: SourceCache,
    ast_cache: AstCache,
    /// Per-compilation-unit instance graph cache. Each entry is the unit's
    /// *self-rooted* graph (own applies pre-applied; no runtime-supplied
    /// designs). Composition happens in `eval_model_internal`, on top of the
    /// cached root unit graph, by overlaying runtime designs.
    ///
    /// Cleared whenever any source cache changes (see
    /// `Runtime::clear_non_source_caches`); per-unit invalidation is a
    /// follow-up.
    unit_graph_cache: UnitGraphCache,
    /// Accumulated `apply X to ref` declarations and design-export content for
    /// every loaded file. Populated incrementally by `load_and_lower_internal`
    /// and passed to `apply_designs` so that `apply_design_recursive` can walk
    /// nested design applies at composition time.
    ///
    /// Cleared alongside `unit_graph_cache` on any source change.
    design_info: IndexMap<ModelPath, ModelDesignInfo>,
    eval_cache: EvalCache,
    /// The most recently composed instance graph, kept reachable from
    /// `&self` so [`Runtime::get_model_diagnostics`] can pull per-instance
    /// diagnostics straight off it (validation, contribution, overlay-target-missing)
    /// and the graph-level cycle bucket.
    ///
    /// Populated by `eval_model_internal` after each composition + eval
    /// pass; cleared whenever any source cache changes (see
    /// [`Runtime::clear_non_source_caches`]) since stale graphs would
    /// surface diagnostics whose spans no longer match the current
    /// source. `None` before the first eval (or after a cache clear),
    /// which `get_model_diagnostics` treats as "no graph-time errors yet".
    composed_graph: Option<InstanceGraph>,
    #[cfg(feature = "python")]
    python_import_cache: PythonImportCache,
    #[cfg(feature = "python")]
    python_call_cache: PythonCallCache,
    #[cfg(feature = "python")]
    python_call_replacement_cache: PythonCallCache,
    builtins: BuiltinRef,
}
