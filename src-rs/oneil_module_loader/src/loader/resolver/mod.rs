use oneil_module::{
    module::Module,
    parameter::Parameter,
    reference::{Identifier, ModulePath},
};

use crate::util::info::InfoMap;

mod expr;
mod parameter;
mod submodel;
mod test;
mod trace_level;
mod unit;
mod variable;

pub use parameter::resolve_parameters;
pub use submodel::resolve_submodels_and_tests;
pub use test::resolve_model_tests;
pub use test::resolve_submodel_tests;

pub type ParameterInfo<'a> = InfoMap<&'a Identifier, &'a Parameter>;
pub type SubmodelInfo<'a> = InfoMap<&'a Identifier, &'a ModulePath>;
pub type ModuleInfo<'a> = InfoMap<&'a ModulePath, &'a Module>;
