<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# J-03 · The live Burble–Gossamer pairing capture

| | |
|---|---|
| **Repository** | evidence lands in `metadatastician/spline`; consumers are `metadatastician/burble` and `metadatastician/gossamer`; ledgers in all three joinery repos |
| **Phase** | 1 — live-pairing chain (the keystone) |
| **Depends on** | S-01 (capture format + validator), G-01 (capability-scoped handles), C-01 (session-as-surface) |
| **Blocks** | S-02, J-04 |
| **Owner decision** | **yes** — needs the owner's environment, consumer checkouts and agreement to run the pairing |
| **Size** | large (this is a session plus a consumer-side runner, not a code sprint) |

## Role

You are an autonomous coding agent running the one experiment the whole
programme is waiting for: a **real** Burble↔Gossamer session, captured to the
S-01 format, decoded by both consumers, and validated by S-01's harness. Your
job is to produce evidence, not enthusiasm; anything not observed is
`absent` and the ledgers keep saying OPEN.

## Mission

Close criterion (d) of spline ADR 0005 and groove's beta gate 1, and produce
the cleave TS-7 noninterference observation over actual bytes: capture a live
pairing where a typed capability token crosses the boundary — opaque on the
wire, linear at each endpoint — with forged/stale rejection, both-sided
cleanup, posture-blind bytes compared across two postures, and malformed
controls retained with their failures.

## Ground truth (verified 2026-09-29 — re-verify; these are the traps)

- The prior 2026-09-07 capture (spline ledger) exercised Gossamer's native
  library against Burble's Endpoint under test configuration via Burble's
  `server/integration/check_gossamer_pairing.sh`. It is recorded as evidence,
  **not** promotion: it did not bind the groove lease to the voice-signaling
  authorisation path, and it does not close (d).
- Spline BETA-ACCEPTANCE warns: Burble's `SignalingChannel` encodes Bebop for
  PubSub and decodes back to JSON before pushing to the WebSocket client, so
  "the `:bebop` default flag alone does not establish Bebop on that
  client-facing wire". **Name and verify the transport boundary you actually
  exercise**; a capture that conflates the two is the exact failure this
  prompt exists to prevent.
- Burble#189 note: main did not compile 2026-08-04→08-07 after a merge damaged
  the generated codec; treat any stale "verified on main" claim with care and
  pin the revision you actually build.
- groove's provider rejects malformed leases with `400`, refuses refresh on
  soft leases (`409`), answers `404` for unknown handles, and `410` for
  expired/consumed handles (§4.6). Those are the rejection observables.
- cleave's TS-7 is vacuous in-process; the evidence must be over the wire.

## Read first

spline: `docs/status/BETA-ACCEPTANCE.adoc` (the 7 requirements),
`docs/acceptance/CAPTURE-FORMAT.adoc` + `tests/acceptance/accept.mjs` (S-01),
`docs/alignment/voice-signal-plane.adoc`.
groove: `spec/JOINERY-ICD.adoc` (J-01), `spec/SPEC.adoc` §4/§5,
`docs/BETA-RUNBOOK.md`, `READINESS.md`.
cleave: `docs/PROOF-NEEDS.adoc` (TS-7/O-6 rows), `docs/standards/RANKED-OWNERSHIP-CLEAVE.adoc` §TS.
burble/gossamer: the pairing runner and the native session connect path
(find them; do not assume paths).

## Deliverables

1. **Runner** — the smallest reproducible script that stands the pairing up:
   burble Endpoint + gossamer native library, both at recorded commits,
   with the groove provider in the loop for the typed token unless the
   consumer path genuinely passes the token through the two endpoints (state
   which, and why, from the code). Command sequence, configuration and tool
   versions recorded verbatim. If the runner lives in a consumer repo, it is a
   minimal test script in that repo with its own CI job; keep consumer changes
   additive and reviewable.
