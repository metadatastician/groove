<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# S-05 · Interop matrix, fault injection, beta acceptance

| | |
|---|---|
| **Repository** | `metadatastician/spline` (consumer evidence via burble/gossamer checkouts, framework-neutral) |
| **Phase** | 3 — spline track |
| **Depends on** | S-04 |
| **Blocks** | J-04, J-05 |
| **Owner decision** | no |
| **Size** | large (2–4 PRs) |

## Role

You are an autonomous coding agent working in a checkout of
`metadatastician/spline`. You are the adversary: your job is to break the wire
layer S-04 built and record exactly how it behaved when broken.

## Mission

Take the wire layer through the conditions it will actually meet — mixed
versions, an explicit JSON opt-out, encode-failure fallback, loss, reordering,
duplication, truncation, capacity pressure — and produce the interop matrix and
fault record that beta acceptance requires. Then record spline's beta status
honestly: what is now validated, and what still requires external targets.

## Ground truth (verified 2026-09-29 — re-verify)

- Deliberate design facts to test, not assume: burble keeps JSON as an explicit
  opt-out **and** as the automatic encode-failure fallback; decode of both
  planes is always accepted, so mixed-version peers keep working
  (`docs/alignment/voice-signal-plane.adoc`).
- Capable clients request `wire_format: bebop` at channel join; legacy
  WebSocket delivery normalises SDP back to JSON. A default flag alone never
  established the wire path — the J-03 capture had to name its boundary.
- Strict decoders: truncated/malformed input rejected by both consumers; a
  frame declaring more bytes than it carries must never decode to defaults
  (burble ruling A4).
- `LeaveReason` values are cross-plane aligned with `room_event` (0..4).
- spline BETA-ACCEPTANCE requires: home-context use and six diverse external
  trials for a CRG beta grade; the acceptance contract itself is the seven
  capture requirements (J-03) plus the stability rule (S-03).
- Toolchain pins in the estate: Rust 1.97.1, Bun 1.3.14 (groove's CI). Use the
  consumers' own pins for their legs.

## Read first

`docs/status/BETA-ACCEPTANCE.adoc` · `docs/decisions/0002..0006` ·
`docs/alignment/voice-signal-plane.adoc` · S-03's fixture suite and runner ·
S-04's `spec/FRAMING.adoc` and implementation · groove `spec/SPEC.adoc` §4
(lease/renew) and `docs/BETA-RUNBOOK.md` (limits vocabulary).

## Deliverables

1. **Interop matrix** `docs/status/INTEROP-MATRIX.adoc` + a runner
   (`tests/interop/`) covering at least:
   - new↔new, new↔old, old↔new with JSON opt-out, old↔old (baseline);
   - decode of both planes accepted in every pairing;
   - the encode-failure fallback path: force an encode failure and record what
     the peer observes, whether ordering/reliability assumptions held, and
     whether any message was silently dropped or duplicated;
   - framing version negotiation: mixed framing versions either negotiate down
     or refuse — never partially parse;
   - the `LeaveReason` cross-plane values across the matrix.
   Each cell is PASS/FAIL/BLOCKED with the command and revision that produced
   it. No "expected to pass" cells.
2. **Fault-injection suite** (`tests/fault/`, transport-level injector):
   loss, reordering, duplication, truncation, corruption, and stall — for
   control plane and media-plane carriage separately. Assertions:
   - control plane: defined reliability/ordering preserved when the transport
     provides it; on violation the session reports an error rather than
     silently continuing with defaulted values;
   - media plane: spline must not add ordering/reliability; injected loss
     passes through as loss, and spline makes no claim it cannot test;
   - no-HOL-blocking under stall (cross-check S-04's test at transport level);
   - reconnect/discovery interaction: what a consumer must do after a fault,
     and what evidence shows it did it.
   Keep the injector deterministic (seeded) and commit the seeds.
3. **Capacity and limits**: measured numbers for frame size, plane count,
   concurrent streams, and buffer behaviour under backpressure; the limits are
   stated in `spec/FRAMING.adoc` and enforced (not just documented). A limit
   without a test is a comment.
4. **Beta acceptance record** `docs/status/BETA-ACCEPTANCE.adoc` update:
   each requirement marked with its evidence path and date; the CRG-related
   items that cannot be satisfied locally remain **OPEN** and say so; the
   grade claim is not upgraded without external trials (J-05).
5. **Home-context use**: a runnable "spline in the estate" scenario
   (provider + one consumer + the framing) with a recorded transcript — the
   dogfooding evidence a CRG C grade needs, distinct from external trials.

## Acceptance criteria

```sh
cargo test --locked
bash tests/interop/run.sh --seeds tests/interop/seeds.txt
bash tests/fault/run.sh --seeds tests/fault/seeds.txt
# demonstrate: remove the truncation rejection → fault suite FAILS
# demonstrate: serialize planes deliberately → HOL assertion FAILS
```

## Evidence to capture

The matrix table; per-seed fault logs with the observed vs declared behaviour;
capacity measurements with the machine/toolchain recorded; the home-context
transcript; the updated BETA-ACCEPTANCE with explicit OPEN rows.

## Non-goals / do not do

- Do not claim external validation; no prompt in this pack can.
- Do not "fix" a consumer's fallback behaviour in spline — record it, and fix
  it in the consumer repo only under a session that targets that repo.
- Do not weaken strictness to make the matrix greener.
- Do not claim media-plane guarantees (no encoding, no jitter buffer, no
  congestion control is spline's).

## Stop conditions

- If a fault path requires a real WebRTC stack to test meaningfully, mark it
  BLOCKED with the reason and the minimal harness that would close it; a
  simulated transport may test *carriage*, never a claim about RTP itself.
- If interop reveals a schema/consumer defect, stop and record it (ADR 0003:
  the consumer owns the schema).

## Definition of done

Matrix + fault suite + capacity tests merged with seeds; BETA-ACCEPTANCE
updated with dated evidence and OPEN rows; home-context transcript recorded;
PR body states the measured numbers, the demonstrations, and precisely which
claims remain unvalidated.
