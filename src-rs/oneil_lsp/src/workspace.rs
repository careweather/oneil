//! Workspace discovery and bulk loading for the LSP server.

use std::path::{Path, PathBuf};

use indexmap::IndexSet;
use oneil_runtime::Runtime;
use oneil_shared::paths::ModelPath;

/// Options controlling workspace model discovery at LSP startup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceDiscoveryOptions {
    /// When `false`, workspace roots are not scanned for model files.
    pub enabled: bool,
    /// Exact directory names that are not descended into during discovery.
    pub skip_dir_names: Vec<String>,
}

/// Returns every `.on` and `.one` file under `workspace_roots`.
///
/// Roots are scanned independently. Hidden directories and names listed in
/// `options.skip_dir_names` are not descended into. Unreadable directories are
/// skipped without failing the whole scan.
#[must_use]
pub fn discover_model_paths(
    workspace_roots: &[PathBuf],
    options: &WorkspaceDiscoveryOptions,
) -> IndexSet<ModelPath> {
    if !options.enabled {
        return IndexSet::new();
    }

    let mut paths = IndexSet::new();

    for root in workspace_roots {
        if root.is_dir() {
            collect_model_paths_under(root, options, &mut paths);
        }
    }

    paths
}

/// Loads source for every model file under `workspace_roots` into `runtime`.
pub fn load_workspace_models(
    runtime: &mut Runtime,
    workspace_roots: &[PathBuf],
    options: &WorkspaceDiscoveryOptions,
) -> usize {
    discover_model_paths(workspace_roots, options)
        .into_iter()
        .filter(|model_path| {
            let (_visited_paths, errors) = runtime.check_model(model_path);
            errors.is_empty()
        })
        .count()
}

/// Recursively collects Oneil model paths under `dir` into `paths`.
fn collect_model_paths_under(
    dir: &Path,
    options: &WorkspaceDiscoveryOptions,
    paths: &mut IndexSet<ModelPath>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            if should_skip_dir(&path, options) {
                continue;
            }

            collect_model_paths_under(&path, options, paths);
        } else if let Ok(model_path) = ModelPath::try_from(path.as_path()) {
            paths.insert(model_path);
        }
    }
}

/// Returns `true` when `dir` should not be scanned for model files.
fn should_skip_dir(dir: &Path, options: &WorkspaceDiscoveryOptions) -> bool {
    let Some(name) = dir.file_name().and_then(|n| n.to_str()) else {
        return true;
    };

    options.skip_dir_names.iter().any(|skip| skip == name) || name.starts_with('.')
}