2. **Capture bundle** in spline, `docs/alignment/captures/<UTC-date>/`,
   conforming to `capture-v1` (S-01) and passing
   `bun tests/acceptance/accept.mjs --capture <dir>`. It must evidence, each
   with a named artifact:
   - endpoint revisions, build commands, configuration, transport boundary;
   - soft and hard connections through real capability operations;
   - control-plane bytes captured and decoded by **both** consumers;
   - the opaque token boundary: valid in, linear handle at each endpoint,
     with forged and stale token rejection and the observed statuses;
   - each endpoint's teardown and expiry evidence, including residue
     (provider `residue: 0` attestation; consumer-side cleanup;
     cleave adapter residue where C-01 is in the path);
   - posture/rank witnesses absent, demonstrated by **two** captures of the
     same logical payload under different dial postures with byte equality,
     plus a schema-level field check — not a text search of one capture;
   - malformed/truncated controls: at least one frame declaring more bytes
     than it carries (must be rejected, never defaulted) and one unknown
     variant; keep both the passing and failing outputs.
3. **Ledger updates, same session, separate small PRs per repo**:
   - spline `docs/status/BETA-ACCEPTANCE.adoc` + `docs/alignment/` — criterion
     (d) marked satisfied **only if** the validator passes with zero
     unevidenced requirements; otherwise record exactly which requirement is
     unmet and leave it OPEN. Do not promote here (S-02 owns the review).
   - groove `READINESS.md` — gate 1 closed or narrowed, with the capture path.
   - cleave `docs/PROOF-NEEDS.adoc` — TS-7 row updated with the wire evidence;
     O-6/O-7 notes if the teardown path observed them (they do not close O-6/O-7
     by themselves; C-02 owns that).
4. **J-02 scenario flip**: the noninterference and token scenarios go from
   PENDING to PASS, with the evidence bundle referenced by path.

## Acceptance criteria

```sh
bun spline/tests/acceptance/accept.mjs --capture <capture-dir>   # zero FAIL, zero unevidenced requirement
bash groove/tests/joinery/gate.sh --scenario token-boundary --scenario noninterference
# both PASS on the captured/recorded evidence
# and a deliberate corruption of the bundle → validator FAILS (demonstrate once)
```

## Evidence to capture

Everything above, plus: the exact commands and their raw output; the two
cross-posture capture directories; the rejection transcript for forged/stale/
truncated inputs; the consumer cleanup observations; a written statement of
what the capture does **not** establish (media, NAT/TURN, audio, negotiate
timing/credentials, whole-transcript equality, universal noninterference).

## Non-goals / do not do

- Do not claim media-plane, NAT/TURN, audio or field-trial evidence.
- Do not claim a universal noninterference theorem from two captures.
- Do not edit consumer release processes or CI beyond an additive test job.
- Do not rebuild burble's codec or gossamer's decoder to make the capture
  cleaner; if a consumer is broken, record it and stop at OPEN.
- Do not mark any criterion satisfied in the same PR that produces its
  evidence — S-02 reviews and records promotion.

## Stop conditions

- **Owner decision required** to proceed at three points: choosing the target
  environment, running the pairing, and any consumer-repo change beyond the
  additive runner. Stop and report at each if the owner has not authorised it.
- If the transport boundary exercised turns out not to carry Bebop (burble's
  legacy JSON normalisation path), stop and report: the honest outcome is a
  narrower criterion, not a friendlier description.
- If S-01's validator cannot express a requirement, that is an S-01 defect;
  fix the format (with its self-tests) rather than hand-waving the capture.

## Definition of done

A validated capture bundle exists; every one of BETA-ACCEPTANCE's seven
requirements is either evidenced by a named artifact or explicitly recorded as
unmet; three ledgers updated truthfully; J-02 scenarios flipped; PR bodies
state commands, results, files, closed items and open items. Handoff to S-02:
the capture path plus the validator's summary table.
