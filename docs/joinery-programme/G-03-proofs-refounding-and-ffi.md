<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# G-03 · Proofs re-founding (Groove.* rename, SPEC data model) + Level 3 Idris2/Zig FFI

| | |
|---|---|
| **Repository** | `metadatastician/groove` |
| **Phase** | 2 — groove track |
| **Depends on** | G-02 (Level 3 wiring), J-01 (cross-repo interface points, for the agreement check) |
| **Blocks** | J-04 |
| **Owner decision** | **yes** — the `Gossamer.ABI.*` → `Groove.*` rename breaks name-based comparability by design; the owner rules on the trade |
| **Size** | large (2–4 PRs) |

## Role

You are an autonomous coding agent working in a checkout of
`metadatastician/groove`. Proofs are the estate's most abused artifact class:
phantom modules counted as progress, per-file "checks" that pass on
module-load errors. You are the opposite of that.

## Mission

Finish the PROOFS-1 re-founding: rename the relocated modules to
`Groove.*` **together with** the cross-repo agreement-check design that makes
the rename safe, re-state the proofs against the current manifest data model
instead of the vendored gossamer ABI, and — only then — make the Level 3
proof/FFI line items executable (or descope them with an owner-level ADR).

## Ground truth (verified 2026-09-29 — re-verify)

- `proofs/` is wired and gated: `groove-proofs.ipkg` typechecks 12 modules in
  CI (`proofs.yml`), `vendor-integrity` diffs the two vendored
  `Gossamer.ABI.{Types,Groove}` files against a pinned upstream commit
  (`proofs/VENDOR.json`), and `tests/check_proofs.sh <evidence-dir>`
  reproduces locally. The three relocated proof modules keep their
  `Gossamer.ABI.*` names **deliberately**: "the cross-repo proof agreement
  check compares theorem surfaces by module name; a rename would break
  comparability."
- PROOFS-1's remaining vision, in the repo's own words: (1) rename to
  `Groove.*` only together with the agreement-check design; (2) re-state the
  proofs against the v0.2 manifest data model in `SPEC.adoc` — "a semantic
  re-founding, not a port".
- Honest limits already recorded: `GrooveTermination.idr` proves ≤4-step
  completeness with subset evidence, not terminal-outcome uniqueness;
  `CapabilityAuthenticity.idr` proves table coverage/membership/attenuation,
  not that a handler address implements the capability; the finite handshake
  model does not prove eventual delivery under loss; **no proof-to-Rust or
  proof-to-Zig refinement is claimed**.
- Gate rules measured here: `idris2 0.7.0 --check file.idr` exits 0 on
  module-load errors — use ipkg `--typecheck`; never `continue-on-error`;
  CI pins Idris2 v0.8.0 (commit `15a3e4e70843f7a34100f6470c04b791330788df`).
- Level 3 target text: Idris2 dependent-type capability interfaces; linear
  handles; C headers generated from the Idris2 ABI; a Zig FFI layer;
  no `believe_me`/`assert_total`.

## Read first

`proofs/README.adoc` · `groove-proofs.ipkg` · `.github/workflows/proofs.yml` ·
`tests/check_proofs.sh` · `proofs/VENDOR.json` · `spec/SPEC.adoc` §2.1
(manifest data model), §3.3 (aspirational type-level guarantees), §4 ·
`spec/CONFORMANCE.adoc` Level 3 · G-02's disposition table.

## Deliverables

1. **Agreement-check design first** — a document
   (`docs/decisions/0012-proof-module-naming-and-agreement.adoc`) that
   specifies: what the cross-repo agreement check compares (theorem surfaces by
   module name), where it lives, what breaks under a rename, and how the rename
   preserves comparability (e.g. an explicit mapping table from old to new
   module names carried in the check, checked into the repo). **Owner decision
   gate:** present the design and the trade; do not perform the rename until
   the owner accepts it.
2. **Rename with the check** (after acceptance): modules renamed to `Groove.*`,
   the mapping table added to the agreement check, `groove-proofs.ipkg`
   updated, `vendor-integrity` still green (vendored deps unchanged!), and a
   CI job that fails if the mapping and the module names ever disagree.
3. **Semantic re-founding** — re-state the three proofs against the current
   manifest data model (§2.1) rather than the vendored gossamer ABI: the new
   statements must be about *this* spec's types. Keep a written mapping from
   old statement to new statement, and update `proofs/README.adoc`'s limits
   paragraph with what the new statements do and do not establish (the old
   honesty text is the model to beat, not to delete).
4. **Trust-base gate**: keep/extend the CI check that greps for `believe_me`,
   `assert_total`, `postulate` (zero in `proofs/`); a new module that
   introduces one fails the build.
5. **Level 3 FFI line items** — decide and execute one of:
   - **Implement**: an Idris2 ABI surface for the capability interfaces with
     generated C headers and a Zig FFI layer exercising it against the
     reference provider's actual wire behaviour; C/Zig conformance tests
     (`reference/ipv6t` is the existing Zig home; a new Zig crate needs a
     layout decision under ADR 0008); no `believe_me`/`assert_total` anywhere;
     proof↔runtime correspondence stated honestly (tests exercise the ABI;
     they do not prove refinement).
   - **Descope**: an accepted ADR removing the FFI items from the Level 3
     target list with the reason, and `spec/CONFORMANCE.adoc` edited to match.
   Either way, no line item remains "target" without a decision.
6. **Ledger and spec honesty**: `proofs/README.adoc`, `spec/CONFORMANCE.adoc`,
   `README.adoc` status table, `docs/dev-notes/known-issues.md` all updated in
   the same PRs; any PROOFS-1 residual that remains open is named.

## Acceptance criteria

```sh
bash tests/check_proofs.sh /absolute/path/to/evidence   # 12 modules typecheck
grep -rn 'believe_me\|assert_total\|postulate' proofs/  # zero (gate, not wish)
# the agreement check refuses a rename without a mapping entry:
#   temporarily rename a module → CI check FAILS, restore
# no per-file --check loops anywhere:
! grep -rn 'idris2 --check' .github/ tests/
```

## Evidence to capture

The check_groove.sh output; CI run links/ids for the pinned Idris2 build; the
old→new statement mapping; the negative-control demonstration (rename without
mapping fails); the FFI decision with its evidence or ADR.

## Non-goals / do not do

- Do not lift gossamer proof code; rebuild against actual types (the phantom
  surface rule).
- Do not add `continue-on-error` or a per-file `--check` loop back.
- Do not claim refinement (proof-to-Rust/Zig) at any point; if you want it,
  that is a separate research prompt with its own ADR.
- Do not rename the modules before the agreement check exists.

## Stop conditions

- **Owner decision** on the rename trade and on Level 3 FFI scope; stop
  without them.
- If the vendored upstream files must change for the re-founding, stop: the
  vendored copies are pinned byte-identical and the change belongs upstream
  first.

## Definition of done

Agreement check designed (and, if accepted, implemented with the rename);
proofs re-founded against the manifest model or the residual restated; trust
gate enforced in CI; Level 3 disposition recorded per line item; every ledger
truthful; PR body lists commands, results, and open items.
