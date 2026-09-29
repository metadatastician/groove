<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# J-01 · Joinery interface control document + pin registry

| | |
|---|---|
| **Repository** | `metadatastician/groove` (the ICD is hosted here; cleave and spline get pointer changes) |
| **Phase** | 0 — foundations |
| **Depends on** | nothing |
| **Blocks** | J-02, J-04, C-01, S-05 |
| **Owner decision** | no |
| **Size** | medium (1–2 PRs) |

## Role

You are an autonomous coding agent working in a checkout of
`metadatastician/groove`. You write precise documents and small, testable
scripts. You never claim a thing is verified without a command, an artifact
path and a date.

## Mission

Create the single inter-repository contract that makes `cleave` (surface),
`groove` (protocol) and `spline` (data-form) composable **by construction**:
a document that numbers every interface point between the three layers, and a
machine-readable pin registry plus checker that keep the three repositories'
claims about each other honest. This is the artifact every other prompt's
acceptance criteria reference; without it, integration is improvisation.

## Ground truth (verified 2026-09-29 — re-verify before you start)

- The layer canon is settled: `groove/docs/decisions/0009-the-joinery-naming-canon.adoc`
  (cleave = surface, groove = protocol, spline = data-form; consumers are not
  layers).
- groove's `spec/LAYERING.adoc` is explicitly **aspirational** and describes a
  future groove-on-cleave layering; groove ADR 0007 keeps cleave separate and
  says cleave work blocks no groove milestone.
- groove `spec/SPEC.adoc` §4.6 already makes the lease vocabulary shared:
  soft expires to a zero-residue wipe, hard refreshes and degrades after three
  whole missed TTL windows — the wire-visible shadow of cleave RC-6.
- spline ADR 0002 (plane multiplexing), ADR 0003 (consumers own schemas),
  ADR 0004 (opaque token, linear at each endpoint, posture never in bytes)
  define spline's side; `groove/registry/bebop-voice-signal-alignment.json`
  and `groove/scripts/check-bebop-alignment.mjs` are the existing SSOT-plus-
  checker pattern to imitate.
- There is currently **no** cross-repo pin file, **no** ICD, and the only
  cross-repo check is spline's `tests/check_alignment.sh` shelling into a
  groove checkout.
- groove's registry (`registry/groove-registry.json`) is the single source of
  truth for ports/services/types (ADR 0006) and is CI-drift-checked. Do not
  create a competing table.

## Read first

`docs/decisions/0009-the-joinery-naming-canon.adoc` ·
`docs/decisions/0007-cleave-separate.adoc` ·
`spec/LAYERING.adoc` · `spec/SPEC.adoc` §2.1/§4/§5/§6 · `spec/CONFORMANCE.adoc` ·
`registry/groove-registry.json` · `scripts/check-bebop-alignment.mjs` ·
`docs/REPO-LAYOUT.adoc` (ADR 0008) · in the cleave repo:
`docs/PROOF-NEEDS.adoc` (O-1..O-9, G-1..G-7) and
`docs/standards/RANKED-OWNERSHIP-CLEAVE.adoc` (RC-1..13, TS-1..7); in the
spline repo: `docs/decisions/0002..0005`, `docs/alignment/voice-signal-plane.adoc`,
`docs/status/BETA-ACCEPTANCE.adoc`.

## Deliverables

1. **`spec/JOINERY-ICD.adoc`** — the interface control document. Status line
   must read: *binding between the three repositories for the interfaces it
   numbers; does not alter normative Groove v0.x text.* Contents:
   - **Layer table**: layer, repo, owns, explicitly does not own, current
     version, current honest status (quote the source ledger).
   - **Numbered interface points `IP-1..IP-n`.** At minimum:
     IP-1 discovery/manifest; IP-2 session lifecycle (groove §4 state machine
     ↔ cleave Surface/handle); IP-3 leases (SPEC §4.6 ↔ cleave RC-6);
     IP-4 handles/tokens (groove authority handle ↔ wire-opaque string ↔
     linear endpoint handle); IP-5 planes (which typed planes a session
     carries ↔ spline multiplexing); IP-6 noninterference (TS-7: what must
     never be encoded); IP-7 attestation/provenance (SPEC §5 hash chain ↔
     cleave audit log); IP-8 version negotiation and compatibility.
     For **each** IP: producer, consumer, direction of authority, normative
     source, error/status mapping (e.g. groove `409`/`410` ↔ cleave
     `RankError`/`LeaseError`), what evidence closes it, and which check (CI
     job or J-02 scenario) tests it.
   - **Prohibited crossings**: rank/ownership witnesses, posture triples,
     lease-state or lifecycle bookkeeping into byte encodings; a new
     serialiser; normative groove text depending on cleave/spline while the
     layering is an annex.
   - **Change procedure**: who may change an IP, what version bump it needs,
     what must be re-run, how a breaking change is announced.
