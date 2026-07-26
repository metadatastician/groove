#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
#
# soft-hard-lease-demo — the "both soft AND hard groove with a real
# capability" evidence run (cleave docs/PROOF-NEEDS.adoc "The bar", groove
# SPEC §4.6, CONF-L2-08..12).
#
#   ① discover the provider's real capabilities
#   ② soft connect (short TTL) → let it lapse → 410 + zero-residue expiry attested
#   ③ hard connect → heartbeat across ≥3 TTL windows → survives
#   ④ graceful disconnect → 200; attestation chain hash-links throughout
#
# Against the reference provider (default):
#   cargo run -p groove-provider -- --port 6465 &   then   ./soft-hard-lease-demo.sh
# Against live burble on a dev machine:
#   TARGET_HOST=127.0.0.1 TARGET_PORT=6473 ./soft-hard-lease-demo.sh
#
# Exits non-zero on any assertion failure. Writes the captured chain to
# examples/evidence/lease-demo-attestations.json (gitignored).

set -euo pipefail

HOST="${TARGET_HOST:-127.0.0.1}"
PORT="${TARGET_PORT:-6465}"
BASE="http://${HOST}:${PORT}/.well-known/groove"
TTL_MS="${TTL_MS:-2000}"
EVIDENCE_DIR="$(dirname "$0")/evidence"

pass=0; fail=0
ok()   { pass=$((pass+1)); echo "  PASS: $1"; }
bad()  { fail=$((fail+1)); echo "  FAIL: $1" >&2; }
need() { command -v "$1" >/dev/null || { echo "missing dependency: $1" >&2; exit 2; }; }
need curl
need python3

json_get() { # json_get <json> <python-expr over d>
  python3 - "$1" "$2" <<'EOF'
import json, sys
d = json.loads(sys.argv[1]); print(eval(sys.argv[2], {"d": d}))
EOF
}

echo "== ① discover: GET ${BASE}"
manifest="$(curl -sf "$BASE" -H 'Accept: application/groove+json')"
service_id="$(json_get "$manifest" 'd["service_id"]')"
caps="$(json_get "$manifest" '", ".join(sorted(d["capabilities"].keys()))')"
echo "  provider: ${service_id}; capabilities: ${caps}"
[ -n "$caps" ] && ok "provider offers a real capability set (${caps})" || bad "no capabilities offered"

consumer() { # consumer <lease-json-or-empty>
  local lease="$1"
  if [ -n "$lease" ]; then
    printf '{"groove_version":"1","service_id":"lease-demo","service_version":"0.1.0","mode":"active","capabilities":{},"consumes":[],"lease":%s}' "$lease"
  else
    printf '{"groove_version":"1","service_id":"lease-demo","service_version":"0.1.0","mode":"active","capabilities":{},"consumes":[]}'
  fi
}

echo "== ② soft connect (ttl ${TTL_MS}ms) → lapse → zero-residue expiry"
soft_resp="$(curl -sf -X POST "$BASE/connect" -H 'Content-Type: application/json' \
  -d "$(consumer "{\"mode\":\"soft\",\"ttl_ms\":${TTL_MS}}")")"
soft_handle="$(json_get "$soft_resp" 'd["handle"]')"
soft_mode="$(json_get "$soft_resp" 'd.get("lease",{}).get("mode","<none>")')"
[ "$soft_mode" = "soft" ] && ok "soft lease accepted and echoed" || bad "lease not echoed: $soft_resp"

sleep "$(python3 -c "print((${TTL_MS}+600)/1000)")"
soft_status="$(curl -s -o /dev/null -w '%{http_code}' -X POST "$BASE/disconnect" \
  -H 'Content-Type: application/json' -d "{\"handle\":\"${soft_handle}\"}")"
[ "$soft_status" = "410" ] && ok "lapsed soft handle answers 410 Gone (expiry IS consumption)" \
  || bad "expected 410 for lapsed soft handle, got ${soft_status}"

echo "== ③ hard connect (ttl ${TTL_MS}ms) → heartbeat ≥3 TTL windows"
hard_resp="$(curl -sf -X POST "$BASE/connect" -H 'Content-Type: application/json' \
  -d "$(consumer "{\"mode\":\"hard\",\"ttl_ms\":${TTL_MS}}")")"
hard_handle="$(json_get "$hard_resp" 'd["handle"]')"
beats=$((7))
for i in $(seq 1 $beats); do
  sleep "$(python3 -c "print(${TTL_MS}/2/1000)")"
  hb="$(curl -s -o /dev/null -w '%{http_code}' "$BASE/heartbeat?handle=${hard_handle}")"
  [ "$hb" = "204" ] || bad "heartbeat $i/{$beats} expected 204, got ${hb}"
done
ok "hard lease heartbeaten across $(python3 -c "print(${beats}*${TTL_MS}/2/${TTL_MS})") TTL windows without being reaped"

echo "== ④ graceful disconnect + attestation chain"
disc_status="$(curl -s -o /dev/null -w '%{http_code}' -X POST "$BASE/disconnect" \
  -H 'Content-Type: application/json' -d "{\"handle\":\"${hard_handle}\"}")"
[ "$disc_status" = "200" ] && ok "hard connection disconnects gracefully (200)" \
  || bad "expected 200 graceful disconnect, got ${disc_status}"

chain="$(curl -sf "$BASE/attestations" || echo '[]')"
mkdir -p "$EVIDENCE_DIR"
printf '%s\n' "$chain" > "$EVIDENCE_DIR/lease-demo-attestations.json"
echo "  chain written to ${EVIDENCE_DIR}/lease-demo-attestations.json"

python3 - "$chain" <<'EOF' && ok "attestation chain: lease-expired(residue=0) present; hash-linkage intact" || exit_code=$?
import json, sys
records = json.loads(sys.argv[1])
if isinstance(records, dict):
    records = records.get("attestations", [])
assert records, "empty attestation chain"
expiries = [r for r in records if r.get("event") == "groove:lease-expired"]
assert expiries, f"no groove:lease-expired in {[r.get('event') for r in records]}"
assert all(r.get("residue") == 0 for r in expiries), "expiry with nonzero residue"
for a, b in zip(records, records[1:]):
    assert b["prev_hash"] == a["hash"], f"chain broken between {a['event']} and {b['event']}"
EOF
if [ "${exit_code:-0}" != "0" ]; then bad "attestation chain checks failed"; fi

echo
echo "== evidence summary (PROOF-NEEDS 'The bar') =="
echo "  real capability:            ${service_id} offers ${caps}"
echo "  soft groove:                lease expired to zero residue, 410 thereafter"
echo "  hard groove:                heartbeat-refreshed across ≥3 TTL windows"
echo "  teardown/attestation:       graceful disconnect 200; chain hash-linked"
echo "  result:                     ${pass} passed, ${fail} failed"
[ "$fail" -eq 0 ]
