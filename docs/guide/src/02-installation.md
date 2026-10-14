# Installation

This section describes how to install the Oneil CLI (Rust implementation) on Linux, Windows, and macOS. The recommended path for most users is to download a pre-built binary from [GitHub Releases](https://github.com/careweather/oneil/releases).

## Python versions

The release CLI supports **CPython 3.12** and **CPython 3.14**. It does not ship Python. Install one of those two, then download the archive whose name ends in that version.

| Python you have | Archive to download | How to install Python if you need it |
| --- | --- | --- |
| 3.14 | `…-py3.14` | `uv python install 3.14` or `brew install python@3.14` |
| 3.12 | `…-py3.12` | `uv python install 3.12` or `brew install python@3.12` |
| Both | `…-py3.14` | The install script and the VS Code / Cursor extension pick 3.14 |
| Neither | `…-py3.14` | Install 3.14, then download that archive |

A 3.12 archive does not run on 3.14, and a 3.14 archive does not run on 3.12.

[Nix](#option-2-nix) is the exception: that build links CPython 3.12 from nixpkgs and includes it, so you do not install Python yourself.

The separate `import oneil` wheel (Appendix A) supports Python 3.10 and newer. That package is for a standalone Python process. The CLI archives above are still 3.12 and 3.14 only.

## Option 1: Download a release from GitHub

Pre-built binaries are published on the [Releases](https://github.com/careweather/oneil/releases) page for:

- **Linux** — `x86_64-unknown-linux-gnu` (`py3.12`, `py3.14`)
- **Windows** — `x86_64-pc-windows-msvc` (`py3.12`, `py3.14`)
- **macOS** — `aarch64-apple-darwin` (Apple Silicon; `py3.12`, `py3.14`)

The CLI does not ship Python. Each archive links one minor version and runs against python installed via various channels: Homebrew, the python.org installer, a distro package, or uv. The [VS Code / Cursor extension](#editor-and-tooling-optional) downloads the archive for the newest of 3.14 and 3.12 that is installed.

Pushing a version tag (for example `v1.0.0`) runs the Release workflow, which builds these archives and attaches them to the GitHub Release for that tag. In GitHub Actions, prefer [`careweather/oneil/actions/install-oneil`](https://github.com/careweather/oneil/tree/main/actions/install-oneil) (or [`model-test-report`](https://github.com/careweather/oneil/tree/main/actions/model-test-report) for full model-repo CI) — see [Appendix C](./c-ci-setup.md).

### Linux / macOS

The release also attaches `install-oneil.sh`, which detects Python 3.14 or 3.12 and downloads that archive into `~/.local/bin`:

```sh
curl -fsSL https://github.com/careweather/oneil/releases/latest/download/install-oneil.sh | bash
# or a specific tag:
curl -fsSL https://github.com/careweather/oneil/releases/download/v1.0.0/install-oneil.sh | bash
```

To pick the archive yourself:

1. Open the [latest release](https://github.com/careweather/oneil/releases/latest).
2. Download the archive for your OS, architecture, and Python version (for example `oneil-v1.0.0-x86_64-unknown-linux-gnu-py3.12.tar.gz` or `oneil-v1.0.0-aarch64-apple-darwin-py3.14.tar.gz`).
3. Unpack and put the extracted binaries on your `PATH`, in the same directory:

   ```sh
   tar -xzf oneil-v*-x86_64-unknown-linux-gnu-py3.12.tar.gz
   sudo mv oneil oneil-runner /usr/local/bin/
   # or, without sudo:
   mkdir -p ~/.local/bin && mv oneil oneil-runner ~/.local/bin/
   # ensure ~/.local/bin is in your PATH
   ```

4. Confirm:

   ```sh
   oneil --version
   ```

### Windows

1. Open the [latest release](https://github.com/careweather/oneil/releases/latest).
2. Download the Windows zip for your Python version (for example `oneil-v1.0.0-x86_64-pc-windows-msvc-py3.12.zip`).
3. Unzip and either move `oneil.exe` and `oneil-runner.exe` into a directory on your `PATH`, or add the folder containing both to your `PATH`.
4. Confirm in PowerShell or Command Prompt:

   ```cmd
   oneil --version
   ```

## Option 2: Nix

If you use [Nix](https://nixos.org/download/) with flakes enabled, you can run or install Oneil without a separate Rust or Python setup. The flake links against CPython 3.12 from nixpkgs and includes it at runtime.

To try Oneil without adding it to a system configuration:

```sh
nix run github:careweather/oneil -- --help
nix run github:careweather/oneil -- path/to/model.on
```

To install it, add the flake overlay and `pkgs.oneil` to your NixOS or home-manager configuration:

```nix
{
  inputs.oneil.url = "github:careweather/oneil";

  # nixpkgs.overlays = [ inputs.oneil.overlays.default ];
  # environment.systemPackages = [ pkgs.oneil ];           # NixOS
  # home.packages = [ pkgs.oneil ];                        # home-manager
}
```

The overlay also provides the VS Code / Cursor extension as
`pkgs.vscode-extensions.careweather.oneil`. That package defaults
`oneil.serverPath` to the flake-built CLI, so the editor uses the same
CPython-linked binary as `pkgs.oneil` instead of downloading a GitHub
Release. Example with home-manager:

```nix
{
  # nixpkgs.overlays = [ inputs.oneil.overlays.default ];
  # programs.vscode.profiles.default.extensions = [
  #   pkgs.vscode-extensions.careweather.oneil
  # ];
}
```

You can also build the extension with `nix build github:careweather/oneil#oneil-vscode`.

The first evaluation compiles from source. Contributors can use `nix develop` in the repository for the Rust toolchain, Python 3.12, and VS Code extension tools.

## Prerequisites for building from source

The options below build Oneil yourself. You will need:

- **Rust**: [rustup](https://rustup.rs/) — install and ensure `cargo` is on your `PATH`.
- **gcc**
  - Install on Fedora/RHEL: `sudo dnf install gcc`
  - Install on Debian/Ubuntu: `sudo apt install build-essential`
- **Python 3.12 or 3.14** — needed at runtime when models [`import`](./11-importing-python.md) Python modules, and when building from source (development headers). The binary runs against the minor version it was built with. Install it with one of:
  - [uv](https://docs.astral.sh/uv/): `uv python install 3.12` or `uv python install 3.14`
  - Homebrew: `brew install python@3.12` or `brew install python@3.14`
  - Fedora/RHEL: `sudo dnf install python3.12-devel` or `python3.14-devel`
  - Debian/Ubuntu: `sudo apt install python3.12-dev` or `python3.14-dev`

  Helper `.py` files can `import oneil` because the CLI includes the [Python library](./a-python-api.md).

## Option 3: Install from source using the install script

From the repository root, the install script builds the **Rust CLI** with default features (so models can [`import`](./11-importing-python.md) `.py` files and those files can `import oneil`).

```sh
git clone https://github.com/careweather/oneil.git
cd oneil
./install.sh
```

On Windows, use `install.bat`.

You need **Python 3.12 or 3.14** (`uv python install 3.14` or `brew install python@3.14`, and the same for 3.12). The script prefers 3.14, then 3.12, via uv, then Homebrew, then `python3.14` / `python3.12` on `PATH`.

## Option 4: Install from source with Cargo

Use this if you want the latest development version or need to customize the build.

1. Clone the repository:

   ```sh
   git clone https://github.com/careweather/oneil.git
   cd oneil
   ```

2. Build and install `oneil` and `oneil-runner` (requires Rust):

   ```sh
   cargo install --path src/oneil_loader
   cargo install --path src/oneil
   ```

   That places both binaries in `~/.cargo/bin` (or `%USERPROFILE%\.cargo\bin` on Windows); keep that directory on your `PATH`. `oneil` is the program you run. `oneil-runner` must sit in the same directory.

   Building from source requires Python 3.12 or 3.14 (see Prerequisites). The installed CLI uses the minor version selected by `PYO3_PYTHON` at build time.

3. Confirm:

   ```sh
   oneil --version
   ```

## Option 5: Run from the repository (development)

For day-to-day development without installing:

```sh
git clone https://github.com/careweather/oneil.git
cd oneil
cargo build -p oneil -p oneil_loader
./target/debug/oneil --version
# or run the runner directly against the Python it was linked with:
cargo run -p oneil -- path/to/model.on
```

## Updating

- **Release binary**: download the newer archive from [Releases](https://github.com/careweather/oneil/releases) and replace the previous `oneil` binary on your `PATH`.
- **Nix**: bump the `oneil` flake input in your configuration and rebuild.
- **From source**: pull the latest code (or check out the new tag), then re-run `./install.sh` or install both `src/oneil_loader` and `src/oneil`.

## Editor and tooling (optional)


- **VS Code / Cursor**: Install the [Oneil extension](https://marketplace.visualstudio.com/items?itemName=careweather.oneil) from the Marketplace for LSP and syntax highlighting, or install `pkgs.vscode-extensions.careweather.oneil` from this flake (see [Option 2: Nix](#option-2-nix)). The Marketplace extension can download the Oneil CLI from [GitHub Releases](https://github.com/careweather/oneil/releases) (Command Palette: “Oneil: Install or Update CLI”, or “Oneil: Select CLI Version…” to install a different published tag). It picks the Python 3.14 archive when 3.14 is installed, otherwise the 3.12 archive. Set `oneil.serverPath` only when you want to force a local build; that setting disables managed updates. The Nix package already sets `oneil.serverPath` to the flake-built CLI.

- **Vim**: See the [Vim support](https://github.com/careweather/oneil#vim-support) section in the main README for syntax highlighting.

## Uninstalling Oneil

If Oneil was installed as a release binary, delete the release binary.

If Oneil was installed with Nix, remove `pkgs.oneil` from your NixOS or home-manager configuration and rebuild.

If Oneil was installed from source, run `cargo uninstall oneil`.

If the Python library was installed with pip, run `pip uninstall oneil` in the same virtual environment.

## Troubleshooting

- **`oneil: command not found`**  
  Ensure the directory containing the `oneil` binary is on your `PATH`.

- **Python-related build errors** (from source) or **`oneil --version` aborts**  
  Install the Python minor version named in the archive (`py3.12` or `py3.14`): `brew install python@3.12`, the python.org installer, distro `python3.12`, or `uv python install 3.12` (and the same for 3.14). `ONEIL_PYTHON` selects which install to use when several of that version are present. See Prerequisites.

- **Permission denied** (Linux/macOS)  
  After moving the binary, run `chmod +x /path/to/oneil` (or the path you used).

- **macOS: “cannot be opened because the developer cannot be verified”**  
  Right-click the binary → **Open**, or remove the quarantine attribute:
  `xattr -d com.apple.quarantine /path/to/oneil`.
