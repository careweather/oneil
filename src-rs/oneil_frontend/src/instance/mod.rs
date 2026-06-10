//! Instancing pass: per-unit graph build, assembly, and design overlay.
//!
//! The frontend's two-phase pipeline lives across this module and the
//! sibling [`crate::resolver`] module:
//!
//! 1. **Resolver** (`crate::resolver`, file-static): parses each file
//!    and lowers it into an [`InstancedModel`] *template* together
//!    with declarative design metadata (`Design`, [`ApplyDesign`]).
//!    Templates have stub child instances and record only file-static
//!    diagnostics.
//!
//! 2. **Instancing** (this module, graph-time): combines the cached
//!    templates into an [`InstanceGraph`] via the per-unit build
//!    pipeline ([`build_unit_graph`] + [`apply_designs`]). The build
//!    inlines each referenced unit's cached subtree, overlays design
//!    contributions, and detects cross-file cycles
//!    ([`CompilationCycleError`]).
//!
//! The core types here are:
//! - [`CompilationUnit`]: cache-key + cycle-stack identity for the
//!   per-unit build.
//! - [`InstancedModel`]: a node in the instance tree.
//! - [`InstanceGraph`]: the user-rooted tree plus its `reference_pool`
//!   of root-shared `reference`-target instances and graph-time error
//!   buckets like [`InstanceGraph::cycle_errors`].

pub mod compilation_unit;
pub mod cycle_error;
pub mod design;
pub mod graph;
pub mod imports;
pub mod model;
pub mod validation_error;

pub use compilation_unit::CompilationUnit;
pub use cycle_error::CompilationCycleError;
pub use design::{ApplyDesign, OverlayParameterValue};
pub use graph::{
    BuiltinLookup, CycleStackFrame, InstanceGraph, ModelDesignInfo, UnitGraphCache, apply_designs,
    build_instance_graph, build_unit_graph, build_unit_graph_for, classify_variables,
};
pub use imports::{AliasImport, ReferenceImport, SubmodelImport};
pub use model::{ContributionDiagnostic, InstancedModel};
pub use validation_error::{
    CycleMember, HostLocation, InstanceValidationError, InstanceValidationErrorKind,
};
