//! Command-line interface definitions for the Oneil CLI

use clap::{Args, Parser, Subcommand};
use std::{fmt, path::PathBuf, str};

/// Oneil language CLI - Main command-line interface structure
#[derive(Parser)]
#[command(name = "oneil")]
#[command(version, about = "Oneil language tooling", long_about = None)]
pub struct CliCommand {
    /// The subcommand to execute
    #[command(subcommand)]
    pub command: Commands,

    /// Disable colors in the output
    ///
    /// When enabled, suppresses colored output for better compatibility with
    /// terminals that don't support ANSI color codes or for redirecting to files.
    #[arg(long)]
    pub no_colors: bool,
}

/// Available top-level commands for the Oneil CLI
#[derive(Subcommand)]
pub enum Commands {
    /// Evaluate an Oneil model
    #[clap(visible_alias = "e")]
    Eval(EvalArgs),

    /// Run tests in an Oneil model
    #[clap(visible_alias = "t")]
    Test(TestArgs),

    /// Print the dependency or "requires" tree for one or more parameters
    Tree(TreeArgs),

    /// Print the builtins for the Oneil language
    Builtins {
        /// The builtins to print
        #[command(subcommand)]
        command: Option<BuiltinsCommand>,
    },

    /// Run the LSP
    Lsp {},

    /// Development tools for debugging and testing Oneil source files
    ///
    /// NOTE: because these commands are not intended for end users, they are hidden
    /// from the help output. However, they can still be used. See `oneil dev --help`
    /// for more information.
    #[clap(hide = true)]
    Dev {
        /// The specific development command to execute
        #[command(subcommand)]
        command: DevCommand,
    },
}

#[expect(
    clippy::struct_excessive_bools,
    reason = "this is a configuration struct for evaluating a model"
)]
#[derive(Args)]
pub struct EvalArgs {
    /// Path to the Oneil model file to evaluate
    #[arg(value_name = "FILE")]
    pub file: PathBuf,

    /// When provided, selects which parameters to print
    ///
    /// The value should be a comma-separated list of parameters. A parameter
    /// may have one or more submodels, separated by a dot. `p.submodel2.submodel1` means the
    /// parameter `p` in `submodel2`, which is in `submodel1`, which
    /// is in the top model.
    ///
    /// When provided, `--print-mode` and `--top-only` are ignored.
    ///
    /// Examples:
    ///
    /// - `--params a` - print the parameter `a` in the top model
    ///
    /// - `--params a,b,c.sub,d` - print the parameters `a`, `b`, and `d` in
    ///   the top model, and the parameter `c` in the submodel `sub`
    ///
    /// - `-p a.submodel2.submodel1` - print the parameter `a` in the submodel `submodel2` in
    ///   the submodel `submodel1` in the top model
    #[arg(long, short = 'p')]
    pub params: Option<VariableList>,

    /// Selects what mode to print the results in
    ///
    /// This can be one of:
    ///
    /// - `trace`: print parameters marked with `*` (trace parameters),
    ///   `**` (debug parameters), or `$` (performance parameters)
    ///
    /// - `perf`: print parameters marked with `$` (performance parameters) only
    ///
    /// - `all`: print all parameter values
    #[arg(long, short = 'm', default_value_t)]
    pub print_mode: PrintMode,

    /// Print debug information
    ///
    /// For parameters marked with `**`, this will print the
    /// values of variables used to evaluate the parameter.
    #[arg(long, short = 'D')]
    pub debug: bool,

    /// Watch files for changes and re-evaluate the model
    #[arg(long)]
    pub watch: bool,

    /// Print info about submodels as well as the top model
    ///
    /// By default, Oneil will only print the results of the top model.
    #[arg(long, short = 'r')]
    pub recursive: bool,

    /// Display partial results even if there are errors
    ///
    /// If errors occurred during evaluation, errors will be printed,
    /// then the partial results will be printed.
    #[arg(long)]
    pub partial: bool,

    /// Don't print the results header
    #[arg(long)]
    pub no_header: bool,

    /// Don't print the test report
    #[arg(long)]
    pub no_test_report: bool,

    /// Don't print the parameters
    ///
    /// Note that this overrides the `--params` and `--print-mode` options.
    #[arg(long)]
    pub no_parameters: bool,
}

#[derive(Args)]
pub struct TestArgs {
    /// Path to the Oneil model file to run tests in
    #[arg(value_name = "FILE")]
    pub file: PathBuf,

    /// Print submodel test results recursively
    ///
    /// By default, only the top model test results are printed. When enabled,
    /// submodel test results are also printed.
    #[arg(long)]
    pub recursive: bool,

    /// Don't print the results header
    #[arg(long)]
    pub no_header: bool,

    /// Don't print the test report
    #[arg(long)]
    pub no_test_report: bool,
}

#[derive(Args)]
pub struct TreeArgs {
    /// Path to the Oneil model file to print the tree for
    #[arg(value_name = "FILE")]
    pub file: PathBuf,

