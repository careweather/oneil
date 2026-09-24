#!/usr/bin/env bash
# Download a Oneil CLI release archive and put `oneil` on PATH.
#
# GitHub Actions: set ONEIL_VERSION (and optionally GH_TOKEN). Writes GITHUB_PATH /
# GITHUB_ENV / GITHUB_OUTPUT when those files exist.
#
# Local: ONEIL_VERSION or the first argument, else DEFAULT_ONEIL_VERSION (stamped
# when this file is attached to a GitHub Release). Installs into
# "${ONEIL_INSTALL_DIR:-$HOME/.local/bin}".
set -euo pipefail

# Stamped by the Release workflow when attaching this script as install-oneil.sh.
DEFAULT_ONEIL_VERSION=""

in_gha() {
  [[ "${GITHUB_ACTIONS:-}" == "true" ]]
}

err() {
  if in_gha; then
    echo "::error::$*"
  else
    echo "Error: $*" >&2
  fi
}

usage() {
  cat <<'EOF'
Download a Oneil CLI release binary and install it.

Usage:
  ONEIL_VERSION=v1.0.0 bash install-oneil.sh
  bash install-oneil.sh v1.0.0

Environment:
  ONEIL_VERSION       Release tag (required unless stamped or passed as $1)
  ONEIL_INSTALL_DIR   Local install directory (default: ~/.local/bin)
  GH_TOKEN            Optional; used by `gh release download` for rate limits
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

ONEIL_VERSION="${ONEIL_VERSION:-${1:-${DEFAULT_ONEIL_VERSION}}}"
if [[ -z "${ONEIL_VERSION}" ]]; then
  err "Set ONEIL_VERSION or pass a release tag (e.g. v1.0.0)."
  exit 1
fi

if in_gha; then
  OS_NAME="${RUNNER_OS}"
  ARCH_NAME="${RUNNER_ARCH}"
else
  case "$(uname -s)" in
    Linux) OS_NAME="Linux" ;;
    Darwin) OS_NAME="macOS" ;;
    MINGW*|MSYS*|CYGWIN*) OS_NAME="Windows" ;;
    *)
      err "Unsupported OS: $(uname -s) (release binaries cover Linux x86_64, Windows x86_64, and Apple Silicon macOS)"
      exit 1
      ;;
  esac
  case "$(uname -m)" in
    x86_64|amd64) ARCH_NAME="X64" ;;
    arm64|aarch64) ARCH_NAME="ARM64" ;;
    *)
      err "Unsupported architecture: $(uname -m)"
      exit 1
      ;;
  esac
fi

case "${OS_NAME}/${ARCH_NAME}" in
  Linux/X64)
    triple="x86_64-unknown-linux-gnu"
    ext="tar.gz"
    binary_name="oneil"
    ;;
  Windows/X64)
    triple="x86_64-pc-windows-msvc"
    ext="zip"
    binary_name="oneil.exe"
    ;;
  macOS/ARM64)
    triple="aarch64-apple-darwin"
    ext="tar.gz"
    binary_name="oneil"
    ;;
  *)
    err "Unsupported runner: ${OS_NAME}/${ARCH_NAME} (release binaries cover Linux x86_64, Windows x86_64, and Apple Silicon macOS)"
    exit 1
    ;;
esac

python_minor_present() {
  local minor="$1"
  if command -v "python${minor}" >/dev/null 2>&1; then
    return 0
  fi
  if command -v uv >/dev/null 2>&1 && uv python find "${minor}" >/dev/null 2>&1; then
    return 0
  fi
  if [[ "${OS_NAME}" == "macOS" ]] \
    && { [[ -e "/opt/homebrew/opt/python@${minor}/Frameworks/Python.framework/Versions/${minor}/Python" ]] \
      || [[ -e "/Library/Frameworks/Python.framework/Versions/${minor}/Python" ]]; }; then
    return 0
  fi
  if [[ "${OS_NAME}" == "Windows" ]] && py "-${minor}" -c "import sys" >/dev/null 2>&1; then
    return 0
  fi
  return 1
}

