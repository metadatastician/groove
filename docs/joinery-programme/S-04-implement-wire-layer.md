<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# S-04 · Implement the wire layer: plane multiplexing, opaque tokens, posture-blind by construction

| | |
|---|---|
| **Repository** | `metadatastician/spline` (with the joint gate exercised from groove) |
| **Phase** | 2 — spline track |
| **Depends on** | S-02 (promotion permits `src/`), S-03 (versioned mapping + fixtures), J-01 (interface points) |
| **Blocks** | S-05, J-04 |
| **Owner decision** | no |
| **Size** | large (3–5 PRs) |

## Role

You are an autonomous coding agent working in a checkout of
`metadatastician/spline`. This is the prompt ADR 0005 kept locked until the
four criteria held. You may now create `src/` — and you must keep spline's
defining virtue: it aligns to what consumers already speak and **never invents
a serialiser of its own**.

## Mission

Implement the reference wire layer: one groove session carrying independent
typed planes over one connection with no head-of-line blocking (the precise
meaning of "parallelise", ADR 0002), opaque authority tokens on the wire with
linear handles at the endpoints (ADR 0004), and a framing whose type system
cannot carry posture, rank or lifecycle witnesses (TS-7) — plus the
conformance suite that makes all three claims falsifiable.

## Ground truth (verified 2026-09-29 — re-verify)

- ADR 0002: "parallelise" = multiplexing independent typed planes of one groove
  session over one connection, no head-of-line blocking. ADR 0003: consumers
  (burble) own their `.bop` schemas; spline owns only the alignment mapping,
  never a copied schema. ADR 0004: cap tokens = signed manifest records
  (groove ADR 0010); handles = wire-opaque strings, linear at each endpoint;
  posture never in bytes.
- Control plane = Bebop (typed, binary, reliable/ordered); media plane =
  WebRTC/RTP (lossy, low-latency, no head-of-line block). spline *carries*
  those; it does not encode media.
- The recorded control-plane frames are real Bebop output from burble's codecs
  (groove `spec/conformance/bebop/voice_signal_frames.hex`), and gossamer has
  an independent strict Zig decoder. Any framing must be decodable by an
  implementation that shares nothing with yours.
- groove's session states (§4), lease rules (§4.6) and attestation chain (§5)
  are the session facts a plane belongs to; groove knows nothing about bytes.
- The squisher caution: the data-form is where `protocol-squisher` stalled. A
  copy/sync seam, a second representation of truth, or lifted code are the
  failure modes to avoid.

## Read first

`docs/decisions/0002..0005` · `docs/alignment/voice-signal-plane.adoc` ·
S-03's `tests/conformance-v1/` and `docs/alignment/IMPLEMENTATIONS.adoc` ·
groove `spec/JOINERY-ICD.adoc` (IP-4, IP-5, IP-6), `spec/SPEC.adoc` §4/§5,
`registry/bebop-voice-signal-alignment.json` · cleave
`docs/standards/RANKED-OWNERSHIP-CLEAVE.adoc` §TS (TS-7).

## Deliverables

1. **ADR `docs/decisions/0006-framing-and-implementation-shape.adoc`** —
   records: the reference implementation language and crate shape (recommend a
   Rust crate `src/` with no `unsafe`, consistent with the estate's proven
   toolchain); the rule that the **contract**, not this crate, is normative;
   what consumers may implement independently; and the version negotiation
   between framing versions.
2. **`spec/FRAMING.adoc`** — the normative framing contract:
   - session header (framing version, plane count, limits) and plane
     descriptors (plane id, kind: control|media, reliability/ordering class);
   - stream multiplexing: per-plane stream ids, interleaving rules, and the
     **no-head-of-line-blocking** requirement stated as a testable property
     (stall one plane; others must still make progress);
   - control plane: Bebop payloads verbatim, reliable/ordered within the plane;
   - media plane: opaque payload carriage, no re-encoding by spline; declared
     loss/latency properties are the transport's, not spline's, and spline must
     not claim otherwise;
   - token carriage: authority tokens and handles are opaque byte strings at
     this layer; no parsing, no semantics; length limits;
   - **prohibited fields**: no posture, rank, ownership, lease-state or
     lifecycle witness may appear anywhere in the encoding; state that the
     framing is posture-blind by construction and name the test that checks it;
   - error taxonomy: framing errors (malformed, unsupported version, unknown
     plane kind, limit exceeded) with the requirement that malformed input is
     rejected, never defaulted;
   - version negotiation and backward/forward-compatibility rules.
