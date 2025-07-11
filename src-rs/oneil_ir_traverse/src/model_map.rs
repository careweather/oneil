use std::collections::HashMap;

use oneil_ir::{
    reference::{Identifier, ModelPath, PythonPath},
    test::TestIndex,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ModelMapEntry<PyT, SubmodelT, ParamT, ModelTestT, SubmodelTestT> {
    python_imports_map: HashMap<PythonPath, PyT>,
    submodels_map: HashMap<Identifier, SubmodelT>,
    parameters_map: HashMap<Identifier, ParamT>,
    model_tests_map: HashMap<TestIndex, ModelTestT>,
    submodel_tests_map: HashMap<Identifier, SubmodelTestT>,
}

impl<PyT, SubmodelT, ParamT, ModelTestT, SubmodelTestT>
    ModelMapEntry<PyT, SubmodelT, ParamT, ModelTestT, SubmodelTestT>
{
    pub fn new(
        python_imports_map: HashMap<PythonPath, PyT>,
        submodels_map: HashMap<Identifier, SubmodelT>,
        parameters_map: HashMap<Identifier, ParamT>,
        model_tests_map: HashMap<TestIndex, ModelTestT>,
        submodel_tests_map: HashMap<Identifier, SubmodelTestT>,
    ) -> Self {
        Self {
            python_imports_map,
            submodels_map,
            parameters_map,
            model_tests_map,
            submodel_tests_map,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelMap<PyT, SubmodelT, ParamT, ModelTestT, SubmodelTestT> {
    map: HashMap<ModelPath, ModelMapEntry<PyT, SubmodelT, ParamT, ModelTestT, SubmodelTestT>>,
}

impl<PyT, SubmodelT, ParamT, ModelTestT, SubmodelTestT>
    ModelMap<PyT, SubmodelT, ParamT, ModelTestT, SubmodelTestT>
{
    pub fn new(
        map: HashMap<ModelPath, ModelMapEntry<PyT, SubmodelT, ParamT, ModelTestT, SubmodelTestT>>,
    ) -> Self {
        Self { map }
    }
}
