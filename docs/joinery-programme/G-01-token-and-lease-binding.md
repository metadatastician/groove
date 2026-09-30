<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# G-01 · Bind handles and leases to real capability operations

| | |
|---|---|
| **Repository** | `metadatastician/groove` |
| **Phase** | 1 — live-pairing chain |
| **Depends on** | nothing |
| **Blocks** | J-03 |
| **Owner decision** | no |
| **Size** | medium (1–2 PRs) |

## Role

You are an autonomous coding agent working in a checkout of
`metadatastician/groove`. You modify the reference provider (`provider/`), the
CLI (`cli/`) and the spec-adjacent docs, and you keep every gate you touch at
least as strong as you found it.

## Mission

Make a groove session's authority *real*: a handle must authorise specific,
declared capabilities — not merely exist — so that the typed-token boundary
required by groove's beta gate 1 and spline ADR 0005(d) is a capability
boundary, with stale/forged/unauthorised rejection and both-sides cleanup
provable from tests. Today the wire implements the lease vocabulary
wonderfully and the authorisation vocabulary barely.

## Ground truth (verified 2026-09-29 — re-verify)

- `READINESS.md` gate 1: the live Burble–Gossamer capture must include
  stale/forged-token rejection and cleanup on both sides. Its §Local consumer
  lifecycle evidence notes: "the current Groove lease is not bound to the
  voice-signaling authorization path, and Gossamer's native session connect
  declares no requested capability."
- `spec/SPEC.adoc` §4.2/§3.1 define admissibility as *offered superset of
  required*; `provider/tests/conformance.rs` `conf_l2_02` checks an
  incompatible connect returns `409` and mints no handle. There is no test
  that a handle authorises only the capabilities it was minted for.
- §4.6 is implemented and conformance-tested (`CONF-L2-08..12`): soft expiry
  with `residue: 0` attestation and `409` on refresh; hard lease refresh
  (`204`, unknown handle `404`) and degradation after three whole missed TTL
  windows; expired-handle disconnect `410`.
- Bearer tokens are random; metadata redaction and bounded sessions exist;
  audit records hash-chain (`CONF-L2-07`).
- `docs/BETA-RUNBOOK.md` states the reference provider "does not implement
  arbitrary advertised application capabilities on behalf of consumers" —
  keep that honesty: this prompt adds authorisation semantics for **declared**
  capabilities plus one reference operation, not an application server.

## Read first

`spec/SPEC.adoc` §3, §4 (incl. §4.6), §5, §9 · `spec/CONFORMANCE.adoc` ·
`provider/src/lib.rs` · `provider/tests/conformance.rs` ·
`provider/tests/boundaries.rs` · `cli/src/probe.rs`, `cli/src/validate.rs` ·
`docs/BETA-RUNBOOK.md` · `READINESS.md` · spline
`docs/decisions/0004-typed-cap-token-boundary.adoc` and
`docs/status/BETA-ACCEPTANCE.adoc`.

## Deliverables

1. **Capability-scoped sessions in the provider.**
   - Connect records the consumer's declared `consumes` (and the provider's
     offered capabilities used to admit it) as part of session state.
   - A handle is accepted by an operation only when that operation's
     capability is in the admitted set. A valid handle used for an
     unauthorised capability is refused with a distinct, documented status
     (choose once and write it down; `403` is the natural candidate — if you
     choose otherwise, record why in the PR and in the spec text).
   - Revocation/expiry is consumption: after soft expiry, lease degradation or
     disconnect, *every* operation on that handle behaves exactly as
     §4.5/§4.6 require (`410`), not merely `/disconnect`.
2. **One reference capability operation** to make the boundary observable
   end-to-end (the smallest honest one: e.g. an `attestation`-scoped
   read of the session's own provenance tail, since `groove-ref` already
   offers `attestation` in `registry/groove-registry.json`). It must be
   documented in the spec/CLI docs as a reference-provider operation, not a
   protocol requirement.
3. **Tests** — extend `provider/tests/conformance.rs` and/or `boundaries.rs`
   (append-only IDs where the spec gains requirements):
   - authorised operation succeeds; unauthorised capability on the same
     handle is refused; no mutation on refusal;
   - forged handle (bad/absent/foreign) refused without state change;
   - stale handle (expired, degraded, disconnected, double-consumed) refused
     with the exact documented status;
   - capability set recorded at connect is what is enforced (a second connect
     with a different declared set does not widen the first);
   - cleanup: after each refusal/expiry path, session count and audit residue
     are as asserted, and the attestation chain is continuous.
   If any of these becomes a new normative requirement, add `CONF-L2-13..` in
   `spec/CONFORMANCE.adoc` with the matching `fn conf_l2_13…` test in the same
   PR (CI enforces ID↔test pairing), and say so in the PR title.
4. **Consumer contract text** for the capture (`docs/dev-notes/` or the
   relevant section of `spec/SPEC.adoc` as a non-normative note): what a
   consumer must send at connect (declared capabilities), what it must do on
   each rejection status, and what it must do at teardown — the checklist
   J-03 will use for both endpoints.
5. **CLI**: `groove probe`/`check-compat` should surface the admitted
   capability set and the refusal reason in their existing human/JSON output
   shapes; no new subcommands.
6. **Ledger updates in the same PR**: `READINESS.md` gate-1 row gains the new
   evidence and stays **OPEN** (the consumer-side capture is J-03's job);
   spec wording updated where the enforced behaviour is now normative.

## Acceptance criteria

```sh
cargo build --locked --workspace
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo test --locked --workspace --release
bun test clients/integration/provider.test.mjs
# and the new boundary demonstration:
cargo test --locked -p groove-provider capability
```

Paste the output, plus a hand-driven demonstration (`curl` or the CLI) of an
authorised and an unauthorised operation on one handle, in the PR body.

## Evidence to capture

Command transcripts; the two-operation demonstration; the refusal statuses
observed; the audit chain excerpt showing continuity across a refusal and an
expiry; the updated `READINESS.md` diff.

## Non-goals / do not do

- No new transport, no new serialiser, no mesh changes (G-02 owns §6).
- Do not turn the reference provider into an application server; one
  reference operation is the ceiling.
- Do not weaken `conf_l2_*` tests or renumber IDs; additions only.
- Do not break §4.6's lease semantics or §4.3's lease-less legacy path.
- Do not implement the consumer side (gossamer is a separate repository and
  J-03 owns the pairing).

## Stop conditions

- If the smallest honest reference operation would require a wire change
  (new plane, new framing), stop: that is spline's territory (S-04) and must
  go through J-01's interface points.
- If enforcing per-capability authorisation would break a documented
  conformance requirement, stop and report rather than editing the
  requirement.

## Definition of done

Capability-scoped authorisation implemented and tested; refusal statuses
documented; consumer contract text written; `READINESS.md` updated with the
new evidence and gate 1 still explicitly OPEN pending J-03; PR body lists the
commands, results, evidence paths and the exact J-03 blocker remaining.
