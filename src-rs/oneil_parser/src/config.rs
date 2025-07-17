/// Configuration for the Oneil parser.
///
/// This struct allows for customization of parser behavior. Currently, the
/// configuration is minimal but provides a foundation for future parser
/// customization options such as parser version selection and language feature
/// toggles.
///
/// # Examples
///
/// ```rust
/// use oneil_parser::Config;
///
/// // Use default configuration
/// let config = Config::default();
///
/// // Create a new configuration (currently identical to default)
/// let config = Config::new();
/// ```
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Config {}

impl Config {
    /// Creates a new configuration with default settings.
    ///
    /// Currently returns the same configuration as `Default::default()`.
    /// This method provides a more explicit way to create configurations
    /// and will be extended as new configuration options are added.
    ///
    /// # Returns
    ///
    /// A new `Config` instance with default settings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oneil_parser::Config;
    ///
    /// let config = Config::new();
    /// assert_eq!(config, Config::default());
    /// ```
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Config {
    /// Creates a default configuration.
    ///
    /// Returns a configuration with all default settings. This is the
    /// recommended configuration for most use cases.
    ///
    /// # Returns
    ///
    /// A `Config` instance with default settings.
    fn default() -> Self {
        Self {}
    }
}
