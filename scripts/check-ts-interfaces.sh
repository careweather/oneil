#!/usr/bin/env bash
# Fails if regenerating `oneil-ts-interfaces` would change any committed file
# under packages/ts-interfaces/.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${repo_root}"

./scripts/generate-ts-interfaces.sh

if ! git diff --exit-code -- packages/ts-interfaces; then
  cat >&2 <<'EOF'

TypeScript bindings in packages/ts-interfaces are out of date.
Run `./scripts/generate-ts-interfaces.sh` and commit the result.

EOF
  exit 1
fi
