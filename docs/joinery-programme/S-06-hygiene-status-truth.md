<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# S-06 · Hygiene: remove template residue and false test claims

| | |
|---|---|
| **Repository** | `metadatastician/spline` |
| **Phase** | 2 — spline track (independent; can run any time) |
| **Depends on** | nothing |
| **Blocks** | nothing (but honest status docs are prerequisites for credible promotion) |
| **Owner decision** | no |
| **Size** | small (1 PR) |

## Role

You are an autonomous coding agent working in a checkout of
`metadatastician/spline`. You delete things. Git history is the undo button
(ADR 0008: "everything at the top level justifies itself or dies").

## Mission

Remove the RSR-template residue from spline and make every status document
truthful. Several files currently describe a different repository: a template
whose test scripts, benchmarks, Idris2 ABI, Zig FFI and 21 workflows do not
exist here. That is not just clutter — it is false evidence living inside a
repository whose entire value proposition is honesty about what is wired.

## Ground truth (verified 2026-09-29 — re-verify)

- `docs/status/TEST-NEEDS.adoc` claims `scripts/validate-template.sh`,
  `benches/template_bench.sh`, `src/interface/ffi/test/integration_test.zig`
  and more — **verified missing**; it describes "rsr-template-repo" and
  "21 workflows".
- `docs/QUICKSTART.adoc` instructs the reader to clone
  `hyperpolymath/rsr-template-repo` and run `just init` — wrong repository.
- `README.adoc` and `docs/status/READINESS.adoc` reference
  `.machine_readable/descriptiles/`, which **does not exist** in the tree.
- Workflow count is **9** (`alignment, codeql, label-triage, labels, oikosbot,
  pages, scorecard, secret-scanner, workflow-safety` + `actions.lock`), where
  ADR 0008 allows ≤5, each exercising a real artifact.
- Template-era doc trees: `docs/{theory,whitepapers,wikis,practice,developer,
  reports,governance,legal,onboarding}/`, plus `docs/RSR_OUTLINE.adoc`,
  `docs/STATE-VISUALIZER.adoc`, and a root `0-AI-MANIFEST.a2ml`.
- Files that *are* real and must survive: `docs/decisions/*`,
  `docs/alignment/*`, `docs/status/{BETA-ACCEPTANCE,PROOF-NEEDS,PROOF-STATUS,READINESS,ROADMAP}.adoc`,
  `tests/check_alignment.sh`, `tests/e2e.sh`, `ARCHITECTURE.md`, `README.adoc`.

## Read first

groove `docs/REPO-LAYOUT.adoc` (ADR 0008) — canonical top level, the kill-list,
the ≤5-workflow rule, community-health placement · spline's own `README.adoc`,
`ARCHITECTURE.md`, `docs/README.adoc`, `docs/status/*`.

## Deliverables

1. **Delete the false evidence**: `docs/status/TEST-NEEDS.adoc` is replaced by
   a truthful `docs/status/TEST-INVENTORY.adoc` that lists the tests that
   actually exist (script, what it runs, when) — or delete it entirely and put
   the inventory in `READINESS.adoc`. Do not keep a document whose claims are
   false.
2. **Fix `docs/QUICKSTART.adoc`** to describe *this* repository's actual use
   (read the ledger, run the alignment check with a groove checkout, run the
   acceptance validator when a capture exists), or delete it if there is
   nothing true to say beyond the README.
3. **Resolve the `.machine_readable/descriptiles/` references**: either remove
   the claim from README/READINESS (recommended — no such tree exists and ADR
   0008 bans the pattern) or, if genuinely wanted, create it with real content
   and justify it in the PR. Default: remove.
4. **Workflows to ≤5, each real.** Keep `alignment` (folds the checker);
   keep one security workflow (codeql or secret-scanner — justify the choice,
   they may fold together); keep `workflow-safety` if it exercises a real
   invariant; delete or fold `labels`/`label-triage`/`oikosbot`/`pages` unless
   one is exercising a real artifact with evidence, and say so in the PR.
   Run `gh actions-lock --no-fix` after changes and commit the lockfile.
5. **Doc trees**: keep a tree only if it carries real content for this repo.
   `docs/alignment/`, `docs/decisions/`, `docs/status/` stay. The rest is
   deleted or folded; if something carries real content (say so with a path),
   keep that file and delete the empty scaffolding around it.
6. **One root manifest at most**: decide whether `0-AI-MANIFEST.a2ml` stays.
   If it stays, it must be accurate and it is the only one (no per-directory
   manifests); if it lies, fix or delete it.
7. **CHANGELOG**: make it a real changelog (Keep a Changelog format, real
   entries with dates) or delete the generated-by-git-cliff scaffolding. A
   changelog that has never recorded a change is a claim too.
8. **PR body** lists every deletion with a one-line reason.

## Acceptance criteria

```sh
ls .github/workflows/*.yml | wc -l                 # ≤5
! grep -rn 'rsr-template-repo\|validate-template.sh\|template_bench.sh' README.adoc docs/  # no residue referencing another repo
! grep -rn 'machine_readable' README.adoc docs/    # claim removed or tree created
bash tests/check_alignment.sh /path/to/groove-checkout   # still reproduces (c)
```

## Non-goals / do not do

- Do not delete `docs/decisions/`, `docs/alignment/`, `docs/status/` (those
  carry the real record) or any test that reproduces a live criterion.
- Do not add per-directory AI manifests; do not create `.machine_readable/`.
- Do not touch the estate-level `.github` org repo from here.
- Do not "clean up" by rewriting history or force-pushing main.

## Stop conditions

- If a deletion would remove the only record of a decision or a criterion
  (search before deleting), keep the record and delete only the scaffolding.
- If a workflow you want to delete is the only thing enforcing a stated
  invariant, either keep it or state the invariant as unenforced with a
  matching ledger edit — do not leave a silent gap.

## Definition of done

False test claims gone; QUICKSTART truthful or gone; `.machine_readable`
claims resolved; ≤5 real workflows with the lockfile regenerated; doc trees
curated; CHANGELOG real or gone; PR body enumerates deletions with reasons and
lists any invariant that lost a gate (with the ledger edit that records it).
