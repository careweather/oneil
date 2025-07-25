use std::io::Error as IoError;
use std::path::Path;

use crate::printer::error::util::Error;
use crate::printer::util::ColorChoice;

pub fn print(path: &Path, error: &IoError, color_choice: &ColorChoice) {
    let message = format!("couldn't read `{}` - {}", path.display(), error);
    let error = Error::builder().with_message(message).build();
    let error = error.to_string(color_choice);
    println!("{}", error);
}