    /// The parameter to print the tree for
    #[arg(value_name = "PARAM", required = true)]
    pub params: Vec<String>,

    /// Print the tree of parameter references
    ///
    /// By default, the tree printed represents the dependencies
    /// of the provided parameters. When enabled, the tree instead
    /// represents parameters where the provided parameters are referenced.
    #[arg(long)]
    pub list_refs: bool,

    /// Print submodel values in the tree
    ///
    /// By default, only the top model values are included in the tree. When enabled,
    /// submodel values are also included in the tree.
    #[arg(long)]
    pub recursive: bool,

    /// Depth of the tree to print
    ///
    /// By default, the tree is printed to the full depth. When enabled,
    /// the tree is printed to the specified depth.
    #[arg(long)]
    pub depth: Option<usize>,

    /// Display partial trees even if there are errors
    ///
    /// If errors occurred during evaluation, errors will be printed,
    /// then the partial trees will be printed.
    #[arg(long)]
    pub partial: bool,
}

/// Available subcommands for the `Builtins` command
#[derive(Subcommand, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinsCommand {
    /// Print all the builtins
    #[command(name = "all")]
    All,

    /// Print the builtin units
    #[command(name = "unit")]
    Units,

    /// Print the builtin functions
    #[command(name = "func")]
    Functions,

    /// Print the builtin values
    #[command(name = "value")]
    Values,

    /// Print the builtin unit prefixes
    #[command(name = "prefix")]
    Prefixes,
}

/// Development-specific commands for the Oneil CLI
#[expect(
    clippy::enum_variant_names,
    reason = "the names are descriptive and just happen to start with the same word; in the future, other commands may be added that don't start with the same word"
)]
#[derive(Subcommand)]
pub enum DevCommand {
    /// Print the Abstract Syntax Tree (AST) of a Oneil source file
    PrintAst {
        /// Path to the Oneil source file(s) to parse and display
        #[arg(value_name = "FILE")]
        files: Vec<PathBuf>,

        /// Display partial AST even if there are parsing errors
        ///
        /// When enabled, shows the portion of the AST that was successfully
        /// parsed. Useful for debugging incomplete or malformed code.
        #[arg(long)]
        display_partial: bool,

        /// Print the output in debug format
        ///
        /// When enabled, displays the raw debug representation of the AST instead
        /// of the formatted tree structure. Useful for detailed internal analysis.
        #[arg(long)]
        print_debug: bool,
    },
    /// Print the Intermediate Representation (IR) of a Oneil source file
    PrintIr {
        /// Path to the Oneil source file to process and display
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Display partial IR even if there are loading errors
        ///
        /// When enabled, shows the portion of the IR that was successfully generated
        /// before encountering errors. Useful for debugging model loading issues.
        #[arg(long)]
        display_partial: bool,

        /// Print the output in debug format
        #[arg(long)]
        print_debug: bool,
    },
    /// Print the results of evaluating an Oneil model
    ///
    /// This prints a debug representation, unlike the `Eval` command,
    /// which is intended to be used by end users.
    PrintModelResult {
        /// Path to the Oneil model file to evaluate
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Display partial results even if there are errors
        ///
        /// When enabled, shows the portion of the results that were successfully generated
        /// before encountering errors. Useful for debugging model evaluation issues.
        #[arg(long)]
        display_partial: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrintMode {
    Trace,
    Performance,
    All,
}

impl Default for PrintMode {
    fn default() -> Self {
        Self::Trace
    }
}

impl str::FromStr for PrintMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "all" => Ok(Self::All),
            "trace" => Ok(Self::Trace),
            "perf" => Ok(Self::Performance),
            _ => Err("valid options are `all`, `trace`, or `perf`".to_string()),
        }
    }
}

impl fmt::Display for PrintMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::All => write!(f, "all"),
            Self::Trace => write!(f, "trace"),
            Self::Performance => write!(f, "perf"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableList(Vec<Variable>);

impl VariableList {
    pub fn iter(&self) -> impl Iterator<Item = &Variable> {
        self.0.iter()
    }
}

impl str::FromStr for VariableList {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let params = s
            .split(',')
            .filter_map(|s| (!s.is_empty()).then_some(s.trim().parse::<Variable>()))
            .collect::<Result<_, _>>()?;
        Ok(Self(params))
    }
}

impl fmt::Display for VariableList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(Variable::to_string)
                .collect::<Vec<_>>()
                .join(",")
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable(Vec<String>);

impl Variable {
    /// Splits the variable into a vector of strings.
    ///
    /// `param.submodel1.submodel2` becomes `["param", "submodel1", "submodel2"]`.
    pub fn to_vec(&self) -> Vec<String> {
        self.0.clone()
    }
}

impl str::FromStr for Variable {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.split('.').map(str::to_string).collect()))
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join("."))
    }
}
