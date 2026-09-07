#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
#
# Structural validation for the Groove browser extension (Firefox MV2).
# Run from anywhere: paths resolve relative to this script.

set -uo pipefail
cd "$(dirname "$0")/.."

pass=0
fail=0

check() {
  local desc="$1"; shift
  if "$@"; then
    echo "PASS: $desc"
    pass=$((pass + 1))
  else
    echo "FAIL: $desc"
    fail=$((fail + 1))
  fi
}

check "manifest.json is valid JSON" bun -e 'JSON.parse(require("fs").readFileSync("manifest.json"))'
check "manifest is MV2" bun -e 'const m=JSON.parse(require("fs").readFileSync("manifest.json")); if(m.manifest_version!==2) throw 0'
check "no MV3-only host_permissions key (MV2: origins live in permissions)" bash -c '! grep -q host_permissions manifest.json'
check "icons exist and are PNGs" bash -c 'file icons/groove-48.png icons/groove-96.png | grep -c "PNG image" | grep -q 2'
check "all background scripts exist" bun -e '
  const fs=require("fs");
  const m=JSON.parse(fs.readFileSync("manifest.json"));
  for (const s of m.background.scripts) if (!fs.existsSync(s)) throw new Error(s);
'
check "generated targets in sync with registry" bun scripts/gen-targets.mjs --check
check "vendored client in sync with clients/js/groove-client.js" bash -c 'diff <(tail -n +2 background/groove-client.vendor.js) ../js/groove-client.js'
check "no stale verification claims" bash -c '! grep -rq "Idris2-verified" popup/ background/ content/'
check "no invented /message or /recv endpoints" bash -c '! grep -rqE "groove/(message|recv)" background/ content/'

echo "----"
echo "$pass passed, $fail failed"
exit "$fail"
