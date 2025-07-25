// error: expected `foo`, found `bar`
//  --> test.on:5:12
//   |
// 5 | fn main() {
//   |            ^ expected `foo`
//   |
//   = note: expected `foo`
//   = note: found `bar`

use crate::printer::util::ColorChoice;

pub struct Error {
    message: String,
}

impl Error {
    pub fn builder() -> ErrorBuilder {
        ErrorBuilder::new()
    }

    pub fn to_string(&self, color_choice: &ColorChoice) -> String {
        let error = color_choice.red("error");
        let message = &self.message;
        let message_line = format!("{}: {}", error, message);
        let message_line = color_choice.bold(&message_line);
        message_line
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string(&ColorChoice::DisableColors))
    }
}

pub struct ErrorBuilder {
    message: Option<String>,
}

impl ErrorBuilder {
    pub fn new() -> Self {
        Self { message: None }
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    pub fn build(self) -> Error {
        let message = self.message.expect("message is required");
        Error { message }
    }
}
