//! Binary entry point for the Oneil CLI.

#![expect(
    clippy::multiple_crate_versions,
    reason = "this isn't causing problems, and it's going to take time to fix"
)]

fn main() {
    oneil_cli::main();
}
