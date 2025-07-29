//! Command-line interface definitions for the Oneil CLI
//!
//! This module defines the command-line argument parsing structure using the `clap` crate.
//! It provides a hierarchical command structure with development tools for debugging
//! and analyzing Oneil source files.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Oneil language CLI - Main command-line interface structure
///
/// This struct represents the top-level command-line interface for the Oneil tool.
/// It uses `clap` for argument parsing and provides a hierarchical command structure.
#[derive(Parser)]
#[command(name = "oneil")]
#[command(version, about = "Oneil language tooling", long_about = None)]
pub struct CliCommand {
    /// The subcommand to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available top-level commands for the Oneil CLI
///
/// Currently supports development tools for debugging and testing Oneil source files.
#[derive(Subcommand)]
pub enum Commands {
    /// Development tools for debugging and testing Oneil source files
    ///
    /// Provides utilities for parsing, analyzing, and displaying Oneil code
    /// in various formats for development and debugging purposes.
    Dev {
        /// The specific development command to execute
        #[command(subcommand)]
        command: DevCommands,
    },
}

/// Development-specific commands for the Oneil CLI
///
/// These commands are designed for developers working with Oneil source files,
/// providing tools for syntax analysis, error debugging, and code inspection.
#[derive(Subcommand)]
pub enum DevCommands {
    /// Print the Abstract Syntax Tree (AST) of a Oneil source file
    ///
    /// Parses the specified file and displays its AST in a hierarchical tree format.
    /// Useful for understanding the structure of Oneil code and debugging parsing issues.
    PrintAst {
        /// Path to the Oneil source file to parse and display
        #[arg(value_name = "FILE")]
        file: PathBuf,

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
    ///
    /// Loads and processes the specified file to generate the IR, then displays
    /// it in a hierarchical format. The IR represents the processed model after
    /// resolution of imports, parameters, and references.
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
        ///
        /// When enabled, displays the raw debug representation of the IR instead
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
}
