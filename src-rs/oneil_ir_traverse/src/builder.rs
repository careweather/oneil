use std::collections::HashMap;

use oneil_ir::{
    reference::{Identifier, ModelPath, PythonPath},
    test::TestIndex,
};

use crate::model_map::{ModelMap, ModelMapEntry};

#[derive(Debug, Clone, PartialEq)]
pub struct ModelMapEntryBuilder<PyT, SubmodelT, ParamT, ModelTestT, SubmodelTestT> {
    python_imports_map: HashMap<PythonPath, PyT>,
    submodels_map: HashMap<Identifier, SubmodelT>,
    parameters_map: HashMap<Identifier, ParamT>,
    model_tests_map: HashMap<TestIndex, ModelTestT>,
    submodel_tests_map: HashMap<Identifier, SubmodelTestT>,
}

impl<PyT, SubmodelT, ParamT, ModelTestT, SubmodelTestT>
    ModelMapEntryBuilder<PyT, SubmodelT, ParamT, ModelTestT, SubmodelTestT>
{
    pub fn new() -> Self {
        Self {
            python_imports_map: HashMap::new(),
            submodels_map: HashMap::new(),
            parameters_map: HashMap::new(),
            model_tests_map: HashMap::new(),
            submodel_tests_map: HashMap::new(),
        }
    }

    pub fn add_python_import_data(&mut self, path: PythonPath, value: PyT) {
        assert!(
            !self.python_imports_map.contains_key(&path),
            "python import already exists"
        );
        self.python_imports_map.insert(path, value);
    }

    pub fn add_submodel_data(&mut self, id: Identifier, value: SubmodelT) {
        assert!(
            !self.submodels_map.contains_key(&id),
            "submodel already exists"
        );
        self.submodels_map.insert(id, value);
    }

    pub fn add_parameter_data(&mut self, id: Identifier, value: ParamT) {
        assert!(
            !self.parameters_map.contains_key(&id),
            "parameter already exists"
        );
        self.parameters_map.insert(id, value);
    }

    pub fn add_model_test_data(&mut self, index: TestIndex, value: ModelTestT) {
        assert!(
            !self.model_tests_map.contains_key(&index),
            "model test already exists"
        );
        self.model_tests_map.insert(index, value);
    }

    pub fn add_submodel_test_data(&mut self, id: Identifier, value: SubmodelTestT) {
        assert!(
            !self.submodel_tests_map.contains_key(&id),
            "submodel test already exists"
        );
        self.submodel_tests_map.insert(id, value);
    }
}

