//! Unified facade for the Oneil language crates.
//!
//! Re-exports all Oneil libraries under shortened names with the `oneil_` prefix removed.

#![expect(
    clippy::multiple_crate_versions,
    reason = "this isn't causing problems, and it's going to take time to fix"
)]

#[cfg(feature = "rust-lib")]
pub use rust_lib::*;

#[cfg(feature = "rust-lib")]
mod rust_lib {
    pub use oneil_analysis as analysis;
    pub use oneil_ast as ast;
    pub use oneil_builtins as builtins;
    pub use oneil_cli as cli;
    pub use oneil_eval as eval;
    pub use oneil_ir as ir;
    pub use oneil_lsp as lsp;
    pub use oneil_output as output;
    pub use oneil_parser as parser;
    pub use oneil_resolver as resolver;
    pub use oneil_runtime as runtime;
    pub use oneil_shared as shared;

    #[cfg(feature = "python-imports")]
    pub use oneil_python as python;
}

#[cfg(feature = "python-lib")]
pub use python_lib::*;

#[cfg(feature = "python-lib")]
mod python_lib {

    pub use oneil_python::oneil_python_module;
}
