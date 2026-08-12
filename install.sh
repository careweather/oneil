#!/usr/bin/env bash
set -euo pipefail

# Repository root (directory containing this script).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

NO_PYTHON=false
EDITABLE=false

usage() {
	cat <<'EOF'
Usage: install.sh [options]

  Builds and installs the Oneil CLI with Cargo. By default, also installs the
  Python package (import oneil) for the current interpreter.

Options:
  --no-python    Install the CLI only: no Python bindings and no pip package.
  -e, --editable Install the Python package in editable mode (development).
  -h, --help     Show this help.

Prerequisites:
  - Cargo (Rust): https://rustup.rs/
  - gcc (or another C toolchain Cargo can use for linking on this platform)
  - For the default install: Python 3.10+ with pip (python3 -m pip / python -m pip)
    and Python development headers (e.g. python3-devel on Fedora/RHEL,
    python3-dev on Debian/Ubuntu)
EOF
}

while [[ $# -gt 0 ]]; do
	case "$1" in
	--no-python) NO_PYTHON=true ;;
	-e | --editable) EDITABLE=true ;;
	-h | --help)
		usage
		exit 0
		;;
	*)
		echo "Unknown option: $1" >&2
		usage >&2
		exit 1
		;;
	esac
	shift
done

if [[ "$NO_PYTHON" == true && "$EDITABLE" == true ]]; then
	echo "Note: --editable has no effect with --no-python." >&2
fi

if ! command -v cargo >/dev/null 2>&1; then
	cat >&2 <<EOF
Error: Cargo was not found on your PATH.

Install the Rust toolchain with rustup:
  https://rustup.rs/

  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

Then restart your terminal, or run:
  source ${HOME}/.cargo/env
EOF
	exit 1
fi

if ! command -v gcc >/dev/null 2>&1; then
	cat >&2 <<'EOF'
Error: gcc was not found on your PATH.

A C compiler is required to build Oneil (Rust linking and native extensions).

Install one of:
  Fedora/RHEL: sudo dnf install gcc
  Debian/Ubuntu: sudo apt install build-essential
EOF
	exit 1
fi

ONEIL_PKG="$SCRIPT_DIR/src-rs/oneil"
if [[ ! -f "$ONEIL_PKG/Cargo.toml" ]]; then
	echo "Error: expected Cargo.toml at $ONEIL_PKG" >&2
	exit 1
fi

PYTHON_CMD=""
if [[ "$NO_PYTHON" == false ]]; then
	if [[ ! -f "$SCRIPT_DIR/pyproject.toml" ]]; then
		echo "Error: pyproject.toml not found at $SCRIPT_DIR" >&2
		exit 1
	fi

	if command -v python3 >/dev/null 2>&1; then
		PYTHON_CMD="python3"
	elif command -v python >/dev/null 2>&1; then
		PYTHON_CMD="python"
	else
		cat <<'EOF' >&2
Error: Python 3.10+ is required for the library install but no python3/python was found.

Install Python, or re-run with --no-python to install only the CLI with no Python bindings.
EOF
		exit 1
	fi

	if ! "$PYTHON_CMD" -c 'import sys; sys.exit(0 if sys.version_info >= (3, 10) else 1)' 2>/dev/null; then
		echo "Error: Python 3.10 or newer is required. Found: $($PYTHON_CMD --version 2>&1)" >&2
		exit 1
	fi

	if ! "$PYTHON_CMD" -c 'import os, sys, sysconfig; inc=sysconfig.get_path("include"); sys.exit(0 if os.path.isfile(os.path.join(inc, "Python.h")) else 1)' 2>/dev/null; then
		cat >&2 <<'EOF'
Error: Python development headers were not found (Python.h is missing).

The CLI build links against Python; install headers before building.

Install the development package for your distribution, then re-run this script:
  Fedora/RHEL: sudo dnf install python3-devel
  Debian/Ubuntu: sudo apt install python3-dev
EOF
		exit 1
	fi
fi

echo "Installing Oneil CLI with Cargo..."
cargo install --force --path "$ONEIL_PKG" --no-default-features --features rust-lib

if [[ "$NO_PYTHON" == false ]]; then
	echo "Installing Oneil Python package (deprecated)..."
	cd "$SCRIPT_DIR"
	if [[ "$EDITABLE" == true ]]; then
		"$PYTHON_CMD" -m pip install -e .
	else
		"$PYTHON_CMD" -m pip install .
	fi
fi

echo ""
echo "Done."
echo "Ensure ~/.cargo/bin is on your PATH to run: oneil --version"
