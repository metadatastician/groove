<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# S-02 · Promotion review — record all four criteria, unblock `src/`

| | |
|---|---|
| **Repository** | `metadatastician/spline` |
| **Phase** | 2 — spline track |
| **Depends on** | J-03 (validated capture) |
| **Blocks** | S-03, S-04 |
| **Owner decision** | **yes** — the promotion decision itself is the owner's |
| **Size** | small (one PR plus a review) |

## Role

You are an autonomous coding agent working in a checkout of
`metadatastician/spline`. This is a **review and recording** task, not an
engineering task. Your value is refusing to round up.

## Mission

Conduct the promotion review ADR 0005 demands: assemble the evidence for
criteria (a)–(d), mark each as met/unmet with a path to its evidence, and —
only if all four are met — record spline's exit from seed status, which lifts
the `src/` prohibition. Where any criterion's evidence is historical, say so
and require a fresh reproduction or explicitly accept the historical record
with its dates.

## Ground truth (verified 2026-09-29 — re-verify)

- ADR 0005 requires **all four** before promotion: (a) burble control-plane
  traffic over its Bebop codecs end-to-end in live transport; (b) a second
  consumer parses the same plane; (c) the alignment mapping has a conformance
  test in groove CI; (d) the typed-token boundary is exercised by the
  gossamer↔burble pairing demo.
- The alignment ledger records (a)/(b)/(c) as **shipped then**: burble#172/#178/
  #179/#180, gossamer#145, groove#30, with the honest caveats (burble#189 codec
  breakage window; SDP normalised to JSON on legacy WebSocket delivery unless a
  client requests `wire_format: bebop`).
- Criterion (c) is the only criterion reproducible locally today:
  `bash tests/check_alignment.sh /absolute/path/to/groove-checkout`.
- Criterion (d) is produced by J-03 and validated by `tests/acceptance/`.
- ADR 0005's stability rule (versioned mapping; two independent
  implementations) is **additional** and is not part of promotion; S-03 owns
  it.

## Read first

`docs/decisions/0005-promotion-criteria.adoc` ·
`docs/status/BETA-ACCEPTANCE.adoc` · `docs/alignment/voice-signal-plane.adoc` ·
`docs/status/{READINESS,ROADMAP,PROOF-STATUS}.adoc` ·
`tests/check_alignment.sh` · S-01's validator output for the J-03 capture ·
groove's `scripts/check-bebop-alignment.mjs`.

## Deliverables

1. **Promotion review record** `docs/status/PROMOTION-REVIEW-<date>.adoc`
   (or an ADR-0005 status update if the owner prefers a single document):
   - per criterion: verdict (met/unmet), evidence path, **how it was
     reproduced or why it is accepted historically**, date, and reviewer;
   - the explicit statement that (d) evidence is the J-03 capture and the
     validator result (paste the summary table);
   - the stability debt that remains (versioned mapping, second independent
     implementation) with S-03 as owner;
   - what promotion does and does not license: `src/` may now exist; nothing
     is claimed about production readiness, external users, or media-plane
     behaviour.
2. **If and only if all four are met**: a minimal set of consistency edits —
   `README.adoc` status line (seed → promoted, with the date and the review
   path), `ARCHITECTURE.md`, `docs/status/READINESS.adoc`,
   `docs/decisions/0005-promotion-criteria.adoc` status note, and
   `CHANGELOG.md` entry. If **any** criterion is unmet, report which and stop;
   do not edit the status line.
3. **Estate coherence**: check the spline row in the estate's own records
   (`cadastra` triage and any mirror listing) is not contradicted; do not
   edit other repos — list the follow-ups instead.
4. **Gate check**: `bash tests/check_alignment.sh` and
   `bun tests/acceptance/accept.mjs --capture …` both run in the repo's CI on
   push (fold into the existing alignment workflow; ≤5 workflows).

## Acceptance criteria

```sh
bash tests/check_alignment.sh /path/to/groove-checkout   # reproduces (c)
bun tests/acceptance/accept.mjs --capture <J-03-capture> # zero FAIL
git grep -n 'seed' README.adoc | head                   # only truthful mentions remain
```

## Non-goals / do not do

- Do not create `src/` in this prompt. Promotion *permits* it; S-04 builds it.
- Do not accept (a)–(c) as current without saying they are historical records.
- Do not renumber or delete ADR 0005's criteria; add a status line, not a
  rewrite.
- Do not describe promotion as "stability"; the stability rule is separate.

## Stop conditions

- The **owner** must sign the promotion verdict; if the owner has not decided,
  deliver the review record with the recommendation and leave the status
  unchanged.
- If a criterion's evidence cannot be traced to a revision and a command, mark
  it unmet and stop — do not reconstruct it from memory of the ledger.

## Definition of done

Review record merged (promotion or a precise stop); every status doc in spline
consistent with the verdict; CHANGELOG truthful; CI runs both gates; PR body
lists the per-criterion table and what remains open (stability rule → S-03).
