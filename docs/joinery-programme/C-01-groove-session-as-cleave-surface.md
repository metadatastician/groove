<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# C-01 · A groove session as a cleave surface (the layering annex made real)

| | |
|---|---|
| **Repository** | `metadatastician/cleave` (in-repo sibling crate; groove's annex gets a status update) |
| **Phase** | 1 — live-pairing chain |
| **Depends on** | J-01 (interface points; read it first) |
| **Blocks** | J-03, C-04 |
| **Owner decision** | no |
| **Size** | large (2–3 PRs) |

## Role

You are an autonomous coding agent working in a checkout of
`metadatastician/cleave`. The kernel is real and small; you extend it with a
first *consumer-shaped* adapter, without weakening a single KERN invariant. You
write tests that would fail if the adapter were deleted.

## Mission

Implement the first half of the aspirational LAYERING annex: a real,
feature-gated adapter that presents a **groove session as a cleave surface** —
connect mints, heartbeat refreshes a hard lease, soft leases expire to zero
residue, disconnect linearly consumes and tears down through the staircase —
and prove it against a live `groove-provider`, updating the annex from
"aspirational" to "implemented for IP-2/IP-3 with evidence" (still
non-normative for Groove v0.x).

## Ground truth (verified 2026-09-29 — re-verify)

- cleave kernel: `Surface::new/mint/adopt/teardown/try_teardown/teardown_all/
  tick/heartbeat/dial/stage/posture/residue/audit`; stages `S1`/`S2`;
  `Lease::Soft{ttl}` / `Lease::Hard{ttl}`; `posture_of` is the only `Posture`
  constructor; TTLs are caller-ticked (no autonomous scheduler).
- groove §4.6 already speaks cleave's RC-6 vocabulary: soft expiry is a
  zero-residue wipe attested as `groove:lease-expired` with `"residue": 0`;
  hard refreshes via `GET /.well-known/groove/heartbeat?handle=` → `204`
  (unknown handle `404`), degradation after **three whole missed TTL
  windows**; expired-handle disconnect → `410`.
- groove's `spec/LAYERING.adoc` is explicitly aspirational; groove ADR 0007
  keeps cleave separate and non-blocking. Do not make groove's normative text
  depend on this adapter.
- cleave has no ADR series beyond ADR 0001; the adapter decision (crate vs
  feature, transport ownership, error mapping) deserves an ADR here.
- Raw-material note: the groove provider must be built and started as a child
  process in tests; groove's own suite already runs it in-process, and
  `docs/BETA-RUNBOOK.md` fixes the CLI surface and limits.

## Read first

cleave: `docs/KERNEL.adoc`, `docs/PROOF-NEEDS.adoc`,
`docs/standards/RANKED-OWNERSHIP-CLEAVE.adoc` (RC-1..13, TS-1..7),
`docs/architecture/CLEAVE-ENGINE-DESIGN.adoc`, `src/lib.rs`, `tests/kern.rs`.
groove: `spec/JOINERY-ICD.adoc` (J-01), `spec/SPEC.adoc` §4 (esp. §4.1, §4.2,
§4.5, §4.6), `spec/LAYERING.adoc`, `provider/src/lib.rs`,
`provider/tests/conformance.rs`, `docs/BETA-RUNBOOK.md`.

## Deliverables

1. **ADR `docs/decisions/0002-groove-session-adapter.adoc`** recording: artifact
   shape (recommended: a workspace crate `cleave-groove/` beside `src/`, or a
   `groove` cargo feature — choose one and justify), why the adapter is
   optional, what it does *not* own (discovery, negotiation, bytes), the
   error mapping, and the invariant mapping table.
2. **`cleave-groove` adapter** implementing the mapping table:
   | groove event | cleave operation |
   |---|---|
   | connect accepted | `mint` (session root), handle held as a `Handle` |
   | capability-scoped operation | ordinary handle use, rank-checked |
   | hard-lease heartbeat `204` | `heartbeat` refresh, TTL from the accepted lease |
   | hard lease degradation / soft expiry | `tick`-driven wipe; assert residue 0; consume the handle |
   | `410 Gone` on disconnect | handle already consumed; adapter reports `Err`, never a second consume |
   | disconnect `200` | `teardown` (linear consume) through the staircase, children before parents |
   | provider `409`/`400` on connect | no mint; adapter surfaces a structured refusal |
   Local posture is derived only via `posture_of(stage)` and never sent on the
   wire (TS-7). Offer `dial()` to move stage and document what per-stage
   permission/gating means for an adapter caller.
3. **Tests, `cleave-groove/tests/`**:
   - against an in-process fake transport *and* against a real
     `groove-provider` child process (built from the pinned groove ref in
     J-01's pin file; skip-with-reason if the binary is unavailable, but the
     CI job must build it);
   - soft lease: TTL elapses (caller tick), wipe observed, residue asserted 0,
     handle unusable afterwards, adapter audit matches the provider's
     attestation events;
   - hard lease: ≥3 TTL windows of heartbeats survive; heartbeat stops →
     degrade → wipe; never reaped while renewed;
   - teardown order: children before parents (assert from the adapter's audit
     log), and reverse-order attempts refuse without mutation;
   - foreign/stale/forged handles: `Err` with the receipt preserved, no
     mutation; second teardown unrepresentable (compile-fail test as in the
     kernel's doctest);
   - error mapping table exercised both directions (provider status ↔ adapter
     error variant).
4. **Annex status update in groove** (small PR): `spec/LAYERING.adoc` gains an
   evidence column or a status paragraph: IP-2 and IP-3 are implemented and
   tested *as an annex* by cleave's adapter at a named commit/date; the annex
   remains non-normative; the wire semantics of §4.6 remain exactly as
   specified. Do not edit §4.6's normative text.
5. **Ledger updates**: cleave `docs/PROOF-NEEDS.adoc` G-4 row (a real
   Hard-owns-Soft graph now exists — describe the graph the adapter creates
   and what remains open); `docs/status/READINESS.adoc` updated with the new
   evidence and the still-open items (O-6/O-7, TS-7 over the wire).

## Acceptance criteria

```sh
cargo build --locked --all-targets --workspace
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets          # kernel suite still 18/18 debug
cargo test --locked --doc                  # linearity compile-fail still passes
cargo test --locked -p cleave-groove       # adapter suite
bash tests/check_proofs.sh /absolute/path/to/evidence   # unchanged, still green
```

Plus: kill the provider mid-session in one test and show the adapter's tear-
down ends with asserted zero residue and no dangling handle.

## Evidence to capture

Command transcripts; the adapter audit excerpts showing teardown order and an
expiry wipe; the provider's attestation records for the same events; a short
table mapping each new test to the invariant (RC-n/TS-n/KERN-n) it exercises.

## Non-goals / do not do

- Do not make groove's normative protocol depend on cleave; no §-renumbering.
- Do not put posture, rank or lease-state into any bytes; the adapter is local.
- Do not implement groove's discovery, negotiation or framing; call the
  provider.
- Do not add an autonomous scheduler here (C-04 decides that).
- Do not weaken KERN tests or rename public kernel API without an ADR.

## Stop conditions

- If groove's provider is not buildable in the environment, land the in-process
  fake suite plus a CI job that builds the provider, mark the live leg
  `requires: J-02`, and say so — do not simulate the provider's semantics and
  call it live.
- If TS-2 (permissions widen with descent) conflicts with what a sandboxed or
  remote session should mean, stop and escalate to C-04's design; do not
  silently invent a third posture reading.

## Definition of done

Adapter + ADR + tests merged; annex evidence paragraph merged in groove;
`PROOF-NEEDS` and `READINESS` updated with dated evidence and explicit
open items; J-02's IP-2/IP-3 scenarios flip from PENDING to PASS; PR body
lists commands, results, files, and what remains open (O-6/O-7, TS-7 over the
wire).