3. **`src/`** — the reference implementation: framing codec, plane multiplexer,
   token carriage, strict decode paths, and a conformance runner for S-03's
   fixtures. Constraints: no `unsafe`, zero or minimal dependencies (justify
   any), `#![forbid(unsafe_code)]`, `#![deny(missing_docs)]`, error types that
   distinguish framing errors from transport errors, and no global mutable
   state.
4. **Tests** (`src/` unit + `tests/`):
   - every S-03 fixture through the implementation with matching verdicts;
   - **no-HOL-blocking**: a stalled or slow plane while another proceeds
     (deterministic, no sleeps-as-proof: use controlled scheduling/channels);
   - interleaving, reordering *within a lossy plane*, duplication, and
     backpressure;
   - malformed: truncated header, length lie, unknown plane kind, unknown
     framing version, limit exceeded — all rejected;
   - **posture-blindness by construction**: encode the same logical payload
     from two dial postures (as S-02/S-03 define the comparison) → identical
     bytes; plus a schema-level assertion that prohibited fields cannot be
     constructed (e.g. the header/plane types have no such fields, checked by
     an exhaustive-construction test, not a comment);
   - a **no-new-serialiser guard**: a test or CI check that the framing does not
     define payload schemas and that control-plane payloads are carried
     verbatim.
5. **Fuzzing**: a cargo-fuzz target or an equivalent deterministic harness for
   the decoder, with the corpus committed (or the harness + seed corpus).
6. **CI**: `cargo fmt --all --check`, `cargo clippy --all-targets -D warnings`,
   `cargo test`, docs build; keep the repo to ≤5 workflows by folding into the
   existing alignment workflow or replacing a template-era one (S-06).
7. **Docs and honesty**: `README.adoc`/`ARCHITECTURE.md` updated from seed status
   to the real state (with S-02's review linked); `docs/status/READINESS.adoc`
   rewritten around what now runs; `docs/status/TEST-NEEDS.adoc` replaced by a
   truthful inventory (or deleted in S-06 — do not keep both).
8. **Joint evidence**: J-02's IP-4/IP-5/IP-6 scenarios flip to PASS using this
   implementation and the recorded/real session path; record the run.

## Acceptance criteria

```sh
cargo build --locked
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
cargo test --locked --release
bash tests/conformance-v1/run.sh --impl-a <burble-or-gossamer> --impl-b ./target/release/<bin>
bash groove/tests/joinery/gate.sh --scenario planes --scenario token-boundary --scenario noninterference
```

Demonstrate the malformed-input rejections with raw hex in the PR body, and the
no-HOL-blocking test failing when you deliberately serialize the planes.

## Evidence to capture

Fixture verdict tables against ≥2 implementations; the HOL demonstration; raw
malformed frames + rejections; the posture-blind comparison; the fuzz run
summary; the J-02 scenario output.

## Non-goals / do not do

- **No new payload format, no `.bop` rewrite, no schema copies** (ADR 0003).
- No lifting from `protocol-squisher`/`squisher-corpus`.
- No posture/rank/lease data in bytes, ever — and no "we'll add it later"
  hooks; the fields must be unrepresentable.
- Do not claim media-plane properties spline does not implement; do not encode
  media.
- Do not make groove's normative text depend on this crate; the coupling is
  the ICD (J-01).

## Stop conditions

- If a required property cannot be tested deterministically (e.g. true
  loss/latency on a real transport), state the assumption and test the
  contract boundary instead; do not fabricate a proof of the transport.
- If implementing the framing would require changing the recorded alignment
  or one of S-03's version rules, stop — that is a contract change and goes
  through the ICD's change procedure.

## Definition of done

Framing spec + reference implementation + tests + fuzz harness merged; ≥2
implementations agree on the fixture suite; HOL, malformed, and
posture-blindness demonstrations recorded; ADR 0005's promotion already
recorded by S-02; stability debt from S-03 closed or explicitly BLOCKED; docs
and ledgers truthful; PR body lists commands, results, files, closed and open
items.