2. **`registry/joinery-pins.json`** — machine-readable pins. Per entry:
   `repo`, `ref` (tag or commit), `path`, `sha256` (of the pinned file(s)),
   `verified` (`{date, command, by}`), `kind` (`spec` | `checker` |
   `fixtures` | `implementation`), `consumed_by` (which check/job). Cover at
   least: the spline alignment record + frames, the cleave invariants file,
   the cleave kernel contract, the groove conformance doc and checker.
   Machine-readable single source of truth rules apply: this file is the SSOT,
   and anything derived from it (the ICD tables, a rendered status page) is
   generated or validated, never hand-copied.
3. **`scripts/check-joinery-pins.mjs`** (+ a self-test file following the
   `check-bebop-alignment.test.mjs` pattern) — validates the pins file:
   schema shape; every local path exists; every digest matches when the
   sibling checkout is present; `verified.date` present and not in the future;
   no `consumed_by` naming a job that does not exist in `.github/workflows/`.
   Offline by default; a `--remote` mode resolves refs via `git ls-remote`
   and is used only in a network job. It must fail loudly on an empty or
   unparsable pins file (no vacuous pass).
4. **CI wiring**: add the checker to the existing `spec-consistency` job
   (or a new ≤1 job); do not exceed the repo's workflow budget (ADR 0008 says
   ≤5; if you must add one, delete or fold a governance-residue workflow in
   the same PR and say so).
5. **Pointer PRs (same session, separate PRs, keep them tiny)**:
   - cleave: an ADR (`docs/decisions/0002-joinery-interface-points.adoc`)
     recording that IP-1..n are binding and that the LAYERING annex is now
     implemented piecewise (link the ICD); update `docs/PROOF-NEEDS.adoc`'s
     "See also" only if you can keep it truthful.
   - spline: `docs/decisions/README.adoc` or `docs/architecture/README` entry
     pointing at the ICD; note in `tests/check_alignment.sh`'s comment that the
     pins registry names the groove ref it expects.
   - groove: `spec/LAYERING.adoc` gains a header line: "Interface points IP-1..n
     are numbered in `JOINERY-ICD.adoc`; this annex remains aspirational until
     each IP's evidence column is filled."

## Acceptance criteria (run these; paste results in the PR body)

```sh
cd groove
bun scripts/check-joinery-pins.mjs                 # exit 0, prints per-entry status
bun scripts/check-joinery-pins.test.mjs            # negative controls pass
grep -c '^| IP-' spec/JOINERY-ICD.adoc             # ≥8 numbered interface points
python3 -c "import json,sys;d=json.load(open('registry/joinery-pins.json'));assert d['pins'],'empty'"
bash -c 'set -euo pipefail; grep -q "TBD\|TODO" spec/JOINERY-ICD.adoc && exit 1 || exit 0'  # no TBDs
```

Plus: every IP row names a test or scenario that does not yet exist → it is
listed in the ICD's *open evidence* column **and** in
`docs/status/` or `READINESS.md` if it changes a beta claim.

## Evidence to capture

`registry/joinery-pins.json` entries with `verified.command` showing the
digest command actually run; the checker's output; a short
`docs/joinery-programme-evidence/J-01.md` (or a PR-body block) recording the
commands, results and date.

## Non-goals / do not do

- Do **not** promote `spec/LAYERING.adoc` to normative text.
- Do **not** renumber or delete any `CONF-*` ID, or add one without a test.
- Do **not** rename layers or consumers (ADR 0009 is settled).
- Do **not** put a second copy of port/service data anywhere; the registry
  stays the SSOT.
- Do **not** create a new repo or a new umbrella project. The ICD lives in
  groove; ADR 0008 governs layout.
- Do **not** hand-edit generated tables (e.g. `clients/browser-extension/background/groove-targets.gen.js`).

## Stop conditions

- If cleave or spline cannot be checked out at a pinned ref from this
  environment, pin by digest of the files you actually read and say so in
  `verified.command` — do not invent a ref.
- If a proposed IP would require changing normative groove text, stop and
  report; that is a spec decision, not an ICD decision.

## Definition of done

ICD merged with ≥8 numbered IPs and no TBDs; pins file + checker + self-test
green in CI; three pointer PRs merged or opened with a link; `index.json`
unaffected; PR body lists commands, results, files, and explicitly names which
IP evidence rows are still **open**. The handoff to J-02: the scenario list it
must implement is exactly the ICD's evidence column.
