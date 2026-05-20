# Installation

This section describes how to install the Oneil CLI (Rust implementation) on Linux, Windows, and macOS. Pre-built binaries are provided for these platforms via [GitHub Releases](https://github.com/careweather/oneil/releases).

## Prerequisites

- **Rust** (for building from source): [rustup](https://rustup.rs/) — install and ensure `cargo` is on your `PATH`.
- **gcc**
  - Install on Fedora/RHEL: `sudo dnf install gcc`
  - Install on Debian/Ubuntu: `sudo apt install build-essential`
- **Python 3.10+ with `pip`** (for importing Python functions in models and for the `oneil` Python package). Install Python development libraries when building from source (see below).
- **Python development libraries**
  - Install on Fedora/RHEL: `sudo dnf install python3-devel`
  - Install on Debian/Ubuntu: `sudo apt install python3-dev`

## Option 1: Download a release from GitHub (NOT AVAILABLE YET)

Pre-built binaries are published on the [Releases](https://github.com/careweather/oneil/releases) page for:

- **Linux** (x86_64, `unknown-linux-gnu`)
- **Windows** (x86_64, `pc-windows-msvc`)
- **macOS** (x86_64 and Apple Silicon, `apple-darwin`)

### Linux / macOS

1. Open the [latest release](https://github.com/careweather/oneil/releases/latest).
2. Download the archive for your OS and architecture (e.g. `oneil-v0.16.0-x86_64-unknown-linux-gnu.tar.gz`).
3. Unpack and put the `oneil` binary on your `PATH`:

   ```sh
   tar -xzf oneil-v*-x86_64-unknown-linux-gnu.tar.gz
   sudo mv oneil /usr/local/bin/
   # or, without sudo:
   mkdir -p ~/.local/bin && mv oneil ~/.local/bin/
   # ensure ~/.local/bin is in your PATH
   ```

   On macOS, use the appropriate archive (e.g. `oneil-v*-aarch64-apple-darwin.tar.gz` for Apple Silicon).

4. Confirm:

   ```sh
   oneil --version
   ```

### Windows

1. Open the [latest release](https://github.com/careweather/oneil/releases/latest).
2. Download the `.zip` for Windows (e.g. `oneil-v0.16.0-x86_64-pc-windows-msvc.zip`).
3. Unzip and either move `oneil.exe` into a directory on your `PATH`, or add the folder containing `oneil.exe` to your `PATH`.
4. Confirm in PowerShell or Command Prompt:

   ```cmd
   oneil --version
   ```

## Option 2: Install from source using the install script

From the repository root, the install script checks for **Cargo**, then installs the **Rust CLI** (`cargo install`) and, by default, the **Python package** (`pip install`) so you can run `oneil` and `import oneil`.

```sh
git clone https://github.com/careweather/oneil.git
cd oneil
./install.sh
```

- **Without Python** (CLI only, no bindings and no pip package): `./install.sh --no-python`
- **Editable Python install**: `./install.sh --editable` or `./install.sh -e`

On Windows, run `install.bat` from the repository root with the same flags (`--no-python`, `-e`, `--help`).

For the default install you also need **Python 3.10+** with pip and the development libraries.

## Option 3: Install from source with Cargo

Use this if you want the latest development version or need to customize the build.

1. Clone the repository:

   ```sh
   git clone https://github.com/careweather/oneil.git
   cd oneil
   ```

2. Build and install the `oneil` binary (requires Rust):

   ```sh
   cargo install --path src-rs/oneil
   ```

   This places the `oneil` executable in `~/.cargo/bin` (or `%USERPROFILE%\.cargo\bin` on Windows). Ensure that directory is on your `PATH`.

   Building from source requires Python 3.10+ development headers (see Prerequisites).

3. Confirm:

   ```sh
   oneil --version
   ```

## Option 4: Run from the repository (development)

For day-to-day development without installing:

```sh
git clone https://github.com/careweather/oneil.git
cd oneil
cargo build -p oneil
./target/debug/oneil --version
# or run directly:
cargo run -p oneil -- path/to/model.on
```

## Updating

Currently, there is no dedicated way to update Oneil. If you installed from source, update the source code with `git`, then re-install Oneil. If you downloaded a release from GitHub, download the new version and replace the previous `oneil` binary with the new one.

## Editor and tooling (optional)

- **Vim (currently unmaintained)**: See the [Vim support](https://github.com/careweather/oneil#vim-support) section in the main README for syntax highlighting.

- **VS Code / Cursor**: Install the [Oneil extension](https://marketplace.visualstudio.com/items?itemName=careweather.oneil) from the Marketplace for LSP and syntax highlighting.

## Install Oneil Python library

To install the `oneil` package into your current Python environment from a checkout of the repository, run `pip install .` from the **project root** (the directory that contains `pyproject.toml`):

```sh
git clone https://github.com/careweather/oneil.git
cd oneil
pip install .
```

After installation you can `import oneil` in Python.

> [!NOTE]
> `pip install .` alone does not install the CLI. Use **Option 2** (`./install.sh`) or **Option 3** (`cargo install --path src-rs/oneil`) if you want both the CLI and the library.

## Uninstalling Oneil

If Oneil was installed as a release binary, delete the release binary.

If Oneil was installed from source, run `cargo uninstall oneil`.

If Oneil's Python library was installed, run `pip uninstall oneil` in the same
virtual environment that it was installed in.

## Troubleshooting

- **`oneil: command not found`**  
  Ensure the directory containing the `oneil` binary is on your `PATH`.

- **Python-related build errors** (from source)  
  Install Python 3.10+ and development headers (see Prerequisites).

- **Permission denied** (Linux/macOS)  
  After moving the binary, run `chmod +x /path/to/oneil` (or the path you used).
