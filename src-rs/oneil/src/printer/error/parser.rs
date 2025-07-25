use std::{
    io::{self, Write},
    path::Path,
};

use oneil_parser::error::ParserError;

use crate::printer::{
    ColorChoice,
    error::{file::print as print_file_error, util::Error},
};

pub fn print_all(
    path: &Path,
    errors: &[ParserError],
    color_choice: &ColorChoice,
    writer: &mut impl Write,
) -> io::Result<()> {
    let file_contents = std::fs::read_to_string(path);

    let file_contents = match file_contents {
        Ok(file_contents) => Some(file_contents),
        Err(e) => {
            // if for some reason we can't read the file, print the file reading error,
            // then print the details that we have about the error (without the file contents)
            print_file_error(path, &e, color_choice, writer)?;
            None
        }
    };

    for error in errors {
        print(path, file_contents.as_deref(), error, color_choice, writer)?;
        writeln!(writer)?;
    }

    Ok(())
}

pub fn print(
    path: &Path,
    file_contents: Option<&str>,
    error: &ParserError,
    color_choice: &ColorChoice,
    writer: &mut impl Write,
) -> io::Result<()> {
    let message = error.to_string();

    let error = match file_contents {
        Some(contents) => {
            Error::new_with_contents(path.to_path_buf(), message, contents, error.error_offset)
        }
        None => Error::new(path.to_path_buf(), message),
    };

    let error_string = error.to_string(color_choice);
    writeln!(writer, "{}", error_string)?;

    Ok(())
}
