<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# C-05 · Proofs and the refinement boundary

| | |
|---|---|
| **Repository** | `metadatastician/cleave` |
| **Phase** | 3 — cleave track |
| **Depends on** | C-04 (general dial + G-4 graph) |
| **Blocks** | J-04 |
| **Owner decision** | no |
| **Size** | large (2–4 PRs) |

## Role

You are an autonomous coding agent working in a checkout of
`metadatastician/cleave`. This repository's best artifact is its honesty
ledger; your job is to keep it the best artifact while making the proof layer
cover the surface that now exists.

## Mission

Extend the Idris2 mirrors to the general staircase, the peer protocol and the
G-4 graph; then write down — precisely and once — the **refinement boundary**:
what is proved, what is tested at runtime, and what is assumed. Any claim that
the Rust does what the proofs say must either be backed (checked refinement,
extraction, or an explicit simulation argument with evidence) or struck from
the record.

## Ground truth (verified 2026-09-29 — re-verify)

- Current mirrors: `proofs/Cleave/Kernel/{Types,Posture,HandleLinearity,ResourceCleanup,DMMultiset}.idr`
  + `proofs/Checks/{AcceptLinearity,RejectDuplication,RejectLeak}.idr`; ipkg
  `cleave-proofs.ipkg`; `bash tests/check_proofs.sh <evidence-dir>`; CI
  `kernel.yml`; zero `believe_me`/`assert_total`/`postulate` (a real gate).
- The honesty text (README/PROOF-NEEDS/PROOF-STATUS) already says: "The five
  Idris modules prove statements about their inductive models… No extraction or
  checked Rust refinement exists."; "Cardinality of a rank list is not an
  identity-uniqueness proof; a size-minus-length residue measure is not an
  observation of OS resources."; "General DM accessibility… remain open."
- Reuse policy: audit `hyperpolymath/echo-types` before any echo-typed
  obligation; `tropical-resource-typing` for `MinPlus`/residue→identity;
  `ephapax`'s `RegionStack` for order-dual discipline. Reuse where applicable;
  extend upstream with proofs if not; then cross-document.
- Proof methodology notes from the estate: rebuild against actual types, never
  lift phantom gossamer modules; ipkg `--typecheck` only; negative fixtures as
  first-class citizens.

## Read first

`proofs/README.adoc` · `docs/PROOF-NEEDS.adoc` · `docs/status/PROOF-STATUS.adoc` ·
`proofs/Cleave/Kernel/*` and the negative fixtures · C-04's
`docs/INVARIANT-MATRIX.adoc` · C-02's peer proofs and the ADR for its link
assumption · `tests/check_proofs.sh` · `cleave-proofs.ipkg`.

## Deliverables

1. **Mirror extension**: Idris modules for the general staircase (stages and
   their order, posture totality/monotonicity for N stages, atomic re-dial),
   the peer protocol (joint completion/rupture, from C-02), and the G-4 graph
   property (a hard session owning soft sessions still yields well-founded
   descent). Each with: a positive theorem, a falsifier fixture that fails the
   build when the property is broken, and a `%default total` discipline and
   zero-trust-base policy enforced by CI.
2. **Refinement boundary document** `docs/REFINEMENT.adoc` (or an ADR plus
   that doc): for each proved property, state the correspondence to runtime
   code — exact function/type names — and its status: *checked refinement* (if
   you build one), *differential/conformance evidence* (tests + enumeration
   sizes + what they cover), or *assumption*. Include the known mismatches
   explicitly (e.g. rank list cardinality vs identity uniqueness; residue
   measure vs OS resources; deliberate `mem::forget`; release-mode drop-bomb
   absence). This document is normative for the repo's claims: if a claim is
   not on it, it may not appear in README/READINESS.
3. **One experiment worth doing**: a small, honest step toward refinement —
   for example, a model-based test generator that produces random
   staircase/graph traces, runs the Idris model interpreter (or a checked
   transcription) and the Rust implementation on the same trace, and compares
   observable outcomes. If you cannot run Idris interactively in CI, use a
   checked-in trace corpus with expected outcomes and state the limitation.
   Either way: evidence, with the coverage stated.
4. **Proof-debt registration**: every residual (general DM accessibility,
   TS-6 binder if not adopted, wire-level TS-7 if not closed, peer link
   assumptions, extraction/refinement gap) gets a row in
   `docs/PROOF-NEEDS.adoc` with owner and blocking condition — no folklore.
5. **Ledger + README updates**: `docs/status/PROOF-STATUS.adoc` regenerated
   with the new numbers, README status line paraphrased from the ledger, not
   from ambition.

## Acceptance criteria

```sh
bash tests/check_proofs.sh /absolute/path/to/evidence      # all modules typecheck
grep -rn 'believe_me\|assert_total\|postulate' proofs/      # zero
cargo test --locked --all-targets                            # runtime suite unaffected/green
# negative fixture demonstration: break the modelled property → the fixture FAILS the build
# refinement experiment: run it, and show a deliberate property violation is caught
```

## Evidence to capture

Typecheck transcripts; the negative fixtures' failures; the trace-corpus run
(coverage stated); the boundary document; the new ledger rows with owners.

## Non-goals / do not do

- Do not claim extraction or refinement you did not build. "No refinement
  claimed" is an acceptable, permanent state — it just has to be written.
- Do not lift phantom gossamer proofs; rebuild against actual cleave types.
- Do not extend the trust base (`believe_me` etc.) to make a theorem pass.
- Do not let a status page report a percentage of "theorems proven" — the
  estate has burned that bridge; report obligations with their state.

## Stop conditions

- If an echo-typed obligation appears, stop and run the reuse audit first
  (policy), recording the outcome in PROOF-NEEDS before implementing.
- If the refinement experiment cannot be made deterministic in CI, keep it as
  a locally reproducible script with recorded output and say so; do not
  fabricate coverage.

## Definition of done

Mirrors extended with falsifiers; refinement document merged and README/STATUS
paraphrase it; proof-debt rows registered with owners; CI gates
(`check_proofs.sh`, trust-base grep) green; PR body lists commands, results,
and the precise remaining residuals.
