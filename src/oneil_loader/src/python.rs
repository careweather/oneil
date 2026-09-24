//! Find the `CPython` install this `oneil` binary was linked against.

use std::env;
use std::ffi::{OsStr, OsString};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

/// Minor version baked in by `build.rs` (`3.12`, `3.14`, …).
pub const PYTHON_MINOR: &str = env!("ONEIL_PYTHON_MINOR");

const PROBE_SCRIPT: &str = r#"
import json, os, sys, sysconfig
minor = f"{sys.version_info.major}.{sys.version_info.minor}"
libdir = sysconfig.get_config_var("LIBDIR") or ""
names = []
for key in ("INSTSONAME", "LDLIBRARY"):
    name = sysconfig.get_config_var(key)
    if name and name not in names:
        names.append(name)
candidates = []
if sys.platform == "win32":
    dll = f"python{sys.version_info.major}{sys.version_info.minor}.dll"
    candidates.append(os.path.join(os.path.dirname(sys.executable), dll))
if libdir:
    for name in names:
        candidates.append(os.path.join(libdir, name))
framework_prefix = sysconfig.get_config_var("PYTHONFRAMEWORKPREFIX")
if framework_prefix:
    candidates.append(os.path.join(
        framework_prefix, "Python.framework", "Versions", minor, "Python"))
site = None
if sys.prefix != sys.base_prefix:
    if sys.platform == "win32":
        site = os.path.join(sys.prefix, "Lib", "site-packages")
    else:
        site = os.path.join(sys.prefix, "lib", f"python{minor}", "site-packages")
print(json.dumps({
    "minor": minor,
    "base_prefix": sys.base_prefix,
    "candidates": candidates,
    "site_packages": site,
}))
"#;

/// An installed interpreter whose shared library matches [`PYTHON_MINOR`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonLayout {
    /// Prefix that holds this interpreter's standard library.
    pub base_prefix: PathBuf,
    /// `site-packages` inside a virtual environment, when the interpreter is one.
    pub site_packages: Option<PathBuf>,
    /// Shared library file to put on the dynamic loader path.
    pub library: PathBuf,
}

#[derive(Debug, Deserialize)]
struct ProbeJson {
    minor: String,
    base_prefix: PathBuf,
    candidates: Vec<PathBuf>,
    site_packages: Option<PathBuf>,
}

/// How to start a Python process.
#[derive(Debug, Clone)]
struct PythonInvoke {
    program: OsString,
    prefix_args: Vec<OsString>,
}

impl PythonInvoke {
    fn new(program: impl Into<OsString>) -> Self {
        Self {
            program: program.into(),
            prefix_args: Vec::new(),
        }
    }

    fn with_prefix(
        program: impl Into<OsString>,
        prefix_args: impl IntoIterator<Item = impl Into<OsString>>,
    ) -> Self {
        Self {
            program: program.into(),
            prefix_args: prefix_args.into_iter().map(Into::into).collect(),
        }
    }
}

/// Locate the interpreter for [`PYTHON_MINOR`].
///
/// `ONEIL_PYTHON` must be that version. Other candidates are skipped when they
/// are a different version. Returns `Ok(None)` when nothing matching is installed
/// so the runner can still start from an rpath baked in by the linker (Nix).
///
/// # Errors
///
/// Returns an error when `ONEIL_PYTHON` is set and is missing or a different minor version.
pub fn find_python() -> io::Result<Option<PythonLayout>> {
    if let Some(explicit) = env::var_os("ONEIL_PYTHON") {
        let invoke = PythonInvoke::new(explicit);
        return probe(&invoke).map_or_else(
            || {
                Err(io::Error::other(format!(
                    "ONEIL_PYTHON ({}) is not Python {PYTHON_MINOR}",
                    invoke.program.to_string_lossy(),
                )))
            },
            |layout| Ok(Some(layout)),
        );
    }

    for invoke in candidates() {
        if let Some(layout) = probe(&invoke) {
            return Ok(Some(layout));
        }
    }
    Ok(None)
}

