#![expect(
    clippy::multiple_crate_versions,
    reason = "serde's dependency tree duplicates crates the rest of the workspace already builds"
)]

//! Replace this process with `oneil-runner` after pointing it at the user's `CPython`.
//!
//! `oneil-runner` is linked to one Python minor version. On Linux and Windows the
//! loader puts that version's library directory on the loader path and sets
//! `PYTHONHOME`, then execs the runner. On macOS the runner's install name is
//! `@executable_path/libpython3.x.dylib`. dyld resolves that before the runner's
//! `main`, and it ignores `DYLD_LIBRARY_PATH` for a hardened binary, so the loader
//! stages a copy of the runner next to a symlink of that exact name.

mod python;

use std::env;
#[cfg(target_os = "macos")]
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

#[cfg(target_os = "macos")]
use crate::python::macos_library_link_name;
use crate::python::{PythonLayout, find_python, prepend_path};

/// Find Python, prepare the runner, and replace this process with it.
fn main() -> ExitCode {
    match launch() {
        Ok(code) => code,
        Err(error) => {
            let _ = writeln!(io::stderr(), "oneil: {error}");
            ExitCode::from(1)
        }
    }
}

/// Resolve `oneil-runner`, attach the matching `CPython`, and hand off.
///
/// # Errors
///
/// Returns an error when the runner binary is missing, `ONEIL_PYTHON` does not
/// match the linked minor version, or the staged macOS launch directory cannot be written.
fn launch() -> io::Result<ExitCode> {
    let runner = runner_path()?;
    let layout = find_python()?;
    #[cfg(target_os = "macos")]
    let program = match &layout {
        Some(layout) => stage_macos_runner(&runner, layout)?,
        None => runner,
    };
    #[cfg(not(target_os = "macos"))]
    let program = runner;
    let mut command = Command::new(&program);
    if let Some(layout) = &layout {
        apply_python_env(&mut command, layout);
    }
    command.args(env::args_os().skip(1));
    handoff(command)
}

/// `oneil-runner` sitting next to this loader.
///
/// # Errors
///
/// Returns an error when the current executable path has no parent directory or
/// the runner file is not there.
fn runner_path() -> io::Result<PathBuf> {
    let current = env::current_exe()?;
    let parent = current.parent().ok_or_else(|| {
        io::Error::other(format!(
            "cannot find the directory containing {}",
            current.display()
        ))
    })?;
    let name = if cfg!(windows) {
        "oneil-runner.exe"
    } else {
        "oneil-runner"
    };
    let runner = parent.join(name);
    if !runner.is_file() {
        return Err(io::Error::other(format!(
            "missing {} next to {}",
            runner.display(),
            current.display()
        )));
    }
    Ok(runner)
}

/// Environment the runner's dynamic linker and `Py_Initialize` read.
fn apply_python_env(command: &mut Command, layout: &PythonLayout) {
    command.env("PYTHONHOME", &layout.base_prefix);
    if let Some(site) = &layout.site_packages {
        let python_path = prepend_path(site, env::var_os("PYTHONPATH").as_deref());
        command.env("PYTHONPATH", python_path);
    }

    let lib_dir = layout
        .library
        .parent()
        .map_or_else(|| layout.library.clone(), Path::to_path_buf);
    if cfg!(target_os = "linux") {
        let path = prepend_path(&lib_dir, env::var_os("LD_LIBRARY_PATH").as_deref());
        command.env("LD_LIBRARY_PATH", path);
    } else if cfg!(target_os = "windows") {
        let path = prepend_path(&lib_dir, env::var_os("PATH").as_deref());
        command.env("PATH", path);
    } else if cfg!(target_os = "macos") {
        let path = prepend_path(&lib_dir, env::var_os("DYLD_LIBRARY_PATH").as_deref());
        command.env("DYLD_LIBRARY_PATH", path);
    }
}

/// `~/Library/Caches/oneil`, where the staged runner and library symlink live.
#[cfg(target_os = "macos")]
fn macos_cache_dir() -> io::Result<PathBuf> {
    let home = env::var_os("HOME").ok_or_else(|| io::Error::other("HOME is not set"))?;
    Ok(PathBuf::from(home).join("Library/Caches/oneil"))
}

/// Copy the runner into a writable cache directory beside `libpython3.x.dylib`.
///
/// `@executable_path` is the staged copy's directory, so the symlink is visible
/// to dyld without `DYLD_LIBRARY_PATH`. The library-path variable is still set
/// for dependencies of `libpython` that use `@loader_path`.
///
/// # Errors
///
/// Returns an error when the cache directory cannot be created or the runner cannot be copied.
#[cfg(target_os = "macos")]
fn stage_macos_runner(runner: &Path, layout: &PythonLayout) -> io::Result<PathBuf> {
    let cache = macos_cache_dir()?.join(python::PYTHON_MINOR);
    fs::create_dir_all(&cache)?;
    let staged = cache.join("oneil-runner");
    refresh_copy(runner, &staged)?;
    let link_name = macos_library_link_name(python::PYTHON_MINOR);
    let link = cache.join(link_name);
    replace_symlink(&layout.library, &link)?;
    Ok(staged)
}

/// Replace `dest` when `src` is newer or a different size.
#[cfg(target_os = "macos")]
fn refresh_copy(src: &Path, dest: &Path) -> io::Result<()> {
    if same_file_revision(src, dest) {
        return Ok(());
    }
    if dest.exists() {
        fs::remove_file(dest)?;
    }
    fs::copy(src, dest)?;
    let permissions = fs::metadata(src)?.permissions();
    fs::set_permissions(dest, permissions)?;
    if let Ok(modified) = fs::metadata(src)?.modified() {
        fs::File::options()
            .write(true)
            .open(dest)?
            .set_modified(modified)?;
    }
    Ok(())
}

/// True when `dest` already matches `src` in length and modification time.
#[cfg(target_os = "macos")]
fn same_file_revision(src: &Path, dest: &Path) -> bool {
    let Ok(src_meta) = fs::metadata(src) else {
        return false;
    };
    let Ok(dest_meta) = fs::metadata(dest) else {
        return false;
    };
    src_meta.len() == dest_meta.len() && src_meta.modified().ok() == dest_meta.modified().ok()
}

/// Point `link` at `target`, replacing a previous symlink or file.
#[cfg(target_os = "macos")]
fn replace_symlink(target: &Path, link: &Path) -> io::Result<()> {
    if link.symlink_metadata().is_ok() {
        fs::remove_file(link)?;
    }
    std::os::unix::fs::symlink(target, link)
}

/// Replace this process on Unix. On Windows, wait and return the runner's status.
fn handoff(mut command: Command) -> io::Result<ExitCode> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        Err(command.exec())
    }
    #[cfg(windows)]
    {
        let status = command.status()?;
        let code = status.code().unwrap_or(1);
        let code = u8::try_from(code).unwrap_or(1);
        Ok(ExitCode::from(code))
    }
}
