# One CLI binary per Python minor version

## Status

Accepted

## Context

The release CLI links `libpython` for one CPython minor version. Homebrew, python.org, a distro package, and uv each put that same library in a different place. Publishing a separate archive per layout meant seven archives, all locked to 3.12.

`dyld` on macOS binds the library named in the binary before any of the program's own code runs. An absolute framework path only matches the install it was linked against. `DYLD_LIBRARY_PATH` is dropped for a binary signed with the hardened runtime.

## Decision

Ship six archives: Linux, Windows, and Apple Silicon, each for Python 3.12 and for Python 3.14.

`oneil` does not link Python. It finds an install of the minor version `oneil-runner` was linked against, sets `PYTHONHOME` to that prefix, and replaces itself with `oneil-runner`.

- Linux: `LD_LIBRARY_PATH` includes the directory of `libpython3.x.so.1.0`. Release builds strip any rpath.
- Windows: that interpreter's directory is prepended to `PATH` so `python3x.dll` resolves.
- macOS: the runner's load command is `@executable_path/libpython3.x.dylib`. Before exec, `oneil` copies the runner into `~/Library/Caches/oneil` and symlinks that name at the discovered library. `dyld` resolves `@executable_path` from the staged copy.

`ONEIL_PYTHON` selects the interpreter when several installs of the same minor version exist.

## Consequences

- One archive runs against Homebrew, python.org, the distro package, or uv, for the minor version it was built for.
- A 3.12 archive does not run on 3.14, and the reverse. Those are two builds.
- The macOS launch does not depend on `DYLD_LIBRARY_PATH` for the runner's own `libpython` load.
- `oneil` and `oneil-runner` are installed next to each other. `cargo install` of only `src/oneil` installs the runner, not the loader.
