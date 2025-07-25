use std::io::Error as IoError;
use std::path::Path;

use crate::printer::error::util::Error;

pub fn print(path: &Path, error: &IoError) {
    let message = format!("couldn't read `{}` - {}", path.display(), error);
    let error = Error::builder().with_message(message).build();

    println!("{}", error);
}
