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

// TODO: create a unified function for the full message format so that errors with source and
//       context with source can be formatted reusing the same code
//
//       error: expected parameter or test
//        --> /home/careweather/Projects/veery/model/radar_chain.on:7:1
//         |
//       7 | PMA3-63GLN+ Gain: Gain_PMA3 = 25 : dB
//         | ^
//         = note: parameter labels must only contain the following characters: `a-z`, `A-Z`, `0-9`, `_`, `-`, `'`
//
//       note: invalid character found here
//        --> /home/careweather/Projects/veery/model/radar_chain.on:7:11
//         |
//       7 | PMA3-63GLN+ Gain: Gain_PMA3 = 25 : dB
//         |           ^
//

use std::{
    io::{self, Write},
    path::Path,
};

use oneil_error::{Context, ErrorLocation, OneilError};

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
pub fn print(
    error: &OneilError,
    color_choice: ColorChoice,
    writer: &mut impl Write,
) -> io::Result<()> {
    let error_string = error_to_string(error, color_choice);
    writeln!(writer, "{error_string}")?;

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
fn error_to_string(error: &OneilError, color_choice: ColorChoice) -> String {
    let message_line = get_error_message_line(error.message(), color_choice);
    let location_line = get_location_line(error.path(), error.location(), color_choice);
    let empty_line = String::new();
    let maybe_source_line = error
        .location()
        .map(|location| get_source_lines(location, error.context(), color_choice));
    let context_with_source_lines =
        get_context_with_source_lines(error.path(), error.context_with_source(), color_choice);

    let mut lines = vec![message_line, location_line];
    lines.extend(maybe_source_line);
    lines.push(empty_line);
    lines.extend(context_with_source_lines);

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
fn get_error_message_line(message: &str, color_choice: ColorChoice) -> String {
    get_message_line("error", ColorChoice::bold_red, message, color_choice)
}

/// Formats a note message line
///
/// Creates a note message line in the format "note: <message>" with blue coloring
/// for the "note" prefix.
///
/// # Arguments
///
/// * `message` - The note message to format
/// * `color_choice` - Color configuration for the output
///
/// # Returns
///
/// Returns the formatted note message line as a string.
fn get_note_message_line(message: &str, color_choice: ColorChoice) -> String {
    get_message_line("note", ColorChoice::bold_blue, message, color_choice)
}

/// Formats a help message line
///
/// Creates a help message line in the format "help: <message>" with blue coloring
/// for the "help" prefix.
///
/// # Arguments
///
/// * `message` - The help message to format
/// * `color_choice` - Color configuration for the output
///
/// # Returns
///
/// Returns the formatted help message line as a string.
fn get_help_message_line(message: &str, color_choice: ColorChoice) -> String {
    get_message_line("help", ColorChoice::bold_blue, message, color_choice)
}

/// Formats a message line with a colored prefix
///
/// Creates a message line in the format "<kind>: <message>" where the kind prefix
/// is colored according to the provided color function and the entire line is bold.
///
/// # Arguments
///
/// * `kind` - The prefix text (e.g. "error", "note", "help")
/// * `kind_color` - Function to apply color formatting to the kind prefix
/// * `message` - The message text to display
/// * `color_choice` - Color configuration for the output
///
/// # Returns
///
/// Returns the formatted message line as a string with appropriate coloring.
///
/// # Examples
///
/// ```
/// let message = get_message_line(
///     "error",
///     ColorChoice::bold_red,
///     "invalid syntax",
///     &color_choice
/// );
/// // Returns bold "error: invalid syntax" with red "error"
/// ```
fn get_message_line(
    kind: &str,
    kind_color: impl FnOnce(ColorChoice, &str) -> String,
    message: &str,
    color_choice: ColorChoice,
) -> String {
    // <kind>: <message>
    let kind_str = kind_color(color_choice, kind);
    let message_line = format!("{kind_str}: {message}");

    color_choice.bold(&message_line)
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
fn get_location_line(
    path: &Path,
    location: Option<&ErrorLocation>,
    color_choice: ColorChoice,
) -> String {
    // location line (line and column are optional)
    //  --> <path>
    // OR
    //  --> <path> (line <line>, column <column>)
    let arrow = color_choice.bold_blue("-->");
    let path = path.display();

    location.map_or_else(
        || format!(" {arrow} {path}"),
        |location| format!(" {arrow} {path}:{}:{}", location.line(), location.column()),
    )
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
fn get_source_lines(
    location: &ErrorLocation,
    context: &[Context],
    color_choice: ColorChoice,
) -> String {
    // source line (if available)
    //   |
    // 1 | use foo bar
    //   |         ^
    //   |
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

    let blank_line = format!("{margin} {bar} ");
    let source_line = format!("{line_label} {bar} {line_source}");
    let pointer_line = format!("{margin} {bar} {pointer_indent}{pointer}{pointer_rest}");

    let context_lines = context.iter().map(|context| {
        let equals = color_choice.bold_blue("=");
        let context_message = match context {
            Context::Note(message) => get_note_message_line(message, color_choice),
            Context::Help(message) => get_help_message_line(message, color_choice),
        };
        let context_line = format!("{margin} {equals} {context_message}");
        context_line
    });

    let mut source_lines = vec![blank_line, source_line, pointer_line];
    source_lines.extend(context_lines);

    source_lines.join("\n")
}

fn get_context_with_source_lines(
    path: &Path,
    contexts: &[(Context, ErrorLocation)],
    color_choice: ColorChoice,
) -> Vec<String> {
    contexts
        .iter()
        .map(|(context, location)| {
            let context_message = match context {
                Context::Note(message) => get_note_message_line(message, color_choice),
                Context::Help(message) => get_help_message_line(message, color_choice),
            };
            let location_line = get_location_line(path, Some(location), color_choice);
            let source_lines = get_source_lines(location, &[], color_choice);
            let empty_line = String::new();
            let mut lines = vec![context_message, location_line];
            lines.push(source_lines);
            lines.push(empty_line);
            lines.join("\n")
        })
        .collect()
}
