# Installation

This section describes how to install the Oneil CLI (Rust implementation) on Linux, Windows, and macOS. The recommended path for most users is to download a pre-built binary from [GitHub Releases](https://github.com/careweather/oneil/releases).

## Option 1: Download a release from GitHub

Pre-built binaries are published on the [Releases](https://github.com/careweather/oneil/releases) page for:

- **Linux** — `x86_64-unknown-linux-gnu` (`system`, `uv`)
- **Windows** — `x86_64-pc-windows-msvc` (`system`, `uv`)
- **macOS** — `aarch64-apple-darwin` (Apple Silicon; `homebrew`, `system`, `uv`)

The CLI does not ship Python. Each archive is linked against a specific **Python 3.12** layout. The [VS Code / Cursor extension](#editor-and-tooling-optional) detects which layout you have and downloads that flavor. For a manual install, pick the matching archive:

- **homebrew** — `brew install python@3.12` (macOS)
- **system** — python.org 3.12 on macOS, distro `libpython3.12` on Linux, or Python 3.12 on `PATH` on Windows
- **uv** — `uv python install 3.12`. The extension sets the library search path when it launches the CLI. For a `PATH` install, prefer Homebrew/system or [build from source](#option-3-install-from-source-using-the-install-script) so the binary is linked to *your* uv prefix.

Pushing a version tag (for example `v1.0.0`) runs the Release workflow, which builds these archives and attaches them to the GitHub Release for that tag. In GitHub Actions, prefer [`careweather/oneil/actions/install-oneil`](https://github.com/careweather/oneil/tree/main/actions/install-oneil) (or [`model-test-report`](https://github.com/careweather/oneil/tree/main/actions/model-test-report) for full model-repo CI) — see [Appendix C](./c-ci-setup.md).

### Linux / macOS

The release also attaches `install-oneil.sh`, which detects Homebrew / uv / system Python 3.12 and downloads that archive into `~/.local/bin`:

```sh
curl -fsSL https://github.com/careweather/oneil/releases/latest/download/install-oneil.sh | bash
# or a specific tag:
curl -fsSL https://github.com/careweather/oneil/releases/download/v1.0.0/install-oneil.sh | bash
```

To pick the archive yourself:

1. Open the [latest release](https://github.com/careweather/oneil/releases/latest).
2. Download the archive for your OS, architecture, and Python layout (for example `oneil-v1.0.0-x86_64-unknown-linux-gnu-system.tar.gz` or `oneil-v1.0.0-aarch64-apple-darwin-homebrew.tar.gz`).
3. Unpack and put the `oneil` binary on your `PATH`:

   ```sh
   tar -xzf oneil-v*-x86_64-unknown-linux-gnu-system.tar.gz
   sudo mv oneil /usr/local/bin/
   # or, without sudo:
   mkdir -p ~/.local/bin && mv oneil ~/.local/bin/
   # ensure ~/.local/bin is in your PATH
   ```

4. Confirm:

   ```sh
   oneil --version
   ```

### Windows

1. Open the [latest release](https://github.com/careweather/oneil/releases/latest).
2. Download the Windows zip for your Python layout (for example `oneil-v1.0.0-x86_64-pc-windows-msvc-system.zip`).
3. Unzip and either move `oneil.exe` into a directory on your `PATH`, or add the folder containing `oneil.exe` to your `PATH`.
4. Confirm in PowerShell or Command Prompt:

   ```cmd
   oneil --version
   ```

## Option 2: Nix

If you use [Nix](https://nixos.org/download/) with flakes enabled, you can run or install Oneil without a separate Rust or Python setup. The flake links against CPython 3.12 from nixpkgs and includes it at runtime.

Run without installing:

```sh
nix run github:careweather/oneil -- --help
nix run github:careweather/oneil -- path/to/model.on
```

Install into your profile:

```sh
nix profile install github:careweather/oneil
oneil --version
```

From a local checkout, `nix build` and `nix run` use the same package (`packages.oneil`). For NixOS or home-manager, apply the flake overlay and add `pkgs.oneil`:

```nix
{
  inputs.oneil.url = "github:careweather/oneil";

  # NixOS: overlays go on nixpkgs, then e.g. environment.systemPackages = [ pkgs.oneil ];
  # home-manager: nixpkgs.overlays = [ inputs.oneil.overlays.default ];
}
```

The first build compiles from source. Contributors can use `nix develop` in the repository for the Rust toolchain, Python 3.12, and VS Code extension tools.

## Prerequisites for building from source

The options below build Oneil yourself. You will need:

- **Rust**: [rustup](https://rustup.rs/) — install and ensure `cargo` is on your `PATH`.
- **gcc**
  - Install on Fedora/RHEL: `sudo dnf install gcc`
  - Install on Debian/Ubuntu: `sudo apt install build-essential`
- **Python 3.12** — the only CPython version Oneil supports. Needed at runtime when models [`import`](./11-importing-python.md) Python modules, and when building from source (development headers). Install it with one of:
  - [uv](https://docs.astral.sh/uv/): `uv python install 3.12`
  - Homebrew: `brew install python@3.12`
  - Fedora/RHEL: `sudo dnf install python3.12-devel`
  - Debian/Ubuntu: `sudo apt install python3.12-dev`

  Helper `.py` files can `import oneil` because the CLI includes the [Python library](./a-python-api.md).

## Option 3: Install from source using the install script

From the repository root, the install script builds the **Rust CLI** with default features (so models can [`import`](./11-importing-python.md) `.py` files and those files can `import oneil`).

```sh
git clone https://github.com/careweather/oneil.git
cd oneil
./install.sh
```

On Windows, use `install.bat`.

You need **Python 3.12** (`uv python install 3.12` or `brew install python@3.12`). The script prefers `uv python find 3.12`, then Homebrew `python@3.12`.

## Option 4: Install from source with Cargo

Use this if you want the latest development version or need to customize the build.

1. Clone the repository:

   ```sh
   git clone https://github.com/careweather/oneil.git
   cd oneil
   ```

2. Build and install the `oneil` binary (requires Rust):

   ```sh
   cargo install --path src/oneil
   ```

   It places `oneil` in `~/.cargo/bin` (or `%USERPROFILE%\.cargo\bin` on Windows); keep that directory on your `PATH`.

   Building from source requires Python 3.12 (see Prerequisites).

3. Confirm:

   ```sh
   oneil --version
   ```

## Option 5: Run from the repository (development)

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

- **Release binary**: download the newer archive from [Releases](https://github.com/careweather/oneil/releases) and replace the previous `oneil` binary on your `PATH`.
- **Nix profile**: `nix profile upgrade oneil` (or re-run `nix profile install github:careweather/oneil`).
- **From source**: pull the latest code (or check out the new tag), then re-run `./install.sh` or `cargo install --path src/oneil`.

## Editor and tooling (optional)


- **VS Code / Cursor**: Install the [Oneil extension](https://marketplace.visualstudio.com/items?itemName=careweather.oneil) from the Marketplace for LSP and syntax highlighting. The extension can download the Oneil CLI from [GitHub Releases](https://github.com/careweather/oneil/releases) (Command Palette: “Oneil: Install or Update CLI”, or “Oneil: Select CLI Version…” to install a different published tag). It picks the Homebrew, system, or uv archive that matches the Python 3.12 on the machine. Set `oneil.serverPath` only when you want to force a local build; that setting disables managed updates.

- **Vim**: See the [Vim support](https://github.com/careweather/oneil#vim-support) section in the main README for syntax highlighting.

## Uninstalling Oneil

If Oneil was installed as a release binary, delete the release binary.

If Oneil was installed with Nix, run `nix profile remove oneil` (or remove `pkgs.oneil` from your NixOS / home-manager configuration).

If Oneil was installed from source, run `cargo uninstall oneil`.

If the Python library was installed with pip, run `pip uninstall oneil` in the same virtual environment.

## Troubleshooting

- **`oneil: command not found`**  
  Ensure the directory containing the `oneil` binary is on your `PATH`.

- **Python-related build errors** (from source) or **`oneil --version` aborts**  
  Install Python 3.12 to match the archive flavor you downloaded (`brew install python@3.12`, the python.org 3.12 installer, distro `python3.12`, or `uv python install 3.12`). See Prerequisites. The extension picks the flavor for you.

- **Permission denied** (Linux/macOS)  
  After moving the binary, run `chmod +x /path/to/oneil` (or the path you used).

- **macOS: “cannot be opened because the developer cannot be verified”**  
  Right-click the binary → **Open**, or remove the quarantine attribute:
  `xattr -d com.apple.quarantine /path/to/oneil`.
