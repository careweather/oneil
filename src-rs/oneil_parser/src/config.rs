/// Configuration for the parser.
///
/// Currently, this doesn't do anything, but it allows for different versions of
/// the parser to exist and be used at the same time
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Config {}

impl Config {
    /// Creates a new configuration with the given options.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}
