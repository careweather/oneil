use std::{collections::HashMap, ops::Deref};

use crate::{
    expr::Expr,
    reference::{Identifier, ModulePath},
};

#[derive(Debug, Clone, PartialEq)]
pub struct ModelTest {}

impl ModelTest {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubmodelTest {
    submodel_path: ModulePath,
    inputs: SubmodelTestInputs,
}

impl SubmodelTest {
    pub fn new(submodel_path: ModulePath, inputs: SubmodelTestInputs) -> Self {
        Self {
            submodel_path,
            inputs,
        }
    }

    pub fn submodel_path(&self) -> &ModulePath {
        &self.submodel_path
    }

    pub fn inputs(&self) -> &SubmodelTestInputs {
        &self.inputs
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubmodelTestInputs(HashMap<Identifier, Expr>);

impl SubmodelTestInputs {
    pub fn new(inputs: HashMap<Identifier, Expr>) -> Self {
        Self(inputs)
    }
}

impl Deref for SubmodelTestInputs {
    type Target = HashMap<Identifier, Expr>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
