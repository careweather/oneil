use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use oneil_ir::{self as ir, ModelPath};
use oneil_shared::error::OneilError;

pub struct IrCache {
    ir_collection: IndexMap<ModelPath, ir::Model>,
    errors: IndexMap<ModelPath, Vec<OneilError>>,
}

impl IrCache {
    pub fn new() -> Self {
        Self {
            ir_collection: IndexMap::new(),
            errors: IndexMap::new(),
        }
    }

    pub fn insert_ir(&mut self, ir: ir::ModelCollection) -> &IndexMap<ModelPath, ir::Model> {
        let ir_map = ir.into_map();

        self.ir_collection.extend(ir_map);

        &self.ir_collection
    }

    pub fn insert_errors(&mut self, path: ModelPath, errors: Vec<OneilError>) -> &[OneilError] {
        self.errors.insert(path.clone(), errors);

        self.errors
            .get(&path)
            .expect("errors should be in cache after insertion")
    }

    pub fn get_result(&self, path: &ModelPath) -> Option<Result<&ir::Model, &[OneilError]>> {
        self.errors
            .get(path)
            .map(|result| Err(result.as_ref()))
            .or_else(|| self.ir_collection.get(path).map(Ok))
    }

    pub fn contains_result(&self, path: &ModelPath) -> bool {
        self.get_result(path).is_some()
    }

    pub fn get_model(&self, path: &ModelPath) -> Option<&ir::Model> {
        self.ir_collection.get(path)
    }

    pub fn contains_model(&self, path: &ModelPath) -> bool {
        self.get_model(path).is_some()
    }

    pub fn get_errors(&self, path: &ModelPath) -> Option<&[OneilError]> {
        self.errors.get(path).map(|errors| errors.as_ref())
    }

    pub fn contains_errors(&self, path: &ModelPath) -> bool {
        self.get_errors(path).is_some()
    }

    pub const fn ir_collection(&self) -> &IndexMap<ModelPath, ir::Model> {
        &self.ir_collection
    }
}
