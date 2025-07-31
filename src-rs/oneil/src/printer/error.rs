//! Error message formatting and display functionality
//!
//! This module provides functionality for formatting and displaying error messages
//! in a user-friendly format. It includes source location highlighting, color coding,
//! and structured error output that matches common compiler error formats.
//!
//! The error display format includes:
//! - Error message with color coding
//! - File path and line/column information
//! - Source code snippet with error highlighting
//! - Visual indicators pointing to the error location

use std::io::{self, Write};

use oneil_error::OneilError;

use crate::printer::ColorChoice;

/// Prints a formatted error message to the specified writer
///
/// Formats the error with appropriate color coding and structure, including
/// the error message, file location, and source code snippet with highlighting.
///
/// # Arguments
///
/// * `error` - The error to format and display
/// * `color_choice` - Color configuration for the output
/// * `writer` - The writer to output the formatted error to
///
/// # Returns
///
/// Returns `io::Result<()>` indicating success or failure of the write operation.
///
/// # Errors
///
/// Returns an error if writing to the writer fails.
pub fn print(error: &OneilError, color_choice: &ColorChoice, writer: &mut impl Write) -> io::Result<()> {
    let error_string = error_to_string(error, color_choice);
    writeln!(writer, "{}", error_string)?;

    Ok(())
}

/// Converts an error to a formatted string representation
///
/// Creates a complete error message string including the error message,
/// location information, and source code snippet with proper formatting.
///
/// # Arguments
///
/// * `error` - The error to format
/// * `color_choice` - Color configuration for the output
///
/// # Returns
///
/// Returns a formatted string representation of the error.
fn error_to_string(error: &OneilError, color_choice: &ColorChoice) -> String {
    let message_line = get_message_line(error, color_choice);
    let location_line = get_location_line(error, color_choice);
    let source_line = get_source_lines(error, color_choice);

    let mut lines = vec![message_line, location_line];
    if let Some(source_line) = source_line {
        lines.push(source_line);
    }

    lines.join("\n")
}

/// Formats the main error message line
///
/// Creates the primary error message line in the format "error: <message>"
/// with appropriate color coding.
///
/// # Arguments
///
/// * `error` - The error to format
/// * `color_choice` - Color configuration for the output
///
/// # Returns
///
/// Returns the formatted error message line as a string.
fn get_message_line(error: &OneilError, color_choice: &ColorChoice) -> String {
    // message line
    // error: <message>
    let error_str = color_choice.bold_red("error");
    let message = error.message();
    let message_line = format!("{}: {}", error_str, message);
    let message_line = color_choice.bold(&message_line);
    message_line
}

/// Formats the location information line
///
/// Creates the location line showing file path and optional line/column information
/// in the format " --> <path>" or " --> <path> (line <line>, column <column>)".
///
/// # Arguments
///
/// * `error` - The error to format
/// * `color_choice` - Color configuration for the output
///
/// # Returns
///
/// Returns the formatted location line as a string.
fn get_location_line(error: &OneilError, color_choice: &ColorChoice) -> String {
    // location line (line and column are optional)
    //  --> <path>
    // OR
    //  --> <path> (line <line>, column <column>)
    let arrow = color_choice.bold_blue("-->");
    let path = error.path().display();
    let location_line = match &error.location() {
        // This format is technically less readable than " --> <path> (line <line>, column <column>)"
        // but IDEs like VSCode and Cursor allow you to <ctrl> + click on the error to jump to the
        // location in the file. In addition, the line and column will be displayed in the source
        // code snippet.
        Some(location) => format!(
            " {} {}:{}:{}",
            arrow,
            path,
            location.line(),
            location.column()
        ),
        None => format!(" {} {}", arrow, path),
    };

    location_line
}

/// Formats the source code snippet with error highlighting
///
/// Creates a visual representation of the source code around the error location,
/// including line numbers, source code, and error indicators pointing to the
/// specific location of the error.
///
/// # Arguments
///
/// * `error` - The error to format
/// * `color_choice` - Color configuration for the output
///
/// # Returns
///
/// Returns `Some(String)` with the formatted source code snippet if location
/// information is available, or `None` if no location information exists.
fn get_source_lines(error: &OneilError, color_choice: &ColorChoice) -> Option<String> {
    // source line (if available)
    //   |
    // 1 | use foo bar
    //   |         ^
    //   |
    match &error.location() {
        Some(location) => {
            let line = location.line();
            let column = location.column();
            let length = location.length();
            let line_source = location.line_source();

            // The width of the left margin is based on the number of digits
            // required for the line number (`line.ilog10() + 1` tells us
            // how many digits are in the base 10 representation of the line
            // number).
            let margin_width = line.ilog10() + 1;
            let margin = " ".repeat(margin_width as usize);

            let bar = color_choice.bold_blue("|");

            let line_label = color_choice.bold_blue(&line.to_string());

            let pointer_indent_width = column - 1;
            let pointer_indent = " ".repeat(pointer_indent_width);

            let pointer = color_choice.bold_red("^");
            let pointer_rest = color_choice.bold_red(&"-".repeat(length - 1));

            let blank_line = format!("{} {} ", margin, bar);
            let source_line = format!("{} {} {}", line_label, bar, line_source);
            let pointer_line = format!(
                "{} {} {}{}{}",
                margin, bar, pointer_indent, pointer, pointer_rest
            );

            let source_lines = vec![blank_line, source_line, pointer_line];
            let source_lines = source_lines.join("\n");

            Some(source_lines)
        }
        None => None,
    }
}
