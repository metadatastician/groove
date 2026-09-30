<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# G-05 · Spec/registry coherence, A2ML decision, registration dossier, GRV6

| | |
|---|---|
| **Repository** | `metadatastician/groove` (one Zig change in `reference/ipv6t/`) |
| **Phase** | 2 — groove track |
| **Depends on** | nothing |
| **Blocks** | J-04 |
| **Owner decision** | **yes** — the A2ML disposition and whether to pursue IANA/media-type registration |
| **Size** | medium (1–3 PRs) |

## Role

You are an autonomous coding agent working in a checkout of
`metadatastician/groove`. You reconcile what the spec promises with what the
code, registry and changelog actually say, then make the external-facing
artifacts real.

## Mission

Close the coherence gaps between spec, registry, encodings and reference
implementation, and produce the registration dossier for the media types and
well-known URI — as a prepared submission, not a submitted one. Decide A2ML's
fate honestly: a real optional encoding with a parser and tests, or a
serve-only convenience with the spec text narrowed to say exactly that.

## Ground truth (verified 2026-09-29 — re-verify)

- Version line: `cli/Cargo.toml` and `provider/Cargo.toml` are `0.3.0`;
  `README.adoc` says 0.3.0; `spec/SPEC.adoc` header says **`v0.3.0-draft`**;
  `CHANGELOG.md`'s newest entry is `0.2.0 — 2026-07-02 (unreleased)` — there
  is **no 0.3.0 changelog entry**, and CI checks version coherence across
  Cargo.toml / SPEC.adoc / README only.
- Media types: `application/groove+json` is REQUIRED;
  `application/groove+a2ml` is OPTIONAL and "serve-only until an A2ML parser
  exists" (ADR 0002, CHANGELOG). `spec/SPEC.adoc` §2.1.3 defines the A2ML
  encoding; §10 has IANA considerations (media type §10.1, well-known URI
  §10.2) with no dossier.
- `spec/CONFORMANCE.adoc` `CONF-L1-04` requires: `Accept:
  application/groove+a2ml` returns the A2ML rendering "if the provider
  offers it" — so a provider that never offers it passes trivially; say so or
  change it deliberately.
- Registry SSOT (ADR 0006) is `registry/groove-registry.json`: probe band
  `[[6460,6500]]`, services `groove-ref:6465` (reference) and
  `gossamer:6470` (external), plus capability/protocol types; derived tables:
  `cli/src/registry.rs` (compile-time embed) and the extension's
  `groove-targets.gen.js` (checked by `gen-targets.mjs --check`).
- `reference/ipv6t/` is Zig and pinned to **0.14.1** because the code uses
  `std.net`/`std.time` APIs removed by Zig 0.16's `std.Io` rework; CI's zig
  job comments say migration is a tracked follow-up — but no issue, ADR or
  ledger row currently tracks it.
- DOC-1-style drift is documented in `docs/GROOVE-CLI-DESIGN.adoc`: three
  conflicting port tables (historical), `groove_version` string formats,
  capabilities map-vs-array — confirm each is now impossible by construction
  (registry + validation) or record the residual.

## Read first

`spec/SPEC.adoc` §2.1 (all subsections), §10 · `spec/CONFORMANCE.adoc`
(esp. CONF-L1-04) · `docs/decisions/0002-manifest-encodings.adoc`,
`0006-port-registry.adoc`, `0010-manifest-signing.adoc`,
`0011-version-line-0-3-0.adoc` · `registry/groove-registry.json` ·
`cli/src/registry.rs`, `cli/src/validate.rs`, `cli/src/probe.rs` ·
`provider/src/lib.rs` (manifest rendering + content negotiation) ·
`CHANGELOG.md` · `reference/ipv6t/` + CI zig job comment.

## Deliverables

1. **Version/changelog coherence**: add the missing `0.3.0` entry to
   `CHANGELOG.md` (lease modes §4.6, optional manifest signature §2.1.5,
   ADR 0011) with the real date of the change; decide `v0.3.0-draft` vs
   `v0.3.0` in `SPEC.adoc` and make it consistent with README/Cargo; extend the
   `spec-consistency` CI job to check `CHANGELOG.md` mentions the current
   version (a one-line addition, not a new workflow).
2. **A2ML disposition (owner decision required)** — one of:
   - *Implement*: a real parser with tests, wired into probe/validate/provider
     so `CONF-L1-04`'s conditional path is exercisable and served; or
   - *Narrow*: spec §2.1.3 and README/CHANGELOG text state A2ML is
     **serve-only, never parsed**, and `CONF-L1-04` is annotated accordingly
     with an owner-approved decision record. Either way the spec text and the
     code must agree; no silent "optional".
3. **Registry coherence check**: a small test/check (extend
   `cli/tests/registry_consistency.rs` or the existing JS checks) that every
   port literal and capability/protocol type in `spec/*.adoc` matches the
   registry SSOT — the drift class `docs/GROOVE-CLI-DESIGN.adoc` documented
   (three port tables, string-format divergence) must be mechanically
   impossible or explicitly listed as exempt.
4. **Registration dossier** `docs/registration/` (owner decides whether to
   submit):
   - media type registration template for `application/groove+json`
     (and `+a2ml` if Path A2ML keeps it), with a contact, change controller,
     specification reference and security considerations drawn from §9;
   - well-known URI registration (`/.well-known/groove`) per RFC 8615;
   - a checklist of what the owner must do (send mail / open the registry
     request) and what this repo pins so the dossier stays in sync.
   Do not claim any registration has happened.
5. **GRV6**: either migrate `reference/ipv6t/` to Zig 0.16's `std.Io` (keeping
   the 10 tests green and updating CI's zig pin), or record the migration as
   tracked work: an ADR or ledger row naming the blocker and the target
   version, and a CI comment replaced by a real tracking reference. Do not
   leave it as a code comment.
6. **Spec status line**: `spec/SPEC.adoc` §Status updated to reflect reality
   (executable L1/L2 + lease extension; L3/L4 per G-02; annex non-normative).

## Acceptance criteria

```sh
grep -n '0.3.0' CHANGELOG.md | head -3            # entry exists
grep -n 'v0.3.0' spec/SPEC.adoc | head -3          # matches README/Cargo
cargo test --locked -p groove-cli registry_consistency
cargo test --locked --workspace
bun clients/browser-extension/scripts/gen-targets.mjs --check
# A2ML path: either a parser test exists, or the spec text says serve-only and a
# negative test proves the provider never parses it.
cd reference/ipv6t && zig build test               # 0.14.1 or the new pin
```

## Evidence to capture

Command transcripts; the dossier files; the A2ML decision record; the registry
coherence check output; the version-coherence CI job run; Zig migration result
or the tracking ADR.

## Non-goals / do not do

- Do not hand-edit derived tables; regenerate them from the registry.
- Do not submit IANA registrations (owner action).
- Do not bump the version line without an ADR (ADR 0003/0011 govern this).
- Do not widen `Accept` handling in a way that changes Level-1 semantics; L1
  stays bare-by-default.

## Stop conditions

- **Owner decision** required on A2ML disposition and registration pursuit;
  prepare both and stop for the ruling rather than choosing silently.
- If the Zig migration would break the GRV6 frame tests for reasons outside
  this repo's control, stop at the tracking ADR.

## Definition of done

Version/changelog/spec coherent; A2ML decision recorded and implemented;
registry coherence mechanically checked; dossier prepared with the owner
checklist; GRV6 migrated or tracked by a real reference; PR body lists
commands, results and the owner decisions taken.
