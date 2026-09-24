//! Record the `CPython` minor version this loader must exec against.
//!
//! `PyO3` links `oneil-runner` to one minor version. The loader looks up that
//! same version at runtime. `PYO3_PYTHON` selects the interpreter for both.

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=PYO3_PYTHON");

    let python = std::env::var("PYO3_PYTHON").unwrap_or_else(|_| "python3".to_string());
    let output = Command::new(&python)
        .args([
            "-c",
            "import sys; print(f'{sys.version_info[0]}.{sys.version_info[1]}')",
        ])
        .output();

    let output = match output {
        Ok(output) if output.status.success() => output,
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!("PYO3_PYTHON ({python}) failed while reading its version: {stderr}");
        }
        Err(error) => {
            panic!("could not run {python} to read the Python minor version: {error}");
        }
    };

    let minor = String::from_utf8_lossy(&output.stdout);
    let minor = minor.trim();
    let Some(("3", patch)) = minor.split_once('.') else {
        panic!("expected a Python 3 minor version from {python}, got {minor:?}");
    };
    assert!(
        !patch.is_empty() && patch.bytes().all(|byte| byte.is_ascii_digit()),
        "expected a Python 3 minor version from {python}, got {minor:?}"
    );

    println!("cargo:rustc-env=ONEIL_PYTHON_MINOR={minor}");
}
