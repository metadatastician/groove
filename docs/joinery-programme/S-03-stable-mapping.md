<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# S-03 · Version the mapping; two independent implementations must agree

| | |
|---|---|
| **Repository** | `metadatastician/spline` (mapping lives in groove's registry; spline owns the contract) |
| **Phase** | 2 — spline track |
| **Depends on** | S-02 (promotion permits implementation; the mapping work itself is docs+fixtures) |
| **Blocks** | S-04, S-05 |
| **Owner decision** | no |
| **Size** | medium (1–2 PRs across spline + groove) |

## Role

You are an autonomous coding agent working across checkouts of
`metadatastician/spline` and `metadatastician/groove`. The contract is the
document, not one codebase's behaviour: your job is to make that checkable.

## Mission

Meet ADR 0005's stability rule: give the alignment mapping a version and a
compatibility rule, and prove the contract travels by running the **same**
conformance fixtures through **two independent** implementations of the plane
that share no generated code. Record the provenance of each implementation so
"independent" is a verified statement, not a hopeful one.

## Ground truth (verified 2026-09-29 — re-verify)

- The machine-readable mapping is `groove/registry/bebop-voice-signal-alignment.json`
  (keys: plane, schema_owner, consumers, wire, field_kinds_note, variants,
  cross_plane) and it carries no version field today.
- Recorded frames: `groove/spec/conformance/bebop/voice_signal_frames.hex`
  (produced by burble's codecs, vendored with provenance).
- Implementations that exist and could serve as the two:
  1. burble's generated Elixir codec (`mix bebop.generate` output — burble owns
     the schema, ADR 0003);
  2. gossamer's hand-written strict Zig decoder
     (`src/interface/ffi/src/bebop_voice_signal.zig`);
  3. groove's checker (`scripts/check-bebop-alignment.mjs`) — a decoder in JS.
  Independence must be *shown*: no shared generator output, no copy of a
  generated table; document each implementation's provenance and the command
  that produced/verified it. The checker is the natural third, but two
  suffices — choose the two with the strongest independence claim and say why.
- Strictness is aligned: truncated/malformed input rejected by both consumers;
  no defaulted values (burble ruling A4). `LeaveReason` cross-plane values
  (0..4) are part of the alignment.
- `wire`/`field_kinds_note` in the mapping file describe layout; a change to
  any variant tag, field layout or enum value is a semantic change.

## Read first

`docs/decisions/0005-promotion-criteria.adoc` §What "stable" would additionally
need · `docs/alignment/voice-signal-plane.adoc` · groove's
`registry/bebop-voice-signal-alignment.json`, `scripts/check-bebop-alignment.mjs`
(+ `.test.mjs`), `spec/conformance/bebop/voice_signal_frames.hex` ·
`groove/spec/SPEC.adoc` §3.2 (version constraints, for vocabulary).

## Deliverables

1. **Mapping version** in groove's alignment file: `mapping_version` (start at
   `1`), `compatibility_rule` (prose, normative in spline), and a changelog
   block in the file or in spline's `docs/alignment/` that records what a
   version bump means: additive variant → minor; changed tag/layout/enum →
   major and requires consumer co-release; no tag reassignment ever. Groove's
   checker validates the field is present and monotone against the recorded
   frames (it cannot invent history; it checks the current declaration).
2. **Conformance fixture suite, implementation-agnostic**
   (`spline/tests/conformance-v1/`): the fixture set that both implementations
   must pass — every recorded frame, plus targeted cases: each variant tag;
   each enum value incl. all five `LeaveReason`s; truncated header; declared
   length > carried bytes; unknown tag; zero-length payload where the layout
   requires fields; a frame whose declared field kinds disagree with the
   layout. Fixture format is machine-readable (JSON manifest + hex payloads)
   with provenance.
3. **Runner** `spline/tests/conformance-v1/run.sh` (or `.mjs`, matching
   S-01's style) that:
   - builds/reuses implementation A (burble codec or gossamer decoder) and
     implementation B;
   - executes the fixture set through both;
   - asserts identical accept/reject verdicts and identical decoded values on
     accepts — a disagreement is a FAIL naming both implementations and the
     fixture;
   - records implementation provenance (repo, commit, build command) into the
     run output;
   - is honest when an implementation cannot be built in the environment:
     `BLOCKED(impl-A)` is not a pass.
4. **Independence note** `docs/alignment/IMPLEMENTATIONS.adoc`: one row per
   implementation — provenance, generator or hand-written, what it shares with
   the others (nothing, ideally), and the command that reproduces it.
5. **Groove CI**: extend the existing `spec-consistency` job to run the new
   fixture suite's groove-side check (the checker already lives there), and
   document in spline that the authoritative checker is groove's.

## Acceptance criteria

```sh
bun groove/scripts/check-bebop-alignment.mjs
bun groove/scripts/check-bebop-alignment.test.mjs
bash spline/tests/conformance-v1/run.sh --impl-a <cmd> --impl-b <cmd>
# negative control: mutate one fixture byte → both implementations reject; runner FAILS if either accepts
# negative control: change one mapping version rule → checker/runner FAIL
```

## Evidence to capture

The runner's per-fixture verdict table for both implementations; provenance
lines; the two negative-control demonstrations; the mapping-version changelog.

## Non-goals / do not do

- Do not write a new serialiser or a framing implementation (S-04 does the
  framing; this prompt is contract + fixtures + runner).
- Do not generate a fixture set from one implementation and call it
  independent — fixtures come from burble's actual codecs or are hand-built
  from the layout with provenance.
- Do not put version/handshake bytes in the payload fixtures; encode version as
  framing, per ADR 0002/0004 (see S-04).
- Do not "fix" a disagreement by choosing the friendlier verdict; a
  disagreement is the finding.

## Stop conditions

- If only one implementation can be built in this environment, land the suite,
  mark the other `BLOCKED`, and leave the stability rule **unmet** with the
  blocker named — stability is not achieved by one implementation.
- If the divergence is in the schema itself (burble owns it, ADR 0003), stop
  and record it as a consumer defect with the evidence; do not edit a schema
  you do not own.

## Definition of done

Mapping versioned with a compatibility rule; fixture suite + runner merged;
independence note written; both gates green or explicitly BLOCKED; ADR 0005's
stability rule recorded as met **only** if the two-implementation run is
green — and the PR says which.
