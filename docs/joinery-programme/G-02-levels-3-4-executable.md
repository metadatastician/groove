<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# G-02 · Make Levels 3 and 4 executable — or descope them by ADR

| | |
|---|---|
| **Repository** | `metadatastician/groove` |
| **Phase** | 2 — groove track |
| **Depends on** | nothing (proof work inside Level 3 pairs with G-03) |
| **Blocks** | G-03, J-04 |
| **Owner decision** | no (an *owner* decision is required only if you descope a level rather than implement it — then the ADR is the record) |
| **Size** | large (2–4 PRs) |

## Role

You are an autonomous coding agent working in a checkout of
`metadatastician/groove`. You implement protocols and their executable
conformance, and you are ruthless about the difference between "target" and
"test".

## Mission

Take the two aspirational conformance levels and give each line item one of two
honest outcomes: an executable `CONF-L3-*`/`CONF-L4-*` requirement with a
matching test (CI-enforced), or an accepted ADR that removes it from the spec's
target text and records why. No line item may remain "target" without an owner
decision behind it.

## Ground truth (verified 2026-09-29 — re-verify)

- `spec/CONFORMANCE.adoc` states Levels 3/4 "CANNOT be normative while no
  implementation — including the reference one — meets them; no conformance
  IDs are assigned until an executable test exists for each line item."
- Level 3 targets: Idris2 dependent-type capability interfaces (blocked on
  PROOFS-1 → G-03); linear connection handles (today approximated at runtime
  by `CONF-L2-04`); C headers generated from an Idris2 ABI; Zig FFI;
  no `believe_me`/`assert_total`.
- Level 4 targets: version negotiation with SemVer constraints (§3.2);
  capability-based security requirements (§9.2); distributed discovery
  (mDNS/DNS-SD or BEAM clustering, §2.3); transitive mesh composition with
  type-safety proofs (§6); VeriSimDB provenance (§5.3, "when available").
- Existing executable surface: `CONF-L1-01..04`, `CONF-L2-01..07`,
  `CONF-L2-08..12` (optional lease extension). The CLI already has `mesh`
  (`cli/src/probe.rs::mesh`) but mesh semantics have no conformance IDs; §3.2
  describes SemVer constraints with no test; §9.2 capability restrictions are
  prose; §2.3 distributed discovery is prose.
- CI enforces ID↔test pairing: every `CONF-*` ID in `spec/CONFORMANCE.adoc`
  needs `fn conf_*` in `provider/tests/` (see `.github/workflows/ci.yml`
  `spec-consistency`).

## Read first

`spec/CONFORMANCE.adoc` (§Level 3, §Level 4) · `spec/SPEC.adoc` §2.3, §3.2,
§5.3, §6.1–6.3, §9.2 · `spec/MODULARITY.adoc`, `spec/INNERVATION-SIGNALS.adoc`
(if they constrain the above) · `provider/src/lib.rs`,
`provider/tests/conformance.rs` · `cli/src/probe.rs` (mesh) ·
`docs/GROOVE-CLI-DESIGN.adoc` · `proofs/README.adoc` (the PROOFS-1 boundary
that gates part of Level 3).

## Deliverables

Work item by work item; each lands with a test or an ADR — no orphan prose.

1. **Version negotiation (§3.2) → executable.** Define the wire shape for
   constrained compatibility (e.g. requested/offered constraint objects and the
   negotiation result), implement in the reference provider, test positive and
   negative (unsatisfiable constraint → documented refusal, not a silent
   downgrade). New IDs `CONF-L4-01..` (or L3-01 — choose by the level whose
   text you are making executable and keep the doc's numbering intent).
2. **Capability-based security (§9.2) → executable.** At minimum: a session's
   admitted capabilities constrain operations (pairs with G-01); a request
   requiring a capability the session lacks is refused with a documented
   status; the refusal emits an attestation event. Tests + IDs.
3. **Distributed discovery (§2.3) → implement or descope.** mDNS/DNS-SD is a
   real dependency decision; BEAM clustering is out of this repository's
   stack. Default honest path: implement a minimal, dependency-light
   discovery mechanism if the spec's own text can be satisfied locally
   (e.g. explicit seed/host list with the existing probe mechanism,
   documented as a non-broadcast profile of §2.3), otherwise ADR-descope with
   the reason and remove the claim from Level 4 target text.
4. **Transitive mesh (§6) → executable.** Implement mesh composition over the
   existing mesh view: given A—B and B—C, the mesh view reports the transitive
   relation with the type-safety check §6.2 requires (no composed connection
   may offer a capability pair that admissibility refuses); cycle handling is
   defined and tested. Positive + negative tests, IDs, and CLI output that
   shows the refused composition reason.
5. **VeriSimDB provenance (§5.3) → keep as conditional or descope.** If no
   VeriSimDB endpoint exists in this environment, do not fake it: state the
   condition ("when available via groove"), keep it out of the level's MUST
   list, and add a contract note for the future integration.
6. **Level 3 items owned by G-03** (Idris2 capability interfaces, generated C
   headers, Zig FFI): this prompt must not half-land them. Either G-03 runs
   first and this prompt wires the IDs/tests, or Level 3 stays labelled
   *blocked on G-03* in the spec with the blocker named.
7. **Spec text and docs**: `spec/CONFORMANCE.adoc` updated so no line item
   remains both "target" and unassigned; each ID maps to a test; `README.adoc`
   status table updated; `docs/dev-notes/known-issues.md` entry if something
   is descoped.
8. **CLI**: `groove mesh --json` output shape documented and tested against the
   new semantics (the CLI already emits a mesh JSON graph; bring it into
   conformance rather than inventing a second view).

## Acceptance criteria

```sh
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo test --locked --workspace --release
# every new ID has a test:
python3 - <<'PY'
import re,pathlib
ids=set(re.findall(r'CONF-L[34]-\d\d', pathlib.Path('spec/CONFORMANCE.adoc').read_text()))
src=pathlib.Path('provider/tests/conformance.rs').read_text()
missing=[i for i in ids if f"fn {i.lower().replace('-','_')}" not in src]
print('missing:',missing); assert not missing
PY
bun tests/joinery/gate.sh --scenario mesh    # J-02 scenario (if J-02 landed)
```

## Evidence to capture

Command transcripts; per-item table (implemented+ID / descoped+ADR / blocked
+blocker); the negative tests' observed statuses; updated spec sections.

## Non-goals / do not do

- Do not add a level to make an item fit; use the level whose text you are
  making executable.
- Do not weaken or renumber `CONF-L1/L2`; append-only additions.
- Do not claim Level 3 while the Idris2 ABI leg is unbuilt.
- Do not add heavyweight dependencies for discovery without an explicit
  dependency decision recorded in an ADR.

## Stop conditions

- If an item requires a repository you do not own (gossamer for BEAM-side
  clustering; a VeriSimDB deployment), stop at the contract/ADR, do not
  simulate it and call it conformance.
- If an implementation choice would change normative behaviour beyond the
  level text, stop and raise it as a spec decision.

## Definition of done

Every Level 3/4 line item is implemented-with-test, descoped-with-ADR, or
blocked-with-named-blocker; `spec-consistency` CI green with the new IDs;
README/status updated; PR body lists the per-item disposition and evidence.
