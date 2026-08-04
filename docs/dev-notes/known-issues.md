# Known issues

Verified, scoped gaps found during hygiene passes. Entries here are real and
actionable but were deliberately *not* fixed in the PR that found them,
usually because the fix needs its own validation pass rather than a rushed
drive-by change.

## Missing `governance.yml` / `hypatia-scan.yml` (opened 2026-08-04)

**Status:** open, unscheduled.

As of 2026-08-04, `groove`'s `.github/workflows/` has `actions.lock`,
`ci.yml`, `codeql.yml`, `oikosbot.yml`, `pages.yml`, `proofs.yml`,
`push-email-notify.yml`, and `secret-scanner.yml` (PR #32) — but no
`governance.yml` and no `hypatia-scan.yml`. Two sibling repos in the same
estate have both:

- `metadatastician/enaction-engine` — `.github/workflows/governance.yml` and
  `.github/workflows/hypatia-scan.yml`, both calling
  `hyperpolymath/standards/.github/workflows/governance-reusable.yml` and
  `hyperpolymath/standards/.github/workflows/hypatia-scan-reusable.yml`
  respectively, pinned at
  `@bd0df9ead7faf0cdfe0e13e7966d91e28d0101d4`.
- `metadatastician/idaptik-ums-canonical` — the same two workflows, calling
  the same two reusables, but pinned at a *different* SHA
  (`@d7c22711e830e1f383846472f6e9b99debdb201e`). The two siblings disagree
  on which commit of `standards` they trust — check
  `hyperpolymath/standards` for the current recommended pin before copying
  either one verbatim; don't assume either SHA is still current.

Both workflows are one-job wrappers (see either sibling's copy for the exact
shape): `governance.yml` runs on push/PR/`workflow_dispatch` with
`permissions: {actions: read, contents: read}`; `hypatia-scan.yml` adds a
weekly cron and `security-events: write` (it feeds SARIF, separate from any
`hypatia-scan` job already living inside a combined `static-analysis-gate.yml`
if one gets added later — see enaction-engine's workflow for how it
disambiguates the two consumers of the same tool).

**Why this wasn't just fixed here:** the two reusables aren't single checks —
`governance-reusable.yml` bundles licence consistency, well-known files, and
security-policy presence checks; `hypatia-scan-reusable.yml` runs Hypatia,
whose unwrap/panic rules have a known false-positive rate against comments
and discarded bindings (see the estate's `hypatia-rule-precision` /
`hypatia-unwrap-rule-matches-comments` notes) and would need a real read of
the findings, not a blind merge. Wiring these in also means running the new
job through `gh actions-lock` (this repo already depends on the lockfile
per PR #29 — an unlocked reusable `uses:` is exactly the kind of thing that
caused the repo-wide `startup_failure` PR #29 fixed) rather than hand-editing
the workflow after the fact.

**Suggested next step:** copy one of the two sibling workflows, re-pin to the
current `standards` commit, run `gh actions-lock` (or the estate's usual
lockfile tool) to regenerate `actions.lock`, open the PR, and actually read
the first run's findings before merging — don't just check for green.
