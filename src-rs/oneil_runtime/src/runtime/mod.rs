//! Runtime for the Oneil programming language.
//!
//! The runtime is split into submodules by concern: source loading, AST, IR,
//! evaluation, analysis, builtins, utilities, and (optionally) Python.

mod analysis;
mod ast;
mod builtin;
mod eval;
mod ir;
mod source;
mod util;

#[cfg(feature = "python")]
mod python;

#[cfg(feature = "python")]
use crate::cache::PythonImportCache;
use crate::cache::{AstCache, EvalCache, IrCache, SourceCache};
use crate::std_builtin::StdBuiltins;

/// Runtime for the Oneil programming language.
///
/// The runtime manages caches for source files, ASTs, and IR, and provides
/// methods to load and process Oneil models.
#[derive(Debug)]
pub struct Runtime {
    pub(super) source_cache: SourceCache,
    pub(super) ast_cache: AstCache,
    pub(super) ir_cache: IrCache,
    pub(super) eval_cache: EvalCache,
    #[cfg(feature = "python")]
    pub(super) python_import_cache: PythonImportCache,
    pub(super) builtins: StdBuiltins,
}
