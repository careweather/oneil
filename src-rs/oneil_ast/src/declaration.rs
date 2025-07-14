use super::expression::Expr;
use super::parameter::Parameter;
use super::test::Test;

/// A declaration in an Oneil program
///
/// Declarations are top-level constructs that define imports, model usage,
/// parameters, and tests.
#[derive(Debug, Clone, PartialEq)]
pub enum Decl {
    Import(Import),

    UseModel(UseModel),

    Parameter(Parameter),
    Test(Test),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    pub path: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UseModel {
    pub model_name: String,
    pub subcomponents: Vec<String>,
    pub inputs: Option<Vec<ModelInput>>,
    pub as_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelInput {
    pub name: String,
    pub value: Expr,
}
