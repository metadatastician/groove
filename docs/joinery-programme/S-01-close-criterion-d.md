<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# S-01 · Close criterion (d): acceptance harness, fixtures, reconciliation

| | |
|---|---|
| **Repository** | `metadatastician/spline` |
| **Phase** | 1 — live-pairing chain |
| **Depends on** | nothing |
| **Blocks** | J-03, S-02 |
| **Owner decision** | no |
| **Size** | medium (1–2 PRs) |

## Role

You are an autonomous coding agent working in a checkout of
`metadatastician/spline`. Spline is deliberately thin (ADR 0005: docs, ADRs
and conformance fixtures only — **no `src/`** until all four promotion
criteria hold). You build the *acceptance machinery*, not the wire layer.

## Mission

Turn `docs/status/BETA-ACCEPTANCE.adoc`'s seven numbered requirements for
criterion (d) into an executable acceptance harness plus a versioned capture
format and fixture conventions, so that when J-03 runs the real Burble↔Gossamer
pairing, its output is machine-checkable rather than a folder of logs someone
reads and believes. Also define the reconciliation procedure that keeps the
recorded mapping honest when a capture disagrees with it.

## Ground truth (verified 2026-09-29 — re-verify)

- Criterion (d) is **OPEN**: no live capture with posture provably absent from
  bytes. The 2026-09-07 local capture is recorded honestly as evidence, not
  proof (`docs/alignment/voice-signal-plane.adoc` §Local consumer capture).
- `tests/check_alignment.sh` reproduces criterion (c) by shelling into a
  groove checkout (`scripts/check-bebop-alignment.mjs` +
  `registry/bebop-voice-signal-alignment.json` +
  `spec/conformance/bebop/voice_signal_frames.hex`).
- `tests/e2e.sh` currently exits with an actionable "unmet gate" message —
  the right shape for a gate that is not yet reachable; imitate it.
- Strictness is part of the alignment (burble ruling A4): truncated or
  malformed input is rejected by both consumers; a frame declaring more bytes
  than it carries must never decode to defaulted values.
- `LeaveReason` values are cross-plane aligned (Voluntary=0 … ServerShutdown=4)
  with burble's `room_event` plane.
- No `src/`, no `.machine_readable/`, and `docs/status/TEST-NEEDS.adoc` is
  stale template residue claiming artifacts that do not exist (see S-06).

## Read first

`docs/decisions/0002-parallelise-means-plane-multiplexing.adoc` ·
`docs/decisions/0004-typed-cap-token-boundary.adoc` ·
`docs/decisions/0005-promotion-criteria.adoc` ·
`docs/alignment/voice-signal-plane.adoc` · `docs/status/BETA-ACCEPTANCE.adoc` ·
`docs/status/PROOF-NEEDS.adoc` · `tests/check_alignment.sh` · `tests/e2e.sh` ·
groove's `scripts/check-bebop-alignment.mjs` and its `.test.mjs` (the checker
pattern to imitate).

## Deliverables

1. **`docs/acceptance/CAPTURE-FORMAT.adoc`** — the versioned capture format
   (`capture-v1`). Normative fields per capture: scenario id; endpoint
   revisions (repo, commit, build command, tool versions); configuration;
   the transport boundary **actually exercised** (named, not assumed);
   timestamps; the raw bytes (hex) with provenance; decoders applied and their
   results; token events (mint, consume, expiry, rejection with reason);
   teardown/residue observations from each endpoint; the posture comparison
   table (two captures, byte equality); negative controls with the observed
   rejection. Nothing in the format may be inferred — if a field was not
   observed, it is `absent` and the harness says so.
2. **`tests/acceptance/accept.mjs`** (+ `.test.mjs` self-tests) — the
   validator. Given a capture directory and a groove checkout, it:
   - validates the capture against `capture-v1` (schema + invariants);
   - re-decodes every recorded frame with groove's checker (and with any
     second decoder available locally) and fails on disagreement;
   - asserts the seven BETA-ACCEPTANCE requirements are each evidenced by a
     named field/artifact path — an unevidenced requirement is a FAIL, not a
     footnote;
   - asserts noninterference by comparing **two** captures across postures and
     by a schema-level check that the framing cannot carry posture/rank
     fields; a single-capture text search is explicitly not accepted;
   - asserts negative controls exist and *failed in the expected way*
     (a control that "rejected" with the wrong error is a FAIL).
   No capture present → exit non-zero with an actionable unmet-gate message
   naming J-03.
3. **`tests/acceptance/fixtures/`** — the fixture conventions: new recorded
   frames, when a capture produces them, enter as `.hex` with a provenance
   header (producing repo, commit, generator command, date); the mapping file
   in groove is the SSOT; spline holds fixtures only if groove's checker
   validates them identically (state the rule and where duplication is
   forbidden).
4. **`docs/acceptance/RECONCILIATION.adoc`** — the procedure for a capture
   that disagrees with the recorded mapping: reproduce, isolate
   (schema vs generator vs transport), decide (mapping bug → fix mapping +
   version bump; consumer bug → record in the consumer repo and keep the
   criterion OPEN; semantics change → new mapping version), and record.
   Include the versioning rule spline must adopt before "stable"
   (additive minor / breaking major; tag reassignment forbidden; compatibility
   rule stated).
5. **Wire the validator into `tests/e2e.sh`** so the repo has one entry point,
   and keep `bash tests/check_alignment.sh <groove>` as the (c) reproduction.
6. **README/status honesty**: `docs/status/TEST-NEEDS.adoc` either gets
   rewritten as a truthful inventory of what these scripts do, or is deleted
   in S-06's sweep — coordinate, do not leave both true and false versions.

## Acceptance criteria

```sh
bash tests/check_alignment.sh /path/to/groove-checkout   # (c) still reproduces
bun tests/acceptance/accept.mjs --self-test              # format + validator self-tests pass
bun tests/acceptance/accept.mjs --capture tests/acceptance/fixtures/empty  # FAILS with unmet-gate message
# feed a synthetic capture with a wrong decoder result   → FAIL, wrong reason reported
# feed a capture missing a negative control              → FAIL, missing control named
```

## Evidence to capture

Self-test output; the unmet-gate demonstration; a one-page note in the PR body
mapping each of BETA-ACCEPTANCE's seven requirements to the validator check
that enforces it.

## Non-goals / do not do

- **No `src/`, no framing implementation, no shadow serialiser.** The validator
  parses and checks; it must not become the thing it validates.
- Do not lift from `protocol-squisher`/`squisher-corpus`.
- Do not relax ADR 0005 or claim any criterion as met; S-02 does the recording.
- Do not add workflows beyond the budget (≤5); fold into `alignment.yml` or
  the existing entry points.

## Stop conditions

- If the two-capture posture comparison cannot be defined without a real
  session (because postures change the session, not the payload), stop and
  write down precisely which observable the comparison must use — this is a
  legitimate design question for J-03, and the format must say how the two
  captures are made comparable (same logical payload, different posture).
- If a required field cannot be observed by either endpoint, the format marks
  it `absent`; do not weaken the requirement to make the format close.

## Definition of done

`capture-v1` format, validator, self-tests and reconciliation procedure
merged; `tests/e2e.sh` is a real entry point that fails for the right reason;
the seven requirements each name their enforcement check; PR body states
exactly what remains open (criterion (d) itself, until J-03/S-02).
