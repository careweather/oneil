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

    From {
        path: Vec<String>,
        use_model: String,
        inputs: Option<Vec<ModelInput>>,
        as_name: String,
    },

    Use {
        path: Vec<String>,
        inputs: Option<Vec<ModelInput>>,
        as_name: String,
    },

    Parameter(Parameter),
    Test(Test),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelInput {
    pub name: String,
    pub value: Expr,
}
