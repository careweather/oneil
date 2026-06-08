//! Workspace discovery and bulk loading for the LSP server.

use std::path::{Path, PathBuf};

use indexmap::IndexSet;
use oneil_runtime::Runtime;
use oneil_shared::paths::ModelPath;

/// Directory names skipped while scanning a workspace for Oneil source files.
const SKIP_DIR_NAMES: &[&str] = &[".git", "node_modules", "target", ".worktrees"];

/// Returns every `.on` and `.one` file under `workspace_roots`.
///
/// Roots are scanned independently. Hidden directories and [`SKIP_DIR_NAMES`]
/// are not descended into. Unreadable directories are skipped without failing
/// the whole scan.
#[must_use]
pub fn discover_model_paths(workspace_roots: &[PathBuf]) -> IndexSet<ModelPath> {
    let mut paths = IndexSet::new();

    for root in workspace_roots {
        if root.is_dir() {
            collect_model_paths_under(root, &mut paths);
        }
    }

    paths
}

/// Loads source for every model file under `workspace_roots` into `runtime`.
pub fn load_workspace_models(runtime: &mut Runtime, workspace_roots: &[PathBuf]) -> usize {
    discover_model_paths(workspace_roots)
        .into_iter()
        .filter(|model_path| {
            let (_visited_paths, errors) = runtime.check_model(model_path);
            errors.is_empty()
        })
        .count()
}

/// Recursively collects Oneil model paths under `dir` into `paths`.
fn collect_model_paths_under(dir: &Path, paths: &mut IndexSet<ModelPath>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            if should_skip_dir(&path) {
                continue;
            }

            collect_model_paths_under(&path, paths);
        } else if let Ok(model_path) = ModelPath::try_from(path.as_path()) {
            paths.insert(model_path);
        }
    }
}

/// Returns `true` when `dir` should not be scanned for model files.
fn should_skip_dir(dir: &Path) -> bool {
    let Some(name) = dir.file_name().and_then(|n| n.to_str()) else {
        return true;
    };

    SKIP_DIR_NAMES.contains(&name) || name.starts_with('.')
}
