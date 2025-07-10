use std::collections::HashMap;

use oneil_ir::reference::{Identifier, ModelPath};

pub struct ModelMap<T>(HashMap<ModelPath, HashMap<Identifier, T>>);

impl<T> ModelMap<T> {
    pub fn new(map: HashMap<ModelPath, HashMap<Identifier, T>>) -> Self {
        Self(map)
    }

    pub fn get_model_data(&self, model_path: &ModelPath) -> Option<&HashMap<Identifier, T>> {
        self.0.get(model_path)
    }

    pub fn get_parameter_data(
        &self,
        model_path: &ModelPath,
        parameter_name: &Identifier,
    ) -> Option<&T> {
        self.0
            .get(model_path)
            .and_then(|model_data| model_data.get(parameter_name))
    }
}

pub struct ModelMapBuilder<T>(HashMap<ModelPath, HashMap<Identifier, T>>);

impl<T> ModelMapBuilder<T> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_model_data(&mut self, model_path: ModelPath, data: HashMap<Identifier, T>) {
        assert!(
            !self.0.contains_key(&model_path),
            "Model path already exists"
        );
        self.0.insert(model_path, data);
    }

    pub fn add_parameter_data(
        &mut self,
        model_path: ModelPath,
        parameter_name: Identifier,
        data: T,
    ) {
        self.0
            .entry(model_path)
            .or_insert_with(HashMap::new)
            .insert(parameter_name, data);
    }

    pub fn build(self) -> ModelMap<T> {
        ModelMap(self.0)
    }
}
