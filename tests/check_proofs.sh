#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# Typecheck the real package and prove this gate rejects invalid propositions.
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
if [[ $# != 1 || $1 != /* ]]; then
  printf 'Usage: bash tests/check_proofs.sh /absolute/path/to/evidence-directory\n' >&2
  exit 2
fi
mkdir -p "$1"
proof_run="$(mktemp -d "$1/groove-proofs.XXXXXX")"
compiler="${IDRIS2:-idris2}"
"$compiler" --version
"$compiler" --build-dir "$proof_run/package" --typecheck groove-proofs.ipkg
"$compiler" --build-dir "$proof_run/positive" --typecheck tests/proofs/positive.ipkg
for entry in reject-revocation:Mismatch reject-appdata:Mismatch; do
  fixture="${entry%%:*}"
  diagnostic="${entry#*:}"
  log="$proof_run/$fixture.log"
  if "$compiler" --no-color --build-dir "$proof_run/$fixture" --typecheck "tests/proofs/$fixture.ipkg" > "$log" 2>&1; then
    printf 'FAIL: invalid proposition was accepted: %s\n' "$fixture" >&2
    exit 1
  fi
  # A missing compiler or import error is not a successful rejection control.
  if ! grep -q "$diagnostic" "$log"; then
    sed -n '1,180p' "$log"
    printf 'FAIL: %s failed for an unexpected reason\n' "$fixture" >&2
    exit 1
  fi
  printf 'PASS: %s rejected for its expected type error\n' "$fixture"
done
printf 'Proof gate passed; evidence: %s\n' "$proof_run"
