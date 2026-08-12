#!/usr/bin/env bash
# Download a Oneil CLI release archive and put it on PATH.
# Expected env: ONEIL_VERSION, and optionally GH_TOKEN / GITHUB_TOKEN for gh.
set -euo pipefail

case "${RUNNER_OS}/${RUNNER_ARCH}" in
  Linux/X64)
    triple="x86_64-unknown-linux-gnu"
    archive="oneil-${ONEIL_VERSION}-${triple}.tar.gz"
    binary_name="oneil"
    ;;
  Windows/X64)
    triple="x86_64-pc-windows-msvc"
    archive="oneil-${ONEIL_VERSION}-${triple}.zip"
    binary_name="oneil.exe"
    ;;
  macOS/ARM64)
    triple="aarch64-apple-darwin"
    archive="oneil-${ONEIL_VERSION}-${triple}.tar.gz"
    binary_name="oneil"
    ;;
  *)
    echo "::error::Unsupported runner: ${RUNNER_OS}/${RUNNER_ARCH} (release binaries cover Linux x86_64, Windows x86_64, and Apple Silicon macOS)"
    exit 1
    ;;
esac

install_dir="${RUNNER_TEMP}/oneil-${ONEIL_VERSION}"
mkdir -p "${install_dir}"
cd "${install_dir}"

download_url="https://github.com/careweather/oneil/releases/download/${ONEIL_VERSION}/${archive}"
echo "Downloading ${download_url}"

if command -v gh >/dev/null 2>&1; then
  gh release download "${ONEIL_VERSION}" \
    --repo careweather/oneil \
    --pattern "${archive}" \
    --dir .
else
  curl -fsSL "${download_url}" -o "${archive}"
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

if [[ ! -f "${binary_name}" ]]; then
  echo "::error::Expected ${binary_name} in ${archive}"
  ls -la
  exit 1
fi

chmod +x "${binary_name}"
oneil_path="$(pwd)/${binary_name}"

# Soften macOS Gatekeeper when running unsigned release binaries in CI.
if [[ "${RUNNER_OS}" == "macOS" ]] && command -v xattr >/dev/null 2>&1; then
  xattr -d com.apple.quarantine "${oneil_path}" 2>/dev/null || true
fi

echo "${install_dir}" >> "${GITHUB_PATH}"
export PATH="${install_dir}:${PATH}"

version_line="$("${oneil_path}" --version)"
echo "Installed ${version_line}"
echo "version=${version_line}" >> "${GITHUB_OUTPUT}"
echo "oneil-path=${oneil_path}" >> "${GITHUB_OUTPUT}"
