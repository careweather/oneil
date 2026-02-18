//! Error message formatting and display functionality

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

use std::path::Path;

use anstream::eprintln;
use oneil_shared::error::{Context, ErrorLocation, OneilError};
use owo_colors::{OwoColorize, Style};

use crate::stylesheet;

/// Prints a formatted error message to the specified writer
pub fn print(error: &OneilError, show_internal_errors: bool) {
    if !error.is_internal_error() || show_internal_errors {
        let error_string = error_to_string(error);
        eprintln!("{error_string}");
    }
}

/// Converts an error to a formatted string representation
fn error_to_string(error: &OneilError) -> String {
    let message_line = get_error_message_line(error.message());
    let location_line = get_location_line(error.path(), error.location());
    let empty_line = String::new();
    let maybe_source_line = error.location().map_or_else(
        || get_context_lines(error.context(), 0),
        |location| get_source_and_context_lines(location, error.context(), stylesheet::ERROR_COLOR),
    );
    let context_with_source_lines =
        get_context_with_source_lines(error.path(), error.context_with_source());

    let mut lines = vec![message_line, location_line];
    lines.push(maybe_source_line);
    lines.push(empty_line);
    lines.extend(context_with_source_lines);

    lines.join("\n")
}

/// Formats the main error message line
fn get_error_message_line(message: &str) -> String {
    get_message_line("error", stylesheet::ERROR_COLOR, message)
}

/// Formats a note message line
fn get_note_message_line(message: &str) -> String {
    get_message_line("note", stylesheet::NOTE_COLOR, message)
}

/// Formats a help message line
fn get_help_message_line(message: &str) -> String {
    get_message_line("help", stylesheet::HELP_COLOR, message)
}

/// Formats a message line with a colored prefix
fn get_message_line(kind: &str, kind_color: Style, message: &str) -> String {
    // <kind>: <message>
    let kind_str = kind_color.style(kind);
    let rest = format!(": {message}").bold().to_string();
    let message_line = format!("{kind_str}{rest}");

    message_line.bold().to_string()
}

/// Formats the location information line
fn get_location_line(path: &Path, location: Option<&ErrorLocation>) -> String {
    // location line (line and column are optional)
    //  --> <path>
    // OR
    //  --> <path>:<line>:<column>
    let arrow = stylesheet::SOURCE_ANNOTATION.style("-->");
    let path = path.display();

    location.map_or_else(
        || format!(" {arrow} {path}"),
        |location| format!(" {arrow} {path}:{}:{}", location.line(), location.column()),
    )
}

/// Formats the source code snippet with error highlighting
fn get_source_and_context_lines(
    location: &ErrorLocation,
    context: &[Context],
    code_highlight_color: Style,
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

    let bar = stylesheet::SOURCE_ANNOTATION.style("|");

    let line_label = stylesheet::SOURCE_ANNOTATION.style(line.to_string());

    let pointer_indent_width = column - 1;
    let pointer_indent = " ".repeat(pointer_indent_width);

    let pointer = code_highlight_color.bold().style("^");
    let pointer_rest = code_highlight_color.bold().style("-".repeat(length - 1));

    let blank_line = format!("{margin} {bar} ");
    let source_line = format!("{line_label} {bar} {line_source}");
    let pointer_line = format!("{margin} {bar} {pointer_indent}{pointer}{pointer_rest}");

    let context_lines = get_context_lines(context, margin_width);

    [blank_line, source_line, pointer_line, context_lines].join("\n")
}

fn get_context_lines(context: &[Context], margin_width: u32) -> String {
    let margin = " ".repeat(margin_width as usize);

    context
        .iter()
        .map(|context| {
            let (equals, context_message) = match context {
                Context::Note(message) => {
                    let message = adjust_context_message_indent(message, margin_width);
                    (
                        stylesheet::NOTE_COLOR.bold().style("="),
                        get_note_message_line(&message),
                    )
                }
                Context::Help(message) => {
                    let message = adjust_context_message_indent(message, margin_width);
                    (
                        stylesheet::HELP_COLOR.bold().style("="),
                        get_help_message_line(&message),
                    )
                }
            };
            let context_line = format!("{margin} {equals} {context_message}");
            context_line
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Adjusts the indentation of a context message to match the margin width
fn adjust_context_message_indent(message: &str, margin_width: u32) -> String {
    // ______ = note: <message>
    // ^     ^  ^   ^
    // 1     2  3   4
    //
    // 1. margin_width
    // 2. equals + spaces
    // 3. kind
    // 4. colon + space
    let indent = margin_width + 3 + 4 + 2;
    let indent_str = " ".repeat(indent as usize);

    let newline_replacement = format!("\n{indent_str}");

    message.replace('\n', &newline_replacement)
}

fn get_context_with_source_lines(
    path: &Path,
    contexts: &[(Context, ErrorLocation)],
) -> Vec<String> {
    contexts
        .iter()
        .map(|(context, location)| {
            let (context_message, context_color) = match context {
                Context::Note(message) => (get_note_message_line(message), stylesheet::NOTE_COLOR),
                Context::Help(message) => (get_help_message_line(message), stylesheet::HELP_COLOR),
            };

            let location_line = get_location_line(path, Some(location));
            let source_lines = get_source_and_context_lines(location, &[], context_color);
            let empty_line = String::new();
            let mut lines = vec![context_message, location_line];
            lines.push(source_lines);
            lines.push(empty_line);
            lines.join("\n")
        })
        .collect()
}
