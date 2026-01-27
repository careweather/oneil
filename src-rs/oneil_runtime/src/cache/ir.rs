use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use oneil_ir as ir;
use oneil_shared::error::OneilError;

pub struct IrCache {
    ir_collection: IndexMap<PathBuf, ir::Model>,
    errors: IndexMap<PathBuf, Vec<OneilError>>,
}

impl IrCache {
    pub fn new() -> Self {
        Self {
            ir_collection: IndexMap::new(),
            errors: IndexMap::new(),
        }
    }

    pub fn insert_ir(&mut self, ir: ir::ModelCollection) -> &IndexMap<PathBuf, ir::Model> {
        let ir_map = ir
            .into_map()
            .into_iter()
            .map(|(path, model)| (path.as_ref().to_path_buf(), model));

        self.ir_collection.extend(ir_map);

        &self.ir_collection
    }

    pub fn insert_errors(&mut self, path: PathBuf, errors: Vec<OneilError>) -> &[OneilError] {
        self.errors.insert(path.clone(), errors);

        self.errors
            .get(&path)
            .expect("errors should be in cache after insertion")
    }

    pub fn get_result(&self, path: impl AsRef<Path>) -> Option<Result<&ir::Model, &[OneilError]>> {
        let path = path.as_ref().to_path_buf();

        self.errors
            .get(&path)
            .map(|result| Err(result.as_ref()))
            .or_else(|| self.ir_collection.get(&path).map(Ok))
    }

    pub fn contains_result(&self, path: &PathBuf) -> bool {
        self.get_result(path).is_some()
    }

    pub fn get_model(&self, path: impl AsRef<Path>) -> Option<&ir::Model> {
        self.ir_collection.get(&path.as_ref().to_path_buf())
    }

    pub fn contains_model(&self, path: &PathBuf) -> bool {
        self.get_model(path).is_some()
    }

    pub fn get_errors(&self, path: impl AsRef<Path>) -> Option<&[OneilError]> {
        self.errors
            .get(&path.as_ref().to_path_buf())
            .map(|errors| errors.as_ref())
    }

    pub fn contains_errors(&self, path: &PathBuf) -> bool {
        self.get_errors(path).is_some()
    }

    pub fn ir_collection(&self) -> &IndexMap<PathBuf, ir::Model> {
        &self.ir_collection
    }
}
