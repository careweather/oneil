use std::collections::{HashMap, HashSet};

use oneil_ir as ir;

use crate::util::context::lookup::{self, LookupResult};

pub struct ModelContext<'model> {
    models: &'model HashMap<ir::ModelPath, ir::Model>,
    model_errors: HashSet<&'model ir::ModelPath>,
}

impl<'model> ModelContext<'model> {
    #[must_use]
    pub const fn new(
        models: &'model HashMap<ir::ModelPath, ir::Model>,
        model_errors: HashSet<&'model ir::ModelPath>,
    ) -> Self {
        Self {
            models,
            model_errors,
        }
    }

    #[must_use]
    pub fn lookup_model(&self, model_path: &ir::ModelPath) -> ModelContextResult<'model> {
        let lookup_result = lookup::lookup_with(
            model_path,
            |model_path| self.models.get(model_path),
            |model_path| self.model_errors.contains(model_path),
        );

        ModelContextResult::from(lookup_result)
    }
}

pub enum ModelContextResult<'model> {
    Found(&'model ir::Model),
    HasError,
    NotFound,
}

impl<'model> From<LookupResult<&'model ir::Model>> for ModelContextResult<'model> {
    fn from(result: LookupResult<&'model ir::Model>) -> Self {
        match result {
            LookupResult::Found(model) => ModelContextResult::Found(model),
            LookupResult::HasError => ModelContextResult::HasError,
            LookupResult::NotFound => ModelContextResult::NotFound,
        }
    }
}
