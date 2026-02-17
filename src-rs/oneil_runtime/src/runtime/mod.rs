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
use crate::cache::PythonImportCache;
use crate::cache::{AstCache, EvalCache, IrCache, SourceCache};
use crate::std_builtin::StdBuiltins;

/// Runtime for the Oneil programming language.
///
/// The runtime manages caches for source files, ASTs, and IR, and provides
/// methods to load and process Oneil models.
#[derive(Debug)]
pub struct Runtime {
    source_cache: SourceCache,
    ast_cache: AstCache,
    ir_cache: IrCache,
    eval_cache: EvalCache,
    #[cfg(feature = "python")]
    python_import_cache: PythonImportCache,
    builtins: StdBuiltins,
}
