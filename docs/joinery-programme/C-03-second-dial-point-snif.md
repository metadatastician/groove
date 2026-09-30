<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# C-03 · Second dial point: the SNIF/wasmtime sandbox stage

| | |
|---|---|
| **Repository** | `metadatastician/cleave` |
| **Phase** | 2 — cleave track |
| **Depends on** | nothing (the staircase generalisation is C-04's, but a second stage can start here) |
| **Owner decision** | **yes** — adding a heavyweight dependency (wasmtime) and defining the third stage's semantics is an owner-level call |
| **Size** | large (2–3 PRs) |

## Role

You are an autonomous coding agent working in a checkout of
`metadatastician/cleave`. The kernel was designed so every deeper dial point
reuses the same lifecycle machinery. You are the test of that claim: a real
sandboxed point, wired end-to-end, crash-isolated, with the same rank/handle/
teardown discipline.

## Mission

Wire the SNIF dial point — native → WASM → wasmtime, crash-isolated — as a
real cleave stage: a guest trap becomes an error while the host survives; the
staircase, rank, linear handle and teardown machinery are the same ones the
kernel already proves; the sandbox's cost (what it gives up relative to the
direct point) is recorded, not hidden (the "OF" in cleave's design).

## Ground truth (verified 2026-09-29 — re-verify)

- The design names the SNIF middle as "native → WASM → wasmtime sandbox; a
  guest trap becomes `{:error,_}`, the host survives", and notes it is
  "Realized today in the `snifs` repo — *one detent*, not the dial."
- The kernel's non-goals say explicitly: "No WASM/sandbox stage (that is the
  *third* dial point)" — i.e. this prompt is the sanctioned next step, not a
  scope violation.
- KERN-7/TS-1..7 make posture a function of *dial position*: a new stage must
  come with a posture reading, and TS-2 says permissions widen with descent
  depth. A sandbox looks like *narrowing* — do **not** paper over this; resolve
  it explicitly (see below).
- Cleave's crate is `#![forbid(unsafe_code)]` and dependency-free beyond std;
  wasmtime is a large dependency tree. The ADR must decide how the feature is
  gated (recommended: optional cargo feature, off by default, CI job with the
  feature on).
- `docs/status/PROOF-STATUS.adoc` lists TS-6 as partially open (awaits the
  Ephapax `let!` layer) and TS-3 as degenerate — this stage is where
  per-operation floors start to matter.

## Read first

`docs/architecture/CLEAVE-ENGINE-DESIGN.adoc` (the dial, WF/OF, soft/hard,
open questions) · `docs/KERNEL.adoc` (non-goals + KERN-1..7) ·
`docs/standards/RANKED-OWNERSHIP-CLEAVE.adoc` (§TS, esp. TS-1/2/3/5) ·
`docs/architecture/TRANSMUTE-SECURITY.adoc` · `src/staircase.rs`,
`src/lib.rs` · `docs/PROOF-NEEDS.adoc` (O-9 row) ·
the `snifs` repo (read-only reference; do not vendor it).

## Deliverables

1. **ADR `docs/decisions/0004-sandboxed-dial-point.adoc`** deciding, with
   reasons:
   - the stage's position and name (e.g. `S3`; keep the existing two stages'
     behaviour bit-for-bit);
   - **the posture question**: how a sandboxed point reads under TS-1/TS-2.
     Two honest resolutions exist — (a) the stage is a *different* axis
     (mediation) composed with dial depth, so posture gains a fourth component
     ("mediation") and TS-2 extends to it; or (b) the sandbox is a *shallower
     local* reading of the same dial, with the permission widening applying to
     what the host delegates to the guest. Choose, justify, and update the TS
     falsifier table accordingly — with the owner's sign-off, since this edits
     normative TS text;
   - the dependency gating (feature name, default off), the supported wasmtime
     version pin, and the sandbox limits (fuel/epoch, memory cap, allowed
     host functions);
   - what "one detent, not the dial" from `snifs` means for reuse: align, do
     not vendor.
2. **Implementation** `src/snif/` behind the feature: guest module loading,
   invocation with fuel/epoch interruption and a memory limit, trap → typed
   error (`Result`, not panic), host unaffected, guest cannot reach the host's
   owns-arena except through the declared host-function surface.
3. **Tests** `tests/snif.rs` (feature-gated):
   - happy path: guest invocation succeeds through a minted handle at stage
     `S3`; teardown consumes it; residue 0 asserted;
   - **crash isolation**: guest panics/traps → error returned; host continues
     and its audit is intact (assert before/after);
   - runaway guest: infinite loop stopped by fuel/epoch; memory cap enforced;
     host unaffected;
   - malformed/invalid module → refused before any handle is minted;
   - the same lifecycle discipline: children before parents on teardown with a
     guest-contained subtree; foreign handle refused without mutation;
   - determinism: same input + same fuel → same outcome, twice.
4. **OF record** — a table in the ADR or `docs/architecture/`: what the
   sandbox gives up relative to S1/S2 (latency, direct memory sharing,
   host-function fidelity, debugger/observability, etc.) and what it buys
   (isolation). No silent losses.
5. **Ledger updates**: `docs/PROOF-NEEDS.adoc` O-9 row (kernel enforcement now
   spans two points) and any TS row the ADR touched; `docs/status/READINESS.adoc`
   updated with what now runs and the new dependency's CI cost;
   `docs/KERNEL.adoc`'s non-goal line updated to note the stage exists (or a
   new `docs/DIAL.adoc` if the kernel doc is the wrong home — decide).
6. **CI**: one job building/testing the feature (cargo cache optional), inside
   the ≤5-workflow budget (fold into `kernel.yml`).

## Acceptance criteria

```sh
cargo test --locked --all-targets                          # base: unchanged, green
cargo test --locked --all-targets --features snif          # + sandbox suite
cargo clippy --locked --all-targets --features snif -- -D warnings
cargo test --locked --release --features snif
bash tests/check_proofs.sh /absolute/path/to/evidence      # mirrors still green
# demonstrations:
#   remove the trap handler → crash-isolation test FAILS (host poisoned)
#   remove the memory cap  → runaway test FAILS / or is shown unbounded
```

## Evidence to capture

Trap-to-error transcripts; the fuel/epoch interruption evidence; memory-limit
enforcement; the OF table with measured overhead (numbers, machine recorded);
the posture decision with the owner sign-off; CI job run.

## Non-goals / do not do

- Do not vendor or copy `snifs`; align to it and cite it.
- Do not change S1/S2 behaviour, KERN semantics or public API without an ADR.
- Do not make the feature default-on or mandatory; the base crate stays
  buildable with std only.
- Do not claim proof-to-runtime refinement for anything here.
- Do not use `unsafe` (workspace rule) — wasmtime's safe API is the contract.

## Stop conditions

- **Owner decision** required before anything lands that edits normative TS
  text (the posture question).
- If the dependency's version pin conflicts with the estate's toolchain policy,
  stop and record the conflict rather than pinning to a floating version.
- If crash isolation cannot be demonstrated for a case the design claims
  ("guest trap becomes `{:error,_}`, host survives"), record the failing case
  and do not ship a claim around it.

## Definition of done

Stage implemented behind a feature, tests + crash-isolation evidence recorded,
posture decision signed, OF table honest, ledgers updated, CI green with the
feature on; PR body lists commands, results, files, and what remains open
(e.g. full-surface TS obligations still owned by C-04).
