use std::{
    io::{self, Error as IoError, Write},
    path::Path,
};

use crate::printer::{error::util::Error, util::ColorChoice};

pub fn print(
    path: &Path,
    error: &IoError,
    color_choice: &ColorChoice,
    writer: &mut impl Write,
) -> io::Result<()> {
    let message = format!("couldn't read `{}` - {}", path.display(), error);
    let error = Error::new(path.to_path_buf(), message);
    let error = error.to_string(color_choice);
    writeln!(writer, "{}", error)?;

    Ok(())
}
