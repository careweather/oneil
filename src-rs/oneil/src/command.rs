use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Oneil language CLI
#[derive(Parser)]
#[command(name = "oneil")]
#[command(version, about = "Oneil language tooling", long_about = None)]
pub struct CliCommand {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Development tools for debugging and testing
    Dev {
        #[command(subcommand)]
        command: DevCommands,
    },
}

#[derive(Subcommand)]
pub enum DevCommands {
    /// Print the AST of a file
    PrintAst {
        /// Path to the Oneil source file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Display partial AST even if there are errors
        #[arg(long)]
        display_partial: bool,

        /// Print the output in debug format
        #[arg(long)]
        print_debug: bool,

        /// Disable colors in the output
        #[arg(long)]
        no_colors: bool,
    },
    /// Print the intermediate representation of a file
    PrintIr {
        /// Path to the Oneil source file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Display partial AST even if there are errors
        #[arg(long)]
        display_partial: bool,

        /// Print the output in debug format
        #[arg(long)]
        print_debug: bool,

        /// Disable colors in the output
        #[arg(long)]
        no_colors: bool,
    },
}
