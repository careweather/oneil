use oneil_model_loader::ModelErrorMap;

use crate::{
    convert_error::Error,
    file_parser::{DoesNotExistError, LoadingError},
};

pub fn convert_all(error_map: &ModelErrorMap<LoadingError, DoesNotExistError>) -> Vec<Error> {
    todo!()
}
