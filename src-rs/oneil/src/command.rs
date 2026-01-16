//! Command-line interface definitions for the Oneil CLI

use clap::{Parser, Subcommand};
use std::{fmt, path::PathBuf, str};

/// Oneil language CLI - Main command-line interface structure
#[derive(Parser)]
#[command(name = "oneil")]
#[command(version, about = "Oneil language tooling", long_about = None)]
pub struct CliCommand {
    /// The subcommand to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available top-level commands for the Oneil CLI
#[derive(Subcommand)]
pub enum Commands {
    /// Evaluate an Oneil model
    Eval {
        /// Path to the Oneil model file to evaluate
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Selects which parameters to print
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
        print_mode: PrintMode,

        /// Print debug information
        ///
        /// For parameters marked with `**`, this will print the
        /// values of variables used to evaluate the parameter.
        #[arg(long, short = 'd')]
        debug: bool,

        /// Only print info about the top model
        ///
        /// By default, Oneil will print the results of the top model
        /// and all of its submodels.
        #[arg(long)]
        top_only: bool,

        /// Disable colors in the output
        ///
        /// When enabled, suppresses colored output for better compatibility with
        /// terminals that don't support ANSI color codes or for redirecting to files.
        #[arg(long)]
        no_colors: bool,
    },
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
    /// Run the LSP
    Lsp {},
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

        /// Disable colors in the output
        ///
        /// When enabled, suppresses colored output for better compatibility with
        /// terminals that don't support ANSI color codes or for redirecting to files.
        #[arg(long)]
        no_colors: bool,
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

        /// Disable colors in the output
        ///
        /// When enabled, suppresses colored output for better compatibility with
        /// terminals that don't support ANSI color codes or for redirecting to files.
        #[arg(long)]
        no_colors: bool,
    },
    /// Print the results of evaluating an Oneil model
    ///
    /// This prints a debug representation, unlike the `Eval` command,
    /// which is intended to be used by end users.
    PrintModelResult {
        /// Path to the Oneil model file to evaluate
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Disable colors in the output
        ///
        /// When enabled, suppresses colored output for better compatibility with
        /// terminals that don't support ANSI color codes or for redirecting to files.
        #[arg(long)]
        no_colors: bool,
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
