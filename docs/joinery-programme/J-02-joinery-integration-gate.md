<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# J-02 · The joinery integration gate

| | |
|---|---|
| **Repository** | `metadatastician/groove` (hosts the gate; cleave and spline call it) |
| **Phase** | 0 — foundations |
| **Depends on** | J-01 (interface points IP-1..n) |
| **Blocks** | J-03, J-04, J-05 |
| **Owner decision** | no |
| **Size** | large (2–3 PRs) |

## Role

You are an autonomous coding agent working in a checkout of
`metadatastician/groove`. You build boring, honest test infrastructure: a gate
that fails when the system is broken and reports `PENDING` when a required
piece does not exist yet. A green run that could have been green with the
feature deleted is a defect you are expected to prevent.

## Mission

Build **one runnable gate** that stands all three repositories up together at
their pinned refs and asserts the joinery's cross-layer invariants
end-to-end — discovery, session lifecycle, leases, token boundary, plane
carriage, teardown/residue, attestation continuity, noninterference — with
negative controls for every failure mode, and with an explicit, counted
`PENDING` state for scenarios blocked on prompts that have not landed.

## Ground truth (verified 2026-09-29 — re-verify)

- groove: `groove-provider` (reference provider, `provider/src/lib.rs`) runs on
  loopback; `tests/conformance.rs` implements 16 `conf_l*` tests including the
  lease block; the CLI has `probe`, `check-compat`, `mesh`. Runbook and limits
  are in `docs/BETA-RUNBOOK.md`. `cargo test --locked --workspace` is the
  workspace gate.
- cleave: `cargo test --locked --all-targets` (18 debug / 17 release),
  `cargo test --locked --doc` (compile-fail linearity), `cargo test --locked
  --release`, `bash tests/check_proofs.sh <evidence-dir>`. The kernel API is
  `cleave::{Surface, Handle, Lease, Stage, posture_of, …}`.
- spline: `bash tests/check_alignment.sh /path/to/groove-checkout` reruns
  criterion (c); `tests/e2e.sh` currently fails with an "unmet gate" message,
  which is intentional and must become a real runner only after promotion.
- Nothing runs the three together today; there is no evidence bundle format
  and no scenario vocabulary.
- Estate rules: ≤5 workflows per repo, each exercising a real artifact;
  evidence over claims; no `continue-on-error` on substantive checks.

## Read first

`spec/JOINERY-ICD.adoc` and `registry/joinery-pins.json` (from J-01) ·
`spec/CONFORMANCE.adoc` · `spec/SPEC.adoc` §4–§6 · `docs/BETA-RUNBOOK.md` ·
`provider/tests/*.rs` · cleave `docs/PROOF-NEEDS.adoc` +
`docs/standards/RANKED-OWNERSHIP-CLEAVE.adoc` · spline
`docs/status/BETA-ACCEPTANCE.adoc` + `docs/alignment/voice-signal-plane.adoc`.

## Deliverables

1. **Scenario framework** at `tests/joinery/` (Bun + shell, matching the
   existing `scripts/*.mjs` + `provider/tests` style):
   - `tests/joinery/README.md` — scenario vocabulary, evidence format, how to
     add one, what `PENDING` means.
   - `tests/joinery/lib/` — checkout/build helpers that honour
     `registry/joinery-pins.json` (refs and digests), start/stop the provider,
     drive cleave's test binaries, and capture raw bytes as hex.
   - `tests/joinery/scenarios/*.mjs` — one file per scenario, each declaring
     `id`, `interface_point` (IP-n from the ICD), `requires` (prompt IDs whose
     deliverables it needs), and `assert(ctx)`.
   - `tests/joinery/gate.sh` — the entry point. Flags: `--all`, `--scenario
     <id>`, `--allow-pending` (exits 0 only when every non-pending scenario
     passes and prints the PENDING count), `--evidence <dir>`. Default (no
     `--allow-pending`) exits non-zero if any required scenario is PENDING.
