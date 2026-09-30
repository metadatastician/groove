<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# G-06 · Beta operations: load, multi-process, restart/rollback rehearsal

| | |
|---|---|
| **Repository** | `metadatastician/groove` (plus evidence from a chosen target environment) |
| **Phase** | 3 — operations |
| **Depends on** | G-04 (client surface decided), J-03 (live pairing captured) |
| **Blocks** | J-05 |
| **Owner decision** | **yes** — the deployment owner and environment are named by the owner |
| **Size** | large (a session plus environment runs) |

## Role

You are an autonomous coding agent working in a checkout of
`metadatastician/groove`. Beta gate 4 is an operations gate, not a coding
gate: named owner, real environment, rehearsed rollout and rollback. Your job
is to produce the evidence and refuse to approve on anyone's behalf.

## Mission

Close the operational boundary `READINESS.md` leaves open — "production load,
multi-process integration and target-specific deployment" — with measured
evidence from the real environment: load behaviour at and beyond the documented
limits, multi-process integration, a rehearsed restart and rollback, and a
named deployment owner. No claim without a measurement; no approval without the
owner.

## Ground truth (verified 2026-09-29 — re-verify)

- Documented limits (`docs/BETA-RUNBOOK.md`): 128 concurrent HTTP exchanges,
  1,024 live sessions, 5-second overall exchange deadline, 4,096 retained
  audit records; service IDs ≤128 B, versions ≤64 B, consumes lists ≤64;
  at session capacity connect returns `503`; saturated socket capacity closes
  new exchanges; single-exchange HTTP/1.x, transfer encoding unsupported and
  rejected; incomplete/duplicate Content-Length rejected; browser responses
  capped at 64 KiB / 2 s. Hard leases: 15 s TTL renewed every 5 s; provider
  expiry follows three missed TTL windows.
- The provider "binds only loopback" by default and rejects non-loopback Host
  and Origin; unsigned manifests are the default; self-signature is not an
  identity pin; `GROOVE_SIGNING_KEY` must be injected, never on a command line.
- Audit state is in memory; restart loses sessions and the hash chain; there
  is no persistence migration. A hash chain without an external anchor is not
  tamper-proof storage.
- `READINESS.md` reproduction block lists the full local verification
  (cargo build/test/release/clippy, Bun tests, web-ext lint, alignment checks,
  proofs, `gh actions-lock --no-fix`, `cargo audit --deny warnings`).
- "Local test configurations are not external targets" — that is J-05's
  business; this prompt is the *operations* half, not the external-trials half.

## Read first

`docs/BETA-RUNBOOK.md` · `READINESS.md` · `SECURITY.md` ·
`docs/POSITIONING.adoc` (scope: Level 1 launch)
· `provider/src/lib.rs` (limits + shutdown paths) · `provider/tests/*` ·
`docs/dev-notes/known-issues.md` · J-03's capture bundle and S-05's capacity
measurements.

## Deliverables

1. **Load evidence** `docs/ops/LOAD-<date>.md` + a reproducible harness
   (`tests/load/`): measure at 50 %, 100 % and 150 % of each documented limit
   (exchanges, sessions, audit retention, sizes), record latency distribution
   and error statuses, and verify the documented behaviours actually occur
   (`503` at capacity, socket saturation closes exchanges, oversized inputs
   rejected). Any deviation from the runbook is either a fixed bug or a
   runbook correction in the same PR.
2. **Multi-process integration**: two or more provider processes (different
   ports/bands) plus one consumer, exercising discovery over the probe band,
   independent session tables, and graceful degradation when one process
   stops. Record what is discovered, what is not, and why (the reference
   provider is per-process bookkeeping; say exactly that).
3. **Rollout rehearsal** in the owner-named environment:
   - start from the previously approved artifact, deploy the candidate, verify
     discovery + connect + lease + disconnect + attestation continuity;
   - **restart** the provider (SIGINT path per the runbook) and verify old
     tokens fail and clients rediscover/reconnect;
   - **rollback**: stop the candidate, restart the approved version, verify
     discovery and fresh negotiation; retain the approved binary/config and the
     key pin; do not restore bearer tokens from logs;
   - capture the full transcript (commands, versions, timestamps, observations,
     failures) in `docs/ops/ROLLOUT-REHEARSAL-<date>.md`.
4. **Owner record**: `docs/ops/DEPLOYMENT-OWNER.md` with the named
   owner/environment, the change procedure, the rollback authority, and the
   approval state (default: *not approved* until the owner signs).
5. **Remote CI green** on the merge candidate, with the run identifiers
   recorded; plus `cargo audit --deny warnings` output against a dated RustSec
   database revision.
6. **READINESS update**: gate 4 row closed only if the rehearsal evidence
   supports it and the owner approves; otherwise update with the exact
   remaining item.
7. **Known-issues close-out** for anything load/multi-process work exposes
   (`docs/dev-notes/known-issues.md`).

## Acceptance criteria

```sh
cargo build --locked --workspace --release
bash tests/load/run.sh --profile limits        # writes results to docs/ops/evidence/
# restart test:
#   start provider, connect, SIGINT, restart, prove the old token fails (410/404) and reconnect succeeds
# rollback test recorded in the rehearsal doc with commands and outputs
gh run list --branch <candidate> --limit 5     # remote CI green, ids recorded
```

## Evidence to capture

Measured numbers (not estimates) per limit; the multi-process transcript; the
restart/rollback transcript with timestamps; the owner record; CI run ids;
RustSec database revision; every deviation between observed behaviour and the
runbook.

## Non-goals / do not do

- Do not self-approve production beta; gate 4 is an owner decision.
- Do not test against a local configuration and present it as the deployment
  rehearsal — the run must happen where the owner says it does.
- Do not add persistence, sessions, or tokens to logs to "make restart
  easier"; the runbook's no-persistence property is deliberate.
- Do not weaken the loopback/origin protections to enable a remote test.

## Stop conditions

- **Owner decision / environment** required before the rehearsal; if the
  environment is not available, deliver the load + multi-process evidence and
  the rehearsal *plan*, and leave gate 4 OPEN with the blocker named.
- If observed behaviour contradicts the runbook's limits, stop and fix the
  contract or the code — do not silently record a new number as if it were
  documented.

## Definition of done

Load + multi-process evidence recorded; rollout rehearsal transcript recorded
(or an explicit environment blocker recorded); owner record created; remote CI
green with ids; READINESS gate 4 updated truthfully; PR body lists commands,
results, files, and the exact approval state.
