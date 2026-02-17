//! Output types for the Oneil runtime.

pub mod error;
pub mod reference;

pub use oneil_analysis::output;
pub use oneil_ast as ast;
pub use oneil_ir as ir;
pub use oneil_output::{
    BuiltinDependency, DebugInfo, DependencySet, ExternalDependency, Model, Number, Parameter,
    ParameterDependency, PrintLevel, Test, TestResult, Unit, Value,
};
pub use oneil_shared::{error::OneilError, span::Span};

pub mod tree {
    //! Tree output types.

    pub use oneil_analysis::output::{DependencyTreeValue, ReferenceTreeValue, Tree};
}
