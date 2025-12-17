//! Command-line interface definitions for the Oneil CLI

use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
    /// Development tools for debugging and testing Oneil source files
    Dev {
        /// The specific development command to execute
        #[command(subcommand)]
        command: DevCommand,
    },
    /// Evaluate a Oneil model
    Eval {
        /// Path to the Oneil model file to evaluate
        #[arg(value_name = "FILE")]
        file: PathBuf,

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
}

/// Development-specific commands for the Oneil CLI
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
}
