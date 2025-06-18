use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Oneil language CLI
#[derive(Parser)]
#[command(name = "oneil")]
#[command(about = "Oneil language tooling", long_about = None)]
pub struct Cli {
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
    },
}
