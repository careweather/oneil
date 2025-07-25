use std::io::Write;

use oneil_model_loader::ModelErrorMap;

use crate::file_parser::{DoesNotExistError, LoadingError};

pub fn print(error_map: &ModelErrorMap<LoadingError, DoesNotExistError>, writer: &mut impl Write) {
    todo!()
}
