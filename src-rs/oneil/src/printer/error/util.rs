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
    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn builder() -> ErrorBuilder {
        ErrorBuilder::new()
    }

    pub fn to_string(&self, color_choice: ColorChoice) -> String {
        format!("error: {}", self.message)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string(ColorChoice::DisableColors))
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