/// Search order: active venv, `pythonX.Y` and `python3` on `PATH`, uv, then Homebrew and python.org.
fn candidates() -> Vec<PythonInvoke> {
    let mut found = Vec::new();
    if let Some(venv) = env::var_os("VIRTUAL_ENV") {
        let root = PathBuf::from(venv);
        let program = if cfg!(windows) {
            root.join("Scripts").join("python.exe")
        } else {
            root.join("bin").join("python")
        };
        found.push(PythonInvoke::new(program));
    }

    found.push(PythonInvoke::new(format!("python{PYTHON_MINOR}")));
    found.push(PythonInvoke::new("python3"));
    if cfg!(windows) {
        found.push(PythonInvoke::with_prefix(
            "py",
            [format!("-{PYTHON_MINOR}")],
        ));
    }

    if let Some(uv_python) = uv_python() {
        found.push(PythonInvoke::new(uv_python));
    }

    let homebrew_roots = [
        PathBuf::from(format!("/opt/homebrew/opt/python@{PYTHON_MINOR}")),
        PathBuf::from(format!("/usr/local/opt/python@{PYTHON_MINOR}")),
    ];
    for root in homebrew_roots {
        found.push(PythonInvoke::new(
            root.join("bin").join(format!("python{PYTHON_MINOR}")),
        ));
    }
    found.push(PythonInvoke::new(format!(
        "/Library/Frameworks/Python.framework/Versions/{PYTHON_MINOR}/bin/python{PYTHON_MINOR}"
    )));
    found
}

fn uv_python() -> Option<PathBuf> {
    let output = Command::new("uv")
        .args(["python", "find", PYTHON_MINOR])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8(output.stdout).ok()?;
    let path = path.trim();
    if path.is_empty() {
        None
    } else {
        Some(PathBuf::from(path))
    }
}

fn probe(invoke: &PythonInvoke) -> Option<PythonLayout> {
    let output = Command::new(&invoke.program)
        .args(&invoke.prefix_args)
        .args(["-c", PROBE_SCRIPT])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let parsed: ProbeJson = match serde_json::from_slice(&output.stdout) {
        Ok(parsed) => parsed,
        Err(error) => {
            let _ = writeln!(
                io::stderr(),
                "oneil: ignoring {}: probe output was not valid ({error})",
                invoke.program.to_string_lossy(),
            );
            return None;
        }
    };
    if parsed.minor != PYTHON_MINOR {
        return None;
    }
    let library = first_existing(&parsed.candidates)?;
    Some(PythonLayout {
        base_prefix: parsed.base_prefix,
        site_packages: parsed.site_packages.filter(|path| path.is_dir()),
        library,
    })
}

/// First path that names an existing file.
pub fn first_existing(candidates: &[PathBuf]) -> Option<PathBuf> {
    candidates.iter().find(|path| path.is_file()).cloned()
}

/// `libpython3.12.dylib`, the name macOS dyld loads from `@executable_path`.
#[cfg_attr(not(any(test, target_os = "macos")), expect(dead_code))]
pub fn macos_library_link_name(minor: &str) -> String {
    format!("libpython{minor}.dylib")
}

/// Put `dir` at the front of a `PATH`-style variable.
pub fn prepend_path(dir: &Path, existing: Option<&OsStr>) -> OsString {
    let mut joined = OsString::from(dir);
    let Some(existing) = existing else {
        return joined;
    };
    if existing.is_empty() {
        return joined;
    }
    let sep = if cfg!(windows) { ";" } else { ":" };
    joined.push(sep);
    joined.push(existing);
    joined
}

#[cfg(test)]
mod tests {
    use super::{first_existing, macos_library_link_name, prepend_path};
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn first_existing_skips_missing_paths() {
        let dir = std::env::temp_dir().join(format!("oneil-loader-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp dir");
        let present = dir.join("libpython.so");
        fs::write(&present, b"x").expect("write");
        let picked = first_existing(&[dir.join("missing.so"), present.clone()]);
        assert_eq!(picked, Some(present));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn macos_link_name_matches_the_baked_minor() {
        assert_eq!(macos_library_link_name("3.12"), "libpython3.12.dylib");
        assert_eq!(macos_library_link_name("3.14"), "libpython3.14.dylib");
    }

    #[test]
    fn prepend_path_keeps_the_previous_value() {
        let dir = PathBuf::from("/opt/py/lib");
        let joined = prepend_path(&dir, Some(std::ffi::OsStr::new("/usr/lib")));
        let text = joined.to_string_lossy();
        assert!(text.starts_with("/opt/py/lib"));
        assert!(text.contains("/usr/lib"));
    }
}
