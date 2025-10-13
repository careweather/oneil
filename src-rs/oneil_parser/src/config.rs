/// Configuration for the Oneil parser.
///
/// This struct allows for customization of parser behavior. Currently, the
/// configuration is minimal but provides a foundation for future parser
/// customization options such as parser version selection and language feature
/// toggles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {}

impl Config {
    /// Creates a new configuration with default settings.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl Default for Config {
    /// Creates a default configuration.
    fn default() -> Self {
        Self {}
    }
}
