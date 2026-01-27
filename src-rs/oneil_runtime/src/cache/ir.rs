use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use indexmap::IndexMap;
use oneil_ir as ir;
use oneil_shared::error::OneilError;

pub struct IrCache {
    ir_collection: ir::ModelCollection,
    errors: HashMap<PathBuf, Vec<OneilError>>,
}

impl IrCache {
    pub fn new() -> Self {
        Self {
            ir_collection: ir::ModelCollection::new(HashSet::new(), IndexMap::new()),
            errors: HashMap::new(),
        }
    }

    pub fn insert_ir(&mut self, ir: ir::ModelCollection) -> &ir::ModelCollection {
        self.ir_collection = self.ir_collection.merge(&ir);

        &self.ir_collection
    }

    pub fn insert_errors(&mut self, path: PathBuf, errors: Vec<OneilError>) -> &[OneilError] {
        self.errors.insert(path.clone(), errors);

        self.errors
            .get(&path)
            .expect("errors should be in cache after insertion")
    }

    pub fn get(&self, path: impl AsRef<Path>) -> Option<Result<&ir::Model, &[OneilError]>> {
        let path = path.as_ref().to_path_buf();

        self.errors
            .get(&path)
            .map(|result| Err(result.as_ref()))
            .or_else(|| {
                let path = ir::ModelPath::new(path);
                self.ir_collection.get_models().get(&path).map(Ok)
            })
    }

    pub fn contains_model(&self, path: &PathBuf) -> bool {
        self.get(path).is_some()
    }
}