impl<PyT, SubmodelT, ParamT, ModelTestT, SubmodelTestT>
    Into<ModelMapEntry<PyT, SubmodelT, ParamT, ModelTestT, SubmodelTestT>>
    for ModelMapEntryBuilder<PyT, SubmodelT, ParamT, ModelTestT, SubmodelTestT>
{
    fn into(self) -> ModelMapEntry<PyT, SubmodelT, ParamT, ModelTestT, SubmodelTestT> {
        ModelMapEntry::new(
            self.python_imports_map,
            self.submodels_map,
            self.parameters_map,
            self.model_tests_map,
            self.submodel_tests_map,
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelMapBuilder<
    PyT,
    PyE,
    SubmodelT,
    SubmodelE,
    ParamT,
    ParamE,
    ModelTestT,
    ModelTestE,
    SubmodelTestT,
    SubmodelTestE,
> {
    map:
        HashMap<ModelPath, ModelMapEntryBuilder<PyT, SubmodelT, ParamT, ModelTestT, SubmodelTestT>>,
    error_map:
        HashMap<ModelPath, ModelMapEntryBuilder<PyE, SubmodelE, ParamE, ModelTestE, SubmodelTestE>>,
}

impl<
    PyT,
    PyE,
    SubmodelT,
    SubmodelE,
    ParamT,
    ParamE,
    ModelTestT,
    ModelTestE,
    SubmodelTestT,
    SubmodelTestE,
>
    ModelMapBuilder<
        PyT,
        PyE,
        SubmodelT,
        SubmodelE,
        ParamT,
        ParamE,
        ModelTestT,
        ModelTestE,
        SubmodelTestT,
        SubmodelTestE,
    >
{
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            error_map: HashMap::new(),
        }
    }

    pub fn add_python_import_data(&mut self, model_path: ModelPath, path: PythonPath, value: PyT) {
        self.map
            .entry(model_path)
            .or_insert_with(ModelMapEntryBuilder::new)
            .add_python_import_data(path, value);
    }

    pub fn add_python_import_error(&mut self, model_path: ModelPath, path: PythonPath, error: PyE) {
        self.error_map
            .entry(model_path)
            .or_insert_with(ModelMapEntryBuilder::new)
            .add_python_import_data(path, error);
    }

    pub fn add_submodel_data(&mut self, model_path: ModelPath, id: Identifier, value: SubmodelT) {
        self.map
            .entry(model_path)
            .or_insert_with(ModelMapEntryBuilder::new)
            .add_submodel_data(id, value);
    }

    pub fn add_submodel_error(&mut self, model_path: ModelPath, id: Identifier, error: SubmodelE) {
        self.error_map
            .entry(model_path)
            .or_insert_with(ModelMapEntryBuilder::new)
            .add_submodel_data(id, error);
    }

    pub fn add_parameter_data(&mut self, model_path: ModelPath, id: Identifier, value: ParamT) {
        self.map
            .entry(model_path)
            .or_insert_with(ModelMapEntryBuilder::new)
            .add_parameter_data(id, value);
    }

    pub fn add_parameter_error(&mut self, model_path: ModelPath, id: Identifier, error: ParamE) {
        self.error_map
            .entry(model_path)
            .or_insert_with(ModelMapEntryBuilder::new)
            .add_parameter_data(id, error);
    }

    pub fn add_model_test_data(
        &mut self,
        model_path: ModelPath,
        index: TestIndex,
        value: ModelTestT,
    ) {
        self.map
            .entry(model_path)
            .or_insert_with(ModelMapEntryBuilder::new)
            .add_model_test_data(index, value);
    }

    pub fn add_model_test_error(
        &mut self,
        model_path: ModelPath,
        index: TestIndex,
        error: ModelTestE,
    ) {
        self.error_map
            .entry(model_path)
            .or_insert_with(ModelMapEntryBuilder::new)
            .add_model_test_data(index, error);
    }

    pub fn add_submodel_test_data(
        &mut self,
        model_path: ModelPath,
        id: Identifier,
        value: SubmodelTestT,
    ) {
        self.map
            .entry(model_path)
            .or_insert_with(ModelMapEntryBuilder::new)
            .add_submodel_test_data(id, value);
    }

    pub fn add_submodel_test_error(
        &mut self,
        model_path: ModelPath,
        id: Identifier,
        error: SubmodelTestE,
    ) {
        self.error_map
            .entry(model_path)
            .or_insert_with(ModelMapEntryBuilder::new)
            .add_submodel_test_data(id, error);
    }
}

impl<
    PyT,
    PyE,
    SubmodelT,
    SubmodelE,
    ParamT,
    ParamE,
    ModelTestT,
    ModelTestE,
    SubmodelTestT,
    SubmodelTestE,
> TryInto<ModelMap<PyT, SubmodelT, ParamT, ModelTestT, SubmodelTestT>>
    for ModelMapBuilder<
        PyT,
        PyE,
        SubmodelT,
        SubmodelE,
        ParamT,
        ParamE,
        ModelTestT,
        ModelTestE,
        SubmodelTestT,
        SubmodelTestE,
    >
{
    type Error = (
        ModelMap<PyT, SubmodelT, ParamT, ModelTestT, SubmodelTestT>,
        ModelMap<PyE, SubmodelE, ParamE, ModelTestE, SubmodelTestE>,
    );

    fn try_into(
        self,
    ) -> Result<ModelMap<PyT, SubmodelT, ParamT, ModelTestT, SubmodelTestT>, Self::Error> {
        if self.error_map.is_empty() {
            let map: HashMap<_, _> = self.map.into_iter().map(|(k, v)| (k, v.into())).collect();

            Ok(ModelMap::new(map))
        } else {
            let map: HashMap<_, _> = self.map.into_iter().map(|(k, v)| (k, v.into())).collect();
            let error_map: HashMap<_, _> = self
                .error_map
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect();

            Err((ModelMap::new(map), ModelMap::new(error_map)))
        }
    }
}
