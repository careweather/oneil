// error: expected `foo`, found `bar`
//  --> test.on:5:12
//   |
// 5 | fn main() {
//   |            ^ expected `foo`
//   |
//   = note: expected `foo`
//   = note: found `bar`

use std::path::PathBuf;

use crate::printer::util::ColorChoice;

pub struct Error<'a> {
    path: PathBuf,
    message: String,
    location: Option<ErrorLocation<'a>>,
}

impl<'a> Error<'a> {
    pub fn new(path: PathBuf, message: String) -> Self {
        Self {
            path,
            message,
            location: None,
        }
    }

    pub fn new_with_contents(
        path: PathBuf,
        message: String,
        contents: &'a str,
        offset: usize,
    ) -> Self {
        let location = ErrorLocation::from_source_and_offset(contents, offset);
        Self {
            path,
            message,
            location: Some(location),
        }
    }

    pub fn to_string(&self, color_choice: &ColorChoice) -> String {
        let message_line = self.get_message_line(color_choice);
        let location_line = self.get_location_line(color_choice);
        let source_line = self.get_source_lines(color_choice);

        let mut lines = vec![message_line, location_line];
        if let Some(source_line) = source_line {
            lines.push(source_line);
        }

        lines.join("\n")
    }

    fn get_message_line(&self, color_choice: &ColorChoice) -> String {
        // message line
        // error: <message>
        let error = color_choice.bold_red("error");
        let message = &self.message;
        let message_line = format!("{}: {}", error, message);
        let message_line = color_choice.bold(&message_line);
        message_line
    }

    fn get_location_line(&self, color_choice: &ColorChoice) -> String {
        // location line (line and column are optional)
        //  --> <path>
        // OR
        //  --> <path> (line <line>, column <column>)
        let arrow = color_choice.bold_blue("-->");
        let path = self.path.display();
        let location_line = match &self.location {
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

    fn get_source_lines(&self, color_choice: &ColorChoice) -> Option<String> {
        // source line (if available)
        //   |
        // 1 | use foo bar
        //   |         ^
        //   |
        match &self.location {
            Some(location) => {
                let line = location.line();
                let column = location.column();
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

                let start_blank_line = format!("{} {} ", margin, bar);
                let source_line = format!("{} {} {}", line_label, bar, line_source);
                let pointer_line = format!("{} {} {}{}", margin, bar, pointer_indent, pointer);
                let end_blank_line = start_blank_line.clone();

                let source_lines =
                    vec![start_blank_line, source_line, pointer_line, end_blank_line];
                let source_lines = source_lines.join("\n");

                Some(source_lines)
            }
            None => None,
        }
    }
}

impl<'a> std::fmt::Display for Error<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string(&ColorChoice::DisableColors))
    }
}

// note that line and column are 1-indexed
struct ErrorLocation<'a> {
    source: &'a str,
    offset: usize,
    line: usize,
    column: usize,
}

impl<'a> ErrorLocation<'a> {
    pub fn from_source_and_offset(source: &'a str, offset: usize) -> Self {
        // Find the offset of the first newline before the given offset.
        // The beginning of the file (offset 0) is assumed if there is no
        // newline before the offset.
        let line_start = source[..offset]
            .rfind('\n')
            .map_or(0, |newline_idx| newline_idx + 1);

        // The column is the offset of the error from the beginning of the line
        // (+ 1 because the column is 1-indexed)
        let column = offset - line_start + 1;

        let line = source[..offset].lines().count();

        Self {
            source,
            offset,
            line,
            column,
        }
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }

    pub fn line_source(&self) -> &str {
        let line_start = self.source[..self.offset].rfind('\n').unwrap_or(0);
        let line_end = self.source[self.offset..]
            .find('\n')
            .unwrap_or(self.source.len());
        &self.source[line_start..line_end]
    }
}
