use std::{
    io::{Error as IoError, Write},
    path::Path,
};

use crate::printer::{error::util::Error, util::ColorChoice};

pub fn print(path: &Path, error: &IoError, color_choice: &ColorChoice, writer: &mut impl Write) {
    let message = format!("couldn't read `{}` - {}", path.display(), error);
    let error = Error::builder().with_message(message).build();
    let error = error.to_string(color_choice);
    writeln!(writer, "{}", error);
}
