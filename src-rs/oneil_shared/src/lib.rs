use std::collections::HashMap;

use oneil_ir::reference::{Identifier, ModelPath};

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct ModelMapBuilder<T, E> {
    map: HashMap<ModelPath, HashMap<Identifier, T>>,
    errors: HashMap<ModelPath, HashMap<Identifier, E>>,
}

impl<T, E> ModelMapBuilder<T, E> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            errors: HashMap::new(),
        }
    }

    pub fn add_model_data(&mut self, model_path: ModelPath, data: HashMap<Identifier, T>) {
        assert!(
            !self.map.contains_key(&model_path),
            "Model path already exists"
        );
        self.map.insert(model_path, data);
    }

    pub fn add_parameter_data(
        &mut self,
        model_path: ModelPath,
        parameter_name: Identifier,
        data: T,
    ) {
        self.map
            .entry(model_path)
            .or_insert_with(HashMap::new)
            .insert(parameter_name, data);
    }

    pub fn add_parameter_error(
        &mut self,
        model_path: ModelPath,
        parameter_name: Identifier,
        error: E,
    ) {
        self.errors
            .entry(model_path)
            .or_insert_with(HashMap::new)
            .insert(parameter_name, error);
    }
}

impl<T, E> TryInto<ModelMap<T>> for ModelMapBuilder<T, E> {
    type Error = (ModelMap<T>, ModelMap<E>);

    fn try_into(self) -> Result<ModelMap<T>, Self::Error> {
        if self.errors.is_empty() {
            Ok(ModelMap(self.map))
        } else {
            Err((ModelMap(self.map), ModelMap(self.errors)))
        }
    }
}
