<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# J-05 · External trials, CRG evidence, final honest acceptance

| | |
|---|---|
| **Repository** | all three (evidence hosted in groove; per-repo ledgers updated) |
| **Phase** | 5 — final |
| **Depends on** | J-04 (release train), G-06 (operations), S-05 (interop/beta evidence) |
| **Blocks** | nothing — this is the last mile |
| **Owner decision** | **yes** — the trials, the grade claim and the final release decision are the owner's |
| **Size** | large (a programme of runs, not a coding sprint) |

## Role

You are an autonomous coding agent coordinating the final acceptance round.
You cannot manufacture external evidence, and you must not pretend to: your job
is to make the trials easy to run, the feedback easy to record, and the
resulting grade claim exactly as strong as the evidence.

## Mission

Turn the CRG bar into a run programme — home-context use (dogfooding) plus six
diverse external targets with feedback addressed — then hold the final
acceptance review: reconcile every ledger in the three repositories with
reality, record the grade and release decision, and publish the honest "not
claimed" list alongside it. Where an external target cannot be reached, the
row says so and the grade stays where the evidence stands.

## Ground truth (verified 2026-09-29 — re-verify)

- CRG (`docs/governance/CRG-CRITERIA.adoc` in the cleave and spline copies):
  C = self-validated/dogfooded; B = tested on six or more diverse external
  targets; A = field-proven with external users confirming use; F = reject.
  Evidence over intuition; grades can be demoted by regression.
- All three repos repeat the same requirement in their own words: local test
  configurations are not external targets (groove READINESS; cleave READINESS;
  spline BETA-ACCEPTANCE). No trial may be claimed from a local variant.
- groove beta gate 3 is exactly this; gate 4 (deployment owner + rehearsal) is
  G-06's and must already be recorded on the status page.
- spline's beta acceptance required the (d) capture (J-03) plus the stability
  rule (S-03); both have recorded evidence if this prompt is being run.
- cleave's bar: full-surface "demonstrated" means a second wired dial point,
  soft and hard groove, staircase teardown invoked, linear handle linear,
  proofs guarding running code — plus O-6/O-7 closed over a real wire.
- The estate's own rule set: never CI colour; evidence over claims; the
  `PROOF-NEEDS` honest-ledger pattern; no invented completion percentages.

## Read first

`docs/governance/CRG-CRITERIA.adoc` (cleave) · groove `READINESS.md`,
`docs/JOINERY-STATUS.adoc` (J-04), `tests/joinery/` evidence bundle ·
cleave `docs/status/READINESS.adoc`, `docs/PROOF-NEEDS.adoc` ·
spline `docs/status/BETA-ACCEPTANCE.adoc`, `docs/status/INTEROP-MATRIX.adoc` ·
the three CHANGELOGs and tags from J-04.

## Deliverables

1. **Trial protocol** `docs/trials/PROTOCOL.adoc` (groove): what a target is
   (a distinct deployment/platform/consumer combination — not a different port
   on the same laptop), what each trial must record (environment, versions,
   pins, steps, observations, failures, duration, feedback), how to submit
   (template in `docs/trials/TEMPLATE.md`), and how feedback is triaged
   (issue → disposition: fixed / documented / declined with reason).
2. **Home-context record** `docs/trials/HOME.md`: the dogfooding run(s) on the
   estate's own machines with real consumers — evidence for C, with the same
   fields as an external trial. If home use has already happened, marshal the
   existing evidence rather than re-running it; record dates.
3. **Six diverse external targets**: a table with one row per target —
   environment, platform diversity argument (OS/arch/runtime/consumer mix),
   operator (owner-approved), date, outcome, feedback issues, dispositions.
   Fewer than six rows means the grade claim stays at C and the table records
   `targets: n/6`. Do not pad with local variants.
4. **Feedback dispositions**: every issue from trials gets a disposition and a
   link; regressions from feedback trigger a grade re-assessment (CRG rule).
5. **Ledger reconciliation** (all three repos, one PR each): every status doc,
   README, spec `Status`, PROOF-NEEDS row, INVARIANT-MATRIX cell and
   BETA-ACCEPTANCE item compared against the trials and the gate output;
   anything not supported is corrected **downward**; anything newly supported
   is corrected upward with the evidence link. The three repos must not
   contradict each other or the ICD.
6. **Final acceptance record** `docs/acceptance/FINAL-<date>.adoc` (groove):
   per layer — grade claimed with evidence and per-criterion table; per system
   — the gate run, capture, compatibility matrix; and the
   **not-claimed list** (external validation beyond the six, production load
   beyond the measured envelope, refinement, media-plane guarantees, anything
   else the ledgers list as open). Owner signature line (unsigned = not
   approved).
7. **Independent-verification invitation**: a short, concrete ask (what a
   verifier would run, what they would need, where to report) in the status
   page — the honest path from B toward A. Do not claim it has happened.
8. **Post-release ops handoff**: the runbook and rollback authority from G-06
   linked from the status page; the known-issues file with owners.

## Acceptance criteria

```sh
bash groove/tests/joinery/gate.sh --all            # zero PENDING at acceptance time
bun groove/scripts/check-joinery-pins.mjs
ls groove/docs/trials/                             # home + targets table present
python3 - <<'PY'
# assertion: every trial row has environment, operator, date, outcome, feedback
PY
# the reconciliation PRs: no status doc claims anything absent from the ledgers
```

## Evidence to capture

The trials table with links; the home-context record; feedback dispositions;
the reconciliation diffs; the final acceptance record with the not-claimed
list; the owner's decision (signed or explicitly pending).

## Non-goals / do not do

- Do not claim external trials from local configurations, CI runs or the
  estate's own machines (those are home-context).
- Do not upgrade a grade to satisfy a milestone; CRG grades are earned and can
  be lost.
- Do not delete an honest `OPEN` row to make the final record look finished.
- Do not publish beyond the estate without the owner's decision.

## Stop conditions

- **Owner decision** required at: approving each external target, the final
  grade claim, and the release/publication decision. With fewer than six
  diverse targets, stop at the table and record `n/6`.
- If a trial exposes a regression, stop the acceptance record and feed it back
  through the owning prompt instead of annotating around it.

## Definition of done

Protocol, home record, targets table (n/6 honest), feedback dispositions and
reconciled ledgers merged; final acceptance record with the not-claimed list
and the owner's signature state; independent-verification ask published on the
status page; PR bodies list commands, results, files, closed items, open
items, and the exact grade now claimed with its evidence.
