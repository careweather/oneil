use std::path::Path;

use oneil_parser::error::ParserError;

use crate::convert_error::{Error, convert_file_error};

pub fn convert_all(path: &Path, parser_errors: &[ParserError]) -> Vec<Error> {
    let mut errors = Vec::new();

    let file_contents = std::fs::read_to_string(path);

    let file_contents = match file_contents {
        Ok(file_contents) => Some(file_contents),
        Err(e) => {
            // if for some reason we can't read the file, print the file reading error,
            // then print the details that we have about the error (without the file contents)
            errors.push(convert_file_error(path, &e));
            None
        }
    };

    for parser_error in parser_errors {
        let error = convert(path, file_contents.as_deref(), parser_error);
        errors.push(error);
    }

    errors
}

pub fn convert(path: &Path, file_contents: Option<&str>, error: &ParserError) -> Error {
    let message = error.to_string();
    let location = file_contents.map(|contents| (contents, error.error_offset));

    Error::new_from_offset(path.to_path_buf(), message, location)
}
