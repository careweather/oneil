//! Exports TypeScript bindings for Oneil JSON wire formats into
//! `packages/ts-interfaces/src/generated/`:
//!
//! - [`oneil_cli::json_test_report::TestReport`] (`oneil test --format json`)
//! - [`oneil_lsp::custom_requests::RenderedTree`] (LSP rendered view)
//!
//! Prefer the repo-root helper scripts:
//!
//! ```sh
//! ./scripts/generate-ts-interfaces.sh
//! ./scripts/check-ts-interfaces.sh
//! ```
//!
//! Or run this example directly:
//!
//! ```sh
//! cargo run --example export_ts_interfaces -p oneil_cli --features ts-bindings
//! ```

use std::{fs, path::PathBuf};

use oneil_cli::json_test_report::TestReport;
use oneil_lsp::custom_requests::RenderedTree;
use ts_rs::{Config, TS};

#[expect(
    clippy::unwrap_used,
    reason = "example / codegen tool; failure should abort the generate step"
)]
#[expect(
    clippy::print_stderr,
    reason = "progress message for a CLI codegen example"
)]
fn main() {
    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/ts-interfaces/src/generated");

    if out_dir.exists() {
        for entry in fs::read_dir(&out_dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|ext| ext == "ts") {
                fs::remove_file(path).unwrap();
            }
        }
    } else {
        fs::create_dir_all(&out_dir).unwrap();
    }

    // `.js` import extensions match consumer packages' `"type": "module"`
    // resolution; `number` for large ints matches existing call sites
    // (`offset` / `line` / `column` / `start` / `end` were always `number`).
    let config = Config::new()
        .with_out_dir(&out_dir)
        .with_import_extension(Some("js"))
        .with_large_int("number");

    // Shared leaves (`EvaluatedValue`, `Span`, …) are written once by each
    // root export; the second pass overwrites identical content.
    TestReport::export_all(&config).unwrap();
    RenderedTree::export_all(&config).unwrap();
    eprintln!("Wrote TypeScript bindings to {}", out_dir.display());
}
