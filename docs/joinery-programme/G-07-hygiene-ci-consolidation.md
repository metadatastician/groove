<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# G-07 · Hygiene: ≤5 real workflows, governance decisions, CHANGELOG close-out

| | |
|---|---|
| **Repository** | `metadatastician/groove` |
| **Phase** | 2 — groove track (independent) |
| **Depends on** | nothing |
| **Blocks** | J-04 |
| **Owner decision** | no |
| **Size** | small (1–2 PRs) |

## Role

You are an autonomous coding agent working in a checkout of
`metadatastician/groove`. You prune, fold and record — without deleting
anything that carries real evidence.

## Mission

Bring groove's repository surface in line with its own layout standard
(ADR 0008): five or fewer workflows that each exercise a real artifact, one
root AI manifest at most, no template-era scaffolding — while closing the one
open hygiene item in `docs/dev-notes/known-issues.md` with a decision rather
than a shrug.

## Ground truth (verified 2026-09-29 — re-verify)

- 12 workflow files today: `ci.yml`, `proofs.yml`, `codeql.yml`,
  `dependency-audit.yml`, `label-triage.yml`, `labels.yml`, `oikosbot.yml`,
  `pages.yml`, `push-email-notify.yml`, `scorecard.yml`, `secret-scanner.yml`,
  `workflow-safety.yml` (plus `actions.lock`).
- `docs/dev-notes/known-issues.md` (opened 2026-08-04) records the missing
  `governance.yml` / `hypatia-scan.yml` wrappers, the two sibling repos with
  disagreeing pins, and the correct procedure (copy a sibling, re-pin to the
  current `standards` commit, run `gh actions-lock`, read the first run's
  findings before merging).
- ADR 0008: community-health files belong once in an org-level `.github`
  repo; per-repo keep at most a minimal `SECURITY.md`. Workflows: ≤5, each
  exercising a real artifact; delete anything that cannot fail substantively.
- The seven jobs inside `ci.yml` (rust, registry-drift, webext, zig,
  spec-consistency, and the alignment step) all exercise real artifacts —
  they are the model for what survives.
- Version-line hygiene overlaps G-05: do not duplicate the CHANGELOG work;
  coordinate if both prompts run near each other.

## Read first

`docs/REPO-LAYOUT.adoc` (ADR 0008) · `.github/workflows/*` ·
`docs/dev-notes/known-issues.md` · `CONTRIBUTING.md`, `SECURITY.md`,
`GOVERNANCE.md`, `CODE_OF_CONDUCT.md`, `MAINTAINERS` (root) ·
`hyperpolymath/standards` (for the governance/hypatia recon, when accessible).

## Deliverables

1. **Workflows ≤5, each real.** Proposed shape (adjust with reasons):
   keep `ci.yml` (fold in the registry/webext/zig/spec-consistency jobs —
   they already live there), keep `proofs.yml` (real Idris gate), fold
   `codeql` + `secret-scanner` + `dependency-audit` into one security
   workflow if they can share triggers without losing findings, keep
   `workflow-safety` only if it enforces a real invariant (state which in a
   comment), and delete or move the community/triage workflows
   (`labels`, `label-triage`, `oikosbot`, `pages`, `push-email-notify`,
   `scorecard`) unless one is doing real work here — justify each deletion in
   the PR body with "cannot fail substantively" or "moved to org level".
   Regenerate `actions.lock` with `gh actions-lock --no-fix` and commit it.
2. **Governance/hypatia decision** — close the known-issues entry by one of:
   - **Adopt**: add the two wrapper jobs (fold into the security workflow if
     that keeps the count ≤5), pinned to the *current* `standards` commit
     (check `hyperpolymath/standards` rather than trusting either sibling's
     old SHA), lockfile regenerated, first run's findings actually read and
     recorded in the PR; or
   - **Decline**: an ADR or a known-issues resolution stating why (e.g. the
     checks are covered by org-level required workflows), with the standard
     named. Either way the open item stops being open.
3. **Root surface**: decide `LICENSES/` (kept — ADR 0004), and whether any
   other root file is template residue; verify at most one root AI manifest
   exists (groove currently has none at root — state that explicitly rather
   than leaving it ambiguous).
4. **Community-health placement**: if the org `.github` repo exists and is
   accessible, note which of the root health files are duplicated there;
   per ADR 0008, keep at most `SECURITY.md`. If the org repo is not available,
   record the follow-up rather than deleting the only copy.
5. **Known-issues file**: every entry either resolved (with the PR) or
   explicitly re-dated with an owner and a reason to stay open.

## Acceptance criteria

```sh
ls .github/workflows/*.yml | wc -l        # ≤5
gh actions-lock --no-fix                  # clean
# every remaining workflow: name a way it can fail substantively (in the PR body)
! grep -rn 'oikosbot\|push-email-notify' .github/workflows/ || true   # if deleted, gone
```

## Evidence to capture

Before/after workflow inventory with a one-line justification per deletion or
merge; the lockfile diff; the first-run findings for any adopted wrapper;
the updated known-issues file.

## Non-goals / do not do

- Do not delete `proofs.yml` or any job that enforces a stated invariant.
- Do not delete the only copy of a health file while the org-level copy is
  unverified.
- Do not touch other repos (the org `.github` repo is a separate session).
- Do not bundle the CHANGELOG/version work here (G-05 owns it).

## Stop conditions

- If deleting a workflow would remove the only enforcement of an invariant
  named in README/READINESS, stop and either keep it or update the ledger to
  name the gap — do not leave a silent hole.
- If the governance wrappers' first run produces real findings you cannot
  triage in this session, adopt them but leave the triage as a named open item
  with the run reference.

## Definition of done

≤5 real workflows with a regenerated lockfile; governance/hypatia decided and
the known-issues entry closed; root surface justified; PR body enumerates
every removal with its reason and lists anything that lost a gate.
