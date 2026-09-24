//! Binary entry point for the Oneil CLI.

#[cfg(feature = "rust-lib")]
fn main() {
    oneil_cli::main();
}

#[cfg(not(feature = "rust-lib"))]
fn main() {
    unimplemented!("Python library does not have a CLI");
}
