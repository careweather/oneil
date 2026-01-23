use owo_colors::Style;

// Error styles
pub const ERROR_COLOR: Style = Style::new().red();
pub const NOTE_COLOR: Style = Style::new().blue();
pub const HELP_COLOR: Style = Style::new().blue();
pub const SOURCE_ANNOTATION: Style = Style::new().blue().bold();

// Model output styles
pub const MODEL_LABEL: Style = Style::new().blue();
pub const TESTS_LABEL: Style = Style::new().green();
pub const TESTS_PASS_COLOR: Style = Style::new().green().bold();
pub const TESTS_FAIL_COLOR: Style = Style::new().red().bold();
pub const TEST_EXPR_LABEL: Style = Style::new().bold();
pub const TEST_EXPR_STR: Style = Style::new();
pub const NO_PARAMETERS_MESSAGE: Style = Style::new().italic().dimmed();
pub const MODEL_PATH_HEADER: Style = Style::new().blue().bold();
pub const PARAMETERS_NAME_LABEL: Style = Style::new().blue().bold();
pub const PARAMETER_IDENTIFIER: Style = Style::new().green();
pub const PARAMETER_LABEL: Style = Style::new().dimmed();
pub const PARAMETER_UNIT: Style = Style::new().blue();
pub const TREE_VALUE_NAME: Style = Style::new().green();
pub const TREE_VALUE_EQUATION: Style = Style::new().dimmed();

// Builtin documentation styles
pub const BUILTIN_SECTION_HEADER: Style = Style::new().blue().bold();
pub const BUILTIN_NAME: Style = Style::new().green().bold();
pub const BUILTIN_DESCRIPTION: Style = Style::new().dimmed();
pub const BUILTIN_ALIASES: Style = Style::new();
pub const BUILTIN_FUNCTION_ARGS: Style = Style::new().blue();
pub const BUILTIN_VALUE: Style = Style::new().cyan();
pub const BUILTIN_NOT_FOUND: Style = Style::new().bold();