python_minor=""
for candidate in 3.14 3.12; do
  if python_minor_present "${candidate}"; then
    python_minor="${candidate}"
    break
  fi
done
if [[ -z "${python_minor}" ]]; then
  err "Need Python 3.12 or 3.14 to run the release CLI."
  exit 1
fi

try_download() {
  local archive="$1"
  local download_url="https://github.com/careweather/oneil/releases/download/${ONEIL_VERSION}/${archive}"
  echo "Downloading ${download_url}"
  if command -v gh >/dev/null 2>&1; then
    gh release download "${ONEIL_VERSION}" \
      --repo careweather/oneil \
      --pattern "${archive}" \
      --dir .
  else
    curl -fsSL "${download_url}" -o "${archive}"
  fi
}

if in_gha; then
  work_dir="${RUNNER_TEMP}/oneil-${ONEIL_VERSION}"
else
  work_dir="$(mktemp -d "${TMPDIR:-/tmp}/oneil-install.XXXXXX")"
fi
mkdir -p "${work_dir}"
cd "${work_dir}"

versioned="oneil-${ONEIL_VERSION}-${triple}-py${python_minor}.${ext}"
unflavored="oneil-${ONEIL_VERSION}-${triple}.${ext}"
archive=""

if try_download "${versioned}"; then
  archive="${versioned}"
elif try_download "${unflavored}"; then
  echo "No ${versioned}; using unflavored archive from an older release"
  archive="${unflavored}"
else
  err "Could not download ${versioned} or ${unflavored}"
  exit 1
fi

if [[ "${archive}" == *.zip ]]; then
  if command -v unzip >/dev/null 2>&1; then
    unzip -o "${archive}"
  else
    python3 - "${archive}" <<'PY'
import sys, zipfile
zipfile.ZipFile(sys.argv[1]).extractall(".")
PY
  fi
else
  tar -xzf "${archive}"
fi

runner_name="${binary_name/oneil/oneil-runner}"

if [[ ! -f "${binary_name}" ]]; then
  err "Expected ${binary_name} in ${archive}"
  ls -la
  exit 1
fi

chmod +x "${binary_name}"
if [[ -f "${runner_name}" ]]; then
  chmod +x "${runner_name}"
fi

if [[ "${OS_NAME}" == "macOS" ]] && command -v xattr >/dev/null 2>&1; then
  xattr -d com.apple.quarantine "${binary_name}" 2>/dev/null || true
  if [[ -f "${runner_name}" ]]; then
    xattr -d com.apple.quarantine "${runner_name}" 2>/dev/null || true
  fi
fi

if in_gha; then
  oneil_path="$(pwd)/${binary_name}"
  echo "${work_dir}" >> "${GITHUB_PATH}"
  export PATH="${work_dir}:${PATH}"
else
  dest_dir="${ONEIL_INSTALL_DIR:-${HOME}/.local/bin}"
  mkdir -p "${dest_dir}"
  mv "${binary_name}" "${dest_dir}/${binary_name}"
  if [[ -f "${runner_name}" ]]; then
    mv "${runner_name}" "${dest_dir}/${runner_name}"
  fi
  oneil_path="${dest_dir}/${binary_name}"
  export PATH="${dest_dir}:${PATH}"
fi

version_err="$(mktemp "${TMPDIR:-/tmp}/oneil-version.XXXXXX")"
if ! version_line="$("${oneil_path}" --version 2>"${version_err}")"; then
  err "Installed oneil failed \`--version\`. Install Python ${python_minor}."
  cat "${version_err}" >&2 || true
  rm -f "${version_err}"
  exit 1
fi
rm -f "${version_err}"

echo "Installed ${version_line} (Python ${python_minor}) at ${oneil_path}"
if in_gha; then
  echo "version=${version_line}" >> "${GITHUB_OUTPUT}"
  echo "oneil-path=${oneil_path}" >> "${GITHUB_OUTPUT}"
fi
