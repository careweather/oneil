//! Output types for the Oneil runtime.

pub mod dependency;
pub mod error;
pub mod ir_result;
pub mod reference;
mod tree;

pub use dependency::DependencyGraph;
pub use ir_result::IrLoadResult;
pub use tree::Tree;

pub use oneil_ast as ast;
pub use oneil_ir as ir;
pub use oneil_output::{
    BuiltinDependency, DebugInfo, DependencySet, ExternalDependency, Model, Number, Parameter,
    ParameterDependency, PrintLevel, Test, TestResult, Unit, Value,
};
pub use oneil_shared::{error::OneilError, span::Span};
