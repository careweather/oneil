# One CPython version, several 3.12 layouts

## Status

Superseded by [one CLI binary per Python minor version](./2026-09-23-cli-python-exec-loader.md).

## Context

The release `oneil` binary links against libpython (PyO3). It does not ship a Python install. On macOS the linker records an absolute path to the Python used at build time. A single GitHub binary therefore only runs against the layout it was built with.

The layouts people actually have are few and stable enough to publish:

- **Homebrew** `python@3.12` on Apple Silicon: `/opt/homebrew/opt/python@3.12/Frameworks/Python.framework/Versions/3.12/Python`
- **system** — python.org framework on macOS (`/Library/Frameworks/Python.framework/Versions/3.12/Python`), distro `libpython3.12` on Linux, `python312.dll` on PATH on Windows
- **uv** — `uv python install 3.12`, whose prefix is per-user and patch-versioned (`~/.local/share/uv/python/cpython-3.12.<patch>-…`)

Looking up libpython *inside* a running `oneil` is too late on macOS: dyld binds PyO3’s symbols at process load. A CI-baked uv absolute path will not match another machine. Rewriting load commands after download was rejected as product infrastructure.

## Decision

Support **one** CPython version: **3.12**. Publish one release archive per layout:

- macOS: `homebrew`, `system`, `uv`
- Linux and Windows: `system`, `uv`

Asset names are `oneil-{tag}-{triple}-{flavor}.{tar.gz|zip}`. Older unflavored 1.x names remain a fallback.

Homebrew and system binaries keep a stable absolute load command (or the Windows/Linux soname). The uv flavor is rewritten in CI to `@rpath/libpython3.12.dylib` (macOS) or a soname with no baked rpath (Linux). The extension and `actions/install-oneil` detect which layout is present (Homebrew, then uv, then system), download that flavor, and for uv set `DYLD_LIBRARY_PATH` / `LD_LIBRARY_PATH` / `PATH` **before** spawning `oneil`.

`./install.sh` still links against the builder’s 3.12 (`uv python find 3.12`, then Homebrew).

## Consequences

- Users do not need one prescribed installer. The extension picks the matching binary.
- uv release binaries need a library search path at launch. That is the extension’s job, not a lookup inside `oneil`.
- A new layout (pyenv, conda, …) is another flavor or a source build, not a version × method matrix.
- Newer python.org homepage versions (3.14, …) are not sufficient for the release CLI.
