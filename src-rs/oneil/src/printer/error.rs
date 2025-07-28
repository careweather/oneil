use std::io::{self, Write};

use crate::{convert_error::Error, printer::ColorChoice};

pub fn print(error: &Error, color_choice: &ColorChoice, writer: &mut impl Write) -> io::Result<()> {
    let error_string = error_to_string(error, color_choice);
    writeln!(writer, "{}", error_string)?;

    Ok(())
}

fn error_to_string(error: &Error, color_choice: &ColorChoice) -> String {
    let message_line = get_message_line(error, color_choice);
    let location_line = get_location_line(error, color_choice);
    let source_line = get_source_lines(error, color_choice);

    let mut lines = vec![message_line, location_line];
    if let Some(source_line) = source_line {
        lines.push(source_line);
    }

    lines.join("\n")
}

fn get_message_line(error: &Error, color_choice: &ColorChoice) -> String {
    // message line
    // error: <message>
    let error_str = color_choice.bold_red("error");
    let message = error.message();
    let message_line = format!("{}: {}", error_str, message);
    let message_line = color_choice.bold(&message_line);
    message_line
}

fn get_location_line(error: &Error, color_choice: &ColorChoice) -> String {
    // location line (line and column are optional)
    //  --> <path>
    // OR
    //  --> <path> (line <line>, column <column>)
    let arrow = color_choice.bold_blue("-->");
    let path = error.path().display();
    let location_line = match &error.location() {
        Some(location) => format!(
            " {} {} (line {}, column {})",
            arrow,
            path,
            location.line(),
            location.column()
        ),
        None => format!(" {} {}", arrow, path),
    };

    location_line
}

fn get_source_lines(error: &Error, color_choice: &ColorChoice) -> Option<String> {
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
