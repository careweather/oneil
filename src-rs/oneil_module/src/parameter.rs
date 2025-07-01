use std::{collections::HashMap, ops::Deref};

use crate::reference::Identifier;

#[derive(Debug, Clone, PartialEq)]
pub struct ParameterCollection(HashMap<Identifier, Parameter>);

impl ParameterCollection {
    pub fn new(parameters: HashMap<Identifier, Parameter>) -> Self {
        Self(parameters)
    }

    // TODO: add methods for getting performance parameters, etc.
}

impl Deref for ParameterCollection {
    type Target = HashMap<Identifier, Parameter>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter;
