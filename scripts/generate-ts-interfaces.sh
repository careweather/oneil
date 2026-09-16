#!/usr/bin/env bash
# Regenerate TypeScript bindings for Oneil JSON wire formats from the Rust
# DTOs (via ts-rs). Writes into packages/ts-interfaces/src/generated/ and
# refreshes the package barrel at packages/ts-interfaces/src/index.ts.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
pkg_root="${repo_root}/packages/ts-interfaces"
generated_dir="${pkg_root}/src/generated"

cd "${repo_root}/src"
cargo run --quiet --example export_ts_interfaces -p oneil_cli --features oneil_cli/ts-bindings

# Rebuild the barrel so new generated types are exported without hand edits.
{
  cat <<'EOF'
/**
 * Generated TypeScript bindings for Oneil JSON wire formats.
 *
 * Do not edit `./generated/` by hand — run `./scripts/generate-ts-interfaces.sh`
 * from the repo root. CI fails if regeneration would change committed output
 * (`./scripts/check-ts-interfaces.sh`).
 *
 * See `docs/CODING_STANDARDS.md` (Generated TypeScript Bindings).
 */

EOF

  # Locale-independent order so CI and local `en_US.UTF-8` sort the same.
  for path in $(find "${generated_dir}" -maxdepth 1 -name '*.ts' -print | LC_ALL=C sort); do
    name="$(basename "${path}" .ts)"
    printf 'export type { %s } from "./generated/%s.js"\n' "${name}" "${name}"
  done

  cat <<'EOF'

/** Wire shape for a float under `oneil_shared::serde::f64`. */
export type FloatValue = number | { float_special: "NAN" | "INFINITY" | "NEGATIVE_INFINITY" }
EOF
} > "${pkg_root}/src/index.ts"
