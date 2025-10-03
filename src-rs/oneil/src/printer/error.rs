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

use std::{
    io::{self, Write},
    path::Path,
};

use oneil_shared::{Context, ErrorLocation, OneilError};

use crate::printer::ColorChoice;

/// Prints a formatted error message to the specified writer
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
fn get_error_message_line(message: &str, color_choice: ColorChoice) -> String {
    get_message_line("error", ColorChoice::bold_red, message, color_choice)
}

/// Formats a note message line
fn get_note_message_line(message: &str, color_choice: ColorChoice) -> String {
    get_message_line("note", ColorChoice::bold_blue, message, color_choice)
}

/// Formats a help message line
fn get_help_message_line(message: &str, color_choice: ColorChoice) -> String {
    get_message_line("help", ColorChoice::bold_blue, message, color_choice)
}

/// Formats a message line with a colored prefix
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
fn get_location_line(
    path: &Path,
    location: Option<&ErrorLocation>,
    color_choice: ColorChoice,
) -> String {
    // location line (line and column are optional)
    //  --> <path>
    // OR
    //  --> <path>:<line>:<column>
    let arrow = color_choice.bold_blue("-->");
    let path = path.display();

    location.map_or_else(
        || format!(" {arrow} {path}"),
        |location| format!(" {arrow} {path}:{}:{}", location.line(), location.column()),
    )
}

/// Formats the source code snippet with error highlighting
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
