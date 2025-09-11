use std::collections::HashSet;

use oneil_ir as ir;

use crate::BuiltinRef;

pub struct TestBuiltinRef {
    builtin_variables: HashSet<String>,
    builtin_functions: HashSet<String>,
}

impl TestBuiltinRef {
    pub fn new() -> Self {
        Self {
            builtin_variables: HashSet::new(),
            builtin_functions: HashSet::new(),
        }
    }

    pub fn with_builtin_variables(
        mut self,
        variables: impl IntoIterator<Item = &'static str>,
    ) -> Self {
        let variables = variables.into_iter().map(ToString::to_string);
        self.builtin_variables.extend(variables);
        self
    }

    pub fn with_builtin_functions(
        mut self,
        functions: impl IntoIterator<Item = &'static str>,
    ) -> Self {
        let functions = functions.into_iter().map(ToString::to_string);
        self.builtin_functions.extend(functions);
        self
    }
}

impl BuiltinRef for TestBuiltinRef {
    fn has_builtin_value(&self, identifier: &ir::Identifier) -> bool {
        self.builtin_variables.contains(identifier.as_str())
    }

    fn has_builtin_function(&self, identifier: &ir::Identifier) -> bool {
        self.builtin_functions.contains(identifier.as_str())
    }
}