2. **Scenario set** (minimum; map each to the ICD, not to guesses):
   - *Positive*: discovery + manifest negotiation (JSON, and A2ML if the
     provider serves it) → connect with a soft lease → expiry → zero residue
     attested; connect with a hard lease → heartbeats across ≥3 TTL windows →
     graceful disconnect; session-as-cleave-surface (mint/teardown order via
     cleave's audit and the provider's attestation agree); Bebop control-plane
     framing through the session once S-04 lands; attestation hash chain
     continuous across connect/lease-expired/disconnect; teardown order
     children-before-parent on both sides.
   - *Negative*: incompatible connect → `409`, no handle minted; forged
     handle → rejected without mutation; stale/expired handle → `410`;
     double disconnect → `410`; malformed lease → `400`; truncated/unknown
     Bebop variant → strict rejection, never defaulted values; rank-violating
     adoption → construction refuses; foreign-surface handle → `Err`, receipt
     preserved; second teardown → does not compile (compile-fail is part of
     the gate, invoked through cleave's doc test); corrupted hash chain →
     detection.
   - *Noninterference*: run the same logical payload under two dial postures
     (soft vs hard; and cleave S1 vs S2 once C-01 lands) and assert **byte
     equality** of the encoded payload, plus a schema-level check that the
     framing cannot carry posture/rank fields. A text search of one capture is
     not evidence (spline PROOF-NEEDS says so); this scenario must compare two
     captures.
3. **Evidence bundle**: every run writes
   `tests/joinery/evidence/<UTC-date>-<run-id>/` containing `scenarios.json`
   (per-scenario status + assertion counts), `raw/*.hex` for captured bytes,
   `logs/`, and `pins.json` (the resolved refs/digests actually used).
   Committed evidence is optional; the runner must always produce it and the
   PR must include the summary table.
4. **CI**: one job in groove's `ci.yml` that runs
   `bash tests/joinery/gate.sh --allow-pending` on PRs and reports the PENDING
   table. When J-03/S-04/C-01 have landed, a follow-up commit flips the job to
   `gate.sh --all` with zero PENDING — record that flip and its date in the
   job comment.
5. **Sibling entry points**: one shell file in each sibling repo that shells
   into the gate with its own checkout (`cleave/scripts/joinery-gate.sh`,
   `spline/tests/joinery-gate.sh`), documented as "the gate lives in groove;
   this is the sibling entry point." Do not duplicate scenario logic.

## Acceptance criteria

```sh
cd groove
bash tests/joinery/gate.sh --allow-pending          # exit 0, prints PENDING table
bash tests/joinery/gate.sh --scenario <one-id>      # single scenario runs
bash tests/joinery/gate.sh --all                    # expected non-zero while blockers open: PENDING count printed, no false PASS
# deliberately break something (e.g. point a pin at a wrong digest) and show the gate fails
```

Each scenario must be demonstrated to **fail** when its invariant is violated
(record the demonstration in the PR body: how broken, observed output).

## Evidence to capture

A `tests/joinery/evidence/<date>/` bundle from a full run; the broken-input
demonstration; the PENDING table with the prompt ID blocking each row.

## Non-goals / do not do

- Do not add `continue-on-error`, `|| true`, or a "skip on failure" path.
- Do not implement spline's framing or cleave's lifecycle inside the gate:
  the gate drives the real artifacts at their pinned refs. Test doubles are
  allowed only for transports, never for the invariant under test.
- Do not mark a scenario PASS because it is unreachable; unreachability is
  PENDING with a named blocker.
- Do not create a fourth repository for the harness (ADR 0008).

## Stop conditions

- A scenario needs consumer repos (burble/gossamer): keep it PENDING with
  `requires: [J-03]` and say so; do not vendor consumer code.
- A pin cannot be resolved in this environment: gate reports
  `BLOCKED(pin)` — distinct from PENDING and from FAIL — and exits non-zero.

## Definition of done

Gate merged and wired into CI; ≥15 scenarios with ≥7 negative controls;
noninterference scenario compares two captures; every scenario maps to an ICD
interface point; PENDING rows each name a prompt ID; sibling entry points
exist; PR body contains the run table, the broken-input demonstration, and the
list of what remains **open** for J-03, S-04 and C-01 to close.
