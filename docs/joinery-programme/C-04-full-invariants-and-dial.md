<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# C-04 · Full dial and full invariants: staircase/zipper, RC-1..13, TS-3/5/6/7, G-4, scheduler

| | |
|---|---|
| **Repository** | `metadatastician/cleave` (G-4 graph involves a consumer session, e.g. via C-01) |
| **Phase** | 2 — cleave track |
| **Depends on** | C-01 (session adapter), C-02 (two-peer wire) |
| **Blocks** | C-05, J-04 |
| **Owner decision** | **yes** — the scheduler decision and the general-staircase semantics |
| **Size** | large (3–5 PRs) |

## Role

You are an autonomous coding agent working in a checkout of
`metadatastician/cleave`. You are converting a kernel that demonstrates *one*
point into a surface with a general dial and an invariant matrix where every
row is either tested or explicitly out of scope with a reason.

## Mission

Generalise and complete: the two-stage staircase becomes an N-stage
well-founded staircase with a zipper (`dial_down`/`dial_up`/focus/path), every
RC-1..13 invariant gets a positive and a negative test (or a documented
out-of-scope reason), TS-3/TS-5/TS-6 stop being degenerate, G-4 stops being
vacuous (a real Hard-owns-Soft graph exists via C-01's adapter), and the lease
scheduler question is decided one way or the other.

## Ground truth (verified 2026-09-29 — re-verify)

- Current stages: `S1`/`S2`, `posture_of(Stage)` the only `Posture`
  constructor; `PermSet::{Narrow,Wide}.subset_of`; `Preempt::{Immediate,
  WithNoticeMs(5000)}`. Design doc: the dial is "a zipper walking a
  well-founded staircase"; `dialDown/dialUp/focus` are sketched in
  `docs/architecture/CLEAVE-ENGINE-DESIGN.adoc`.
- `docs/PROOF-NEEDS.adoc`: G-1 closed for the kernel path (tree postorder DM
  argument) with the **general DM accessibility residual open**; G-4 open —
  "RC-1..13 are vacuously green outside the kernel, because no real
  Hard-owns-Soft owns-graph exists in any consumer yet"; TS-3 degenerate
  (per-operation floors await the full surface); TS-5 single-process only;
  TS-6 partial (awaits the Ephapax `let!` layer); TS-7 vacuous in-process.
- Leases are caller-ticked: "the kernel does not provide an autonomous
  scheduler". `docs/status/READINESS.adoc` warns a global ever-growing Surface
  is not a bounded-memory service.
- Estate policy (PROOF-NEEDS "Reuse don't reinvent"): any echo-typed
  obligation must first audit `hyperpolymath/echo-types`; reuse if applicable,
  extend upstream *with proofs* if not, then cross-document. `tropical-resource-typing`
  and `ephapax` are named reuse sources (TS-6 awaits Ephapax's `RegionStack`/
  `let!` discipline).
- `tests/kern.rs` names: kern_1..7 + `order_dual_holds_across_mixed_release_paths`;
  `tests/boundaries.rs` has 8 named boundary tests; every increasing labelled
  tree on seven nodes is enumerated (720) in one of them — the finite
  enumeration style to extend, not to inflate.

## Read first

`docs/architecture/CLEAVE-ENGINE-DESIGN.adoc` ·
`docs/standards/RANKED-OWNERSHIP-CLEAVE.adoc` (invariants + falsifier tables,
§TS) · `docs/PROOF-NEEDS.adoc` (G-1/G-4, O-1..O-9, reuse table) ·
`docs/status/{READINESS,PROOF-STATUS}.adoc` · `src/*.rs` · `tests/{kern,boundaries}.rs` ·
C-01's adapter (for the G-4 graph) · `hyperpolymath/echo-types` and `ephapax`
checkouts if available (read-only audit).

## Deliverables

1. **ADR `docs/decisions/0005-general-staircase.adoc`**: N-stage staircase
   model (stages provably ordered and well-founded; the bottom is ⊥); the
   zipper API (`dial_down`, `dial_up`, focus, path); how posture is derived for
   arbitrary stages (total, monotone per TS-2, atomic on re-dial per TS-5);
   and how soft/hard map onto descent depth (the design's original framing).
2. **Zipper implementation** generalising `Stage`, keeping `S1`/`S2`
   behaviour identical (existing tests unchanged) and adding at least two more
   stages with distinct, justified readings. Re-dial while handles are live
   must be atomic (TS-5): no intermediate posture observable — provide a test
   that would catch a torn read (e.g. concurrent dial + operation with a
   deterministic interleaving harness; if true concurrency is untestable
   deterministically, use a single-threaded schedule with an explicit
   observation hook and say exactly what that proves and does not).
3. **Invariant matrix** `docs/INVARIANT-MATRIX.adoc`: one row per RC-1..13 and
   TS-1..7 — statement · test name(s) · positive/negative · status
   (tested / kernel-only / out-of-scope + reason) · falsifier. No empty cells;
   "out of scope" requires the reason and (where a proof would be needed) a
   PROOF-NEEDS row.
4. **Tests to fill the matrix**:
   - **G-4**: a real Hard-owns-Soft graph — via C-01's adapter or a purpose-
     built consumer-like harness — where a hard session owns soft sessions;
     assert rank descent, teardown order and residue across the graph, and make
     the falsifier (a rank-violating or double-owned edge) fail loudly. Update
     the PROOF-NEEDS G-4 row with what the graph now demonstrates.
   - **TS-3** per-operation floors: at least one operation gated at a shallow
     stage and ungated deeper, with a refusal test at the shallow stage.
   - **TS-6**: audit `ephapax` and either adopt a `let!`-style linear binder
     (with an ADR and no new dependencies if possible) or record the descope
     with the reason and the residual in PROOF-NEEDS.
   - **TS-7**: over the wire (coordinated with C-02/J-03; if the wire leg is not
     in this session, keep the row "wire leg: J-03/J-02" and say so).
   - Extend the finite-enumeration style: e.g. all increasing trees on eight
     nodes, or all two-stage interleavings of mint/heartbeat/dial/teardown —
     with the enumeration size stated.
5. **Scheduler decision (owner decision)**: either (a) an opt-in
   `Scheduler`/`Runtime` helper that drives `tick` for callers who ask for it
   (documented as convenience, not a semantic change), or (b) a recorded
   decision that leases remain caller-ticked, with the contract documented in
   `docs/KERNEL.adoc`/DIAL doc and the beta checklist updated. Either way the
   choice is written, not implied.
6. **Bounded-memory contract**: define and test what happens to a
   long-running Surface (released-node receipt metadata / append-only audit
   growth). Options: documented bound + `Surface::compact()`-style API, or an
   explicit "not a bounded-memory service; use bounded-lifetime surfaces per
   context" contract with a test that measures growth on a workload and
   records the number. Do not leave the READINESS warning unaddressed.
7. **Ledger updates**: PROOF-NEEDS rows for G-4/TS-3/TS-5/TS-6/TS-7 and O-9,
   the invariant matrix, READINESS. Everything dated.

## Acceptance criteria

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo test --locked --doc
cargo test --locked --release
bash tests/check_proofs.sh /absolute/path/to/evidence
# demonstrations (pick at least three, record them):
#   rank violation in the G-4 graph → FAIL
#   torn re-dial observation → FAIL
#   shallow-stage gated operation allowed → FAIL
#   enumeration reduced to a happy path → FAIL
```

## Evidence to capture

Matrix with test names; enumeration sizes; the G-4 graph diagram (text) and its
audit excerpt; the demonstration outputs; measured memory growth; the
scheduler decision record.

## Non-goals / do not do

- Do not claim the general DM accessibility proof; it stays a named residual
  unless you actually complete it (with the echo-types audit first).
- Do not rewrite existing kernel tests to make the new model pass; add tests,
  keep old ones green, and if a semantic change is needed, take it through an
  ADR.
- Do not add async runtimes or heavyweight deps for the scheduler without the
  owner's sign-off.
- Do not fill "out of scope" without a reason and a ledger row.

## Stop conditions

- **Owner decision** on the scheduler and on any normative TS change.
- If a TS row cannot be made testable without a wire (TS-7) or a research
  result (general DM), stop at the row and record, rather than inventing a
  weaker property and calling it satisfied.

## Definition of done

General staircase + zipper merged with old behaviour intact; invariant matrix
complete with tests or recorded out-of-scope reasons; G-4 graph real;
scheduler and memory contracts written and tested; ledgers updated; PR body
lists commands, results, the matrix diff, and open items (C-05's proof work).
