<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# C-06 · Hygiene and the ADR series

| | |
|---|---|
| **Repository** | `metadatastician/cleave` |
| **Phase** | 2 — cleave track (independent) |
| **Depends on** | nothing |
| **Blocks** | nothing (but the release train wants it done) |
| **Owner decision** | no |
| **Size** | small (1 PR) |

## Role

You are an autonomous coding agent working in a checkout of
`metadatastician/cleave`. The kernel is the content; everything else should
either justify itself or go.

## Mission

Finish the de-templating the README says is still outstanding: curate the
generic estate docs, bring workflows to the ≤5 real-artifact rule, and start
cleave's own decision record series — a repository this opinionated without
ADRs for its own choices (dial points, transport, sandbox, hygiene) is
under-documented where it matters and over-documented where it does not.

## Ground truth (verified 2026-09-29 — re-verify)

- README: "Some generic estate docs still live under `docs/` (governance/,
  onboarding/, legal/, …) and can be curated separately — they were kept
  rather than deleted because they carry real content."
- Present: `docs/RSR_OUTLINE.adoc` (RSR template outline), `docs/STATE-VISUALIZER.adoc`,
  `docs/practice/*`, `docs/reports/{audit,compliance,…}`, `docs/legal/EXHIBIT-*`,
  `docs/onboarding/*`, `docs/proposals/root-cleanup.adoc`,
  `docs/governance/*` (CRG criteria, maintenance checklist, TSDM,
  software-development-approach, audit/planning trees), `docs/attribution/*`.
- Workflows: **11** (`actions.lock` excluded) — `codeql`, `dependency-audit`,
  `kernel`, `label-triage`, `labels`, `oikosbot`, `pages`, `scorecard`,
  `secret-scanner`, `structure-check`, `workflow-safety`. ADR 0008
  (groove `docs/REPO-LAYOUT.adoc`) allows ≤5, each exercising a real artifact.
- `docs/decisions/` contains only `0000-template.adoc`, `0001-adopt-rsr-standard.adoc`,
  and a README. Prompts C-01..C-04 add 0002–0005; this prompt adds the
  repository-surface decision and reconciles the series.
- The keeper per ADR 0008 is the honest ledger (`docs/PROOF-NEEDS.adoc`) —
  keep it; it is explicitly named as the one template-era artifact worth
  having.

## Read first

README (status + "real content" table) · groove `docs/REPO-LAYOUT.adoc`
(ADR 0008) · `docs/decisions/` · `.github/workflows/` ·
`docs/governance/{CRG-CRITERIA,MAINTENANCE-CHECKLIST}.adoc` ·
`docs/proposals/root-cleanup.adoc` (it may already propose the curation — if
so, execute the accepted parts and cite it).

## Deliverables

1. **Docs curation** — keep only what carries real content for this repository:
   keep `docs/{KERNEL,PROOF-NEEDS,REFINEMENT(if present),README}.adoc`,
   `docs/standards/`, `docs/architecture/`, `docs/decisions/`, `docs/status/`,
   `proofs/`, the CRG criteria (referenced by READINESS); delete or fold the
   rest (`RSR_OUTLINE`, `STATE-VISUALIZER`, `practice/`, `reports/`,
   `onboarding/`, `legal/`, `proposals/` residue, per-tree `0.x-AI-MANIFEST`
   files, `governance/audit|planning|maintenance` scaffold trees) — with a PR
   body line per deletion. If a file genuinely carries estate-level content,
   say why it stays and what references it.
2. **One root AI manifest at most**: decide whether `0-AI-MANIFEST.a2ml` stays;
   if it stays, make it accurate and keep it the only one. Per-directory
   manifests are banned by ADR 0008.
3. **Workflows to ≤5, each real**: `kernel.yml` (KERN + proofs, keep),
   one Rust hygiene workflow folding `dependency-audit` + `codeql` triggers if
   they share the finding path, one security/secret workflow, one
   structure-check workflow (updated to the post-curation layout), plus
   `workflow-safety` only if it enforces something real here (name it in a
   comment). Delete community/triage workflows (`labels`, `label-triage`,
   `oikosbot`, `pages`, `scorecard`) unless one does real work for this repo —
   ADR 0008 puts estate-wide concerns at org level. Regenerate
   `actions.lock`.
4. **ADR series hygiene** `docs/decisions/0006-repository-surface.adoc`:
   record the surface decisions (what was kept/deleted and why, workflow
   budget, manifest policy, doc-tree policy), and add a short README index in
   `docs/decisions/` listing 0001–0006 with one-line summaries and statuses.
   Reconcile numbering with the ADRs produced by C-01..C-04 (renumber before
   merge if two landed in parallel, not after).
5. **CHANGELOG**: real entries for the kernel and the adapter/wire work as it
   lands (or remove the template scaffolding); no "unreleased forever".
6. **Loose ends**: `docs/status/READINESS.adoc`'s "Verify and operate" block
   must match the actual repo (it already names the right commands — verify
   each still exists after curation); fix or delete anything that references a
   removed file.

## Acceptance criteria

```sh
ls .github/workflows/*.yml | wc -l        # ≤5
! grep -rn 'RSR_OUTLINE\|STATE-VISUALIZER' README.adoc docs/ --include='*.adoc' | grep -v decisions/ || true
bash tests/check_proofs.sh /absolute/path/to/evidence    # untouched, still green
bash tests/validate_structure.sh 2>/dev/null || true      # structure check matches the new layout
cargo test --locked --all-targets                         # code untouched by hygiene, green
```

## Non-goals / do not do

- Do not delete `docs/PROOF-NEEDS.adoc`, the invariant standard, the CRG
  criteria, or anything the README's "real content" table names.
- Do not rewrite history; deletions are commits.
- Do not add a new workflow to replace deleted ones beyond the budget.
- Do not edit other repositories' docs from here.

## Stop conditions

- If a deletion would remove the only copy of a decision, keep the record and
  delete only the scaffolding (search for inbound references first).
- If `proposals/root-cleanup.adoc` contains an accepted decision that conflicts
  with this plan, follow the accepted decision and cite it.

## Definition of done

Docs curated with per-file reasons; workflow count ≤5 with a real-artifact
justification each; ADR 0006 + index merged and numbering reconciled; CHANGELOG
truthful; READINESS block matches the repo; PR body enumerates deletions and
any invariant that lost a gate (with its ledger note).
