<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# C-02 · Two peers: joint completion (O-6) and clean-XOR-rupture (O-7) over a real wire

| | |
|---|---|
| **Repository** | `metadatastician/cleave` |
| **Phase** | 2 — cleave track |
| **Depends on** | nothing (C-04 consumes its results) |
| **Owner decision** | no |
| **Size** | large (2–4 PRs) |

## Role

You are an autonomous coding agent working in a checkout of
`metadatastician/cleave`. The single-process kernel is done and honest; this
prompt is where cleave stops being a local library and has to survive a peer
that dies mid-handshake.

## Mission

Close cleave's two distributed obligations over a real transport: **O-6**
(joint completion — clean-cleave is both peers at ⊥ with dual ack; one-sided is
half-closed residue) and **O-7** (clean-XOR-rupture — a lost final ack resolves
as bounded retransmit *or* escalation to survivor-owns-the-wipe, never
dangling). Build the minimal peer protocol, prove what can be proved, and
record precisely which assumptions remain.

## Ground truth (verified 2026-09-29 — re-verify)

- `docs/PROOF-NEEDS.adoc`: O-6/O-7 are **open**; "they need two peers and a
  wire, neither of which exists yet."
- Kernel facts to preserve: teardown is a tree postorder (children before
  parents, `planParentLast`), residue 0 at ⊥; RC-13's general form is the one
  at stake; `docs/status/PROOF-STATUS.adoc` records G-2's gossamer leg closed
  in `gossamer_groove_disconnect_typed` (children-first postorder, per-subtree
  residue-0 assertion, audit ring) — real precedent for the discipline.
- The kernel's own honesty limits: a size-minus-length residue measure is not an
  observation of OS resources; Rust move semantics do not prevent deliberate
  `mem::forget`; release builds have no drop bomb.
- cleave currently has no transport, no peer, no async runtime, and zero
  dependencies beyond std (keep it that way unless an ADR says otherwise).
- `docs/status/READINESS.adoc` lists "capture the live consumer pairing" and
  external trials as *beta* requirements; this prompt is not that — it is the
  two-peer protocol work.

## Read first

`docs/PROOF-NEEDS.adoc` (O-6, O-7, G-1 residual) ·
`docs/standards/RANKED-OWNERSHIP-CLEAVE.adoc` (RC-9, RC-10, RC-13 and their
falsifier rows; "Clean-cleave is a two-party event") · `docs/KERNEL.adoc` ·
`src/{lib,teardown,handle,lease,audit}.rs` · `tests/{kern,boundaries}.rs` ·
`proofs/Cleave/Kernel/*` and `proofs/README.adoc` (style: prove what you can,
state what you cannot).

## Deliverables

1. **ADR `docs/decisions/0003-two-peer-teardown.adoc`**: the protocol shape —
   a minimal transport abstraction (trait with a std-only loopback TCP
   reference implementation), the message set (propose/ack/commit/wipe-claim,
   named as you like), the well-founded discharge measure across peers, and the
   liveness assumption it rests on (e.g. fair-loss links + bounded retransmit).
   State explicitly which of O-6/O-7 this closes and which residual remains.
2. **Peer implementation** `src/peer/` (feature-gated if you must keep the base
   crate dependency-free): two peers, each with a cleave `Surface`, driving
   joint completion; ack semantics with monotone sequence numbers; bounded
   retransmit with a documented bound; half-closed detection; escalation path
   where exactly one survivor claims the wipe; idempotent wipe application.
3. **Tests `tests/twopeer.rs`** (deterministic, no sleeps-as-proof — fake time
   or explicit scheduling):
   - both peers reach ⊥ with residue 0 and both audit logs show children-before-
     parent order (assert from the logs, not from intent);
   - one-sided completion: peer A completes, B never acks → A escalates and
     the final state is clean **or** ruptured, never dangling; assert which and
     that no handle remains un-consumed;
   - lost final ack → bounded retransmit then escalation; count the retransmits
     and assert the bound;
   - duplicate/reordered acks → idempotent, no double-consume;
   - peer killed (process/connection drop) mid-teardown → survivor path, with
     the wipe claimed exactly once;
   - foreign/malformed peer messages → refused without mutation (the boundary
     discipline the kernel already tests locally, now over a socket).
4. **Idris mirrors** `proofs/Cleave/Peer/{JointCompletion,Rupture}.idr` (names
   indicative): state and prove the two properties over the inductive model
   you define — joint completion reaches ⊥ for both peers under the link
   assumption; rupture resolves to exactly one wipe claim. Include a falsifier
   fixture (a wrong reordering or a double claim that fails to typecheck),
   following the existing negative-fixture style. No `believe_me`,
   `assert_total`, or `postulate`.
5. **Ledger updates**: `docs/PROOF-NEEDS.adoc` O-6/O-7 rows updated with what
   is now closed for *this* protocol, the assumptions kept explicit, and the
   remaining residual (e.g. no fairness guarantee on real networks, no OS
   resource observation); `docs/status/PROOF-STATUS.adoc` summary row;
   `docs/status/READINESS.adoc` updated.
6. **No new dependencies without an ADR**; no `unsafe`.

## Acceptance criteria

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets --features peer   # kernel + two-peer suites
cargo test --locked --doc
cargo test --locked --release --features peer
bash tests/check_proofs.sh /absolute/path/to/evidence
# demonstration: remove the escalation path → the one-sided test FAILS (dangling)
# demonstration: allow duplicate ack consumption → idempotence test FAILS
```

## Evidence to capture

Command transcripts; the two audit logs from a joint completion; the retransmit
count and bound; the survivor-claims-once evidence; the Idris fixtures'
verdicts; the assumptions paragraph, written out for the ledger.

## Non-goals / do not do

- Do not claim distributed consensus, liveness under arbitrary partition, or
  "exactly-once" over an unreliable network without the assumption stated.
- Do not turn cleave into an async-runtime project; std-only until an ADR says
  otherwise.
- Do not close O-6/O-7 wholesale in the ledger; close what the protocol proves
  and name the residual.
- Do not lift gossamer's flat-wipe history or the phantom proofs.

## Stop conditions

- If the protocol requires reasoning about OS resources (fds, mmap) to make
  residue claims, stop: the kernel's honesty note says the residue measure is
  not that, and a new claim needs its own definition.
- If a bounded-retransmit bound cannot be justified for the reference
  transport, record the assumption as a *parameter with a documented default*
  and leave the unconditional claim open.

## Definition of done

Protocol + tests + Idris mirrors + ledger updates merged; the two
demonstrations recorded; O-6/O-7 rows state exactly what is closed and what is
assumed; PR body lists commands, results, files, closed and open items.
