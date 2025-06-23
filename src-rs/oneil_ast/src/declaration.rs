use super::expression::Expr;
use super::parameter::Parameter;
use super::test::Test;

/// A declaration in an Oneil program
///
/// Declarations are top-level constructs that define imports, model usage,
/// parameters, and tests.
#[derive(Debug, Clone, PartialEq)]
pub enum Decl {
    Import {
        path: String,
    },

    UseModel {
        model_name: String,
        subcomponents: Vec<String>,
        inputs: Option<Vec<ModelInput>>,
        as_name: Option<String>,
    },

    Parameter(Parameter),
    Test(Test),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelInput {
    pub name: String,
    pub value: Expr,
}
