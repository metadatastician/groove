<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# J-04 · Release train: versions, pins, changelogs, one joinery status page

| | |
|---|---|
| **Repository** | all three (`groove` hosts the joint status page) |
| **Phase** | 4 — completion |
| **Depends on** | J-02 (gate), S-04 (wire layer), G-02 (levels decided), C-04 (full dial) |
| **Blocks** | J-05 |
| **Owner decision** | **yes** — version numbers for a joint release are an owner call |
| **Size** | medium (1 PR per repo + one joint page) |

## Role

You are an autonomous coding agent working across the three checkouts. You are
the release engineer: coherent versions, pinned refs, real changelogs, one
status page where every claim traces to an artifact.

## Mission

Ship the first coherent joinery release: a version matrix the three repos agree
on, pins that name the exact refs the gate tested, changelogs with real
entries, and a single status page whose every cell links to dated evidence —
including honest `OPEN` cells.

## Ground truth (verified 2026-09-29 — re-verify)

- Versions today: groove `0.3.0` (cli + provider; ADR 0011), cleave `0.1.0`,
  spline unversioned seed (CHANGELOG scaffold only; promotion recorded by S-02).
- groove's CI checks version coherence across `cli/Cargo.toml`, `spec/SPEC.adoc`
  and `README.adoc` only; `CHANGELOG.md`'s newest entry is 0.2.0 — G-05 adds
  the 0.3.0 entry and extends the check; do not duplicate.
- `registry/joinery-pins.json` (J-01) names the refs and digests the gate uses;
  `registry/groove-registry.json` stays the port/service SSOT.
- ADR 0003/0011 govern groove's version line; cleave and spline have their own
  changelogs to make real.
- Estate conventions: Keep a Changelog; conventional commits; no invented
  dates; tags only when the tree is green.

## Read first

groove `docs/decisions/0003-version-line.adoc`, `0011-version-line-0-3-0.adoc`,
`registry/joinery-pins.json`, `tests/joinery/` (J-02 evidence), `README.adoc`,
`CHANGELOG.md` · cleave `README.adoc`, `CHANGELOG.md`, `docs/KERNEL.adoc` ·
spline `README.adoc`, `CHANGELOG.md`, `docs/status/PROMOTION-REVIEW-*.adoc`.

## Deliverables

1. **Version matrix (owner decision)** recorded in J-01's ICD or the joint
   status page: per repo — current version, released-this-train version, the
   pin the train tested, and the compatibility statement between them
   (groove protocol version ↔ cleave surface API ↔ spline framing/mapping
   version). Proposed defaults if the owner has no preference: groove
   `0.4.0` (levels 3/4 disposition + gate), cleave `0.2.0` (dial + two-peer +
   adapter), spline `0.1.0` (first promoted implementation).
2. **Per-repo release edits**: version bumps where justified; CHANGELOG entries
   with real dates and evidence links; README status lines paraphrased from
   ledgers (no adjectives the ledger cannot support); spec `Status` sections
   updated (groove); `docs/status/*` refreshed.
3. **Pins refresh**: `registry/joinery-pins.json` updated to the tested refs
   with `verified` blocks; the checker green; a note describing what a pin
   bump requires (re-run the gate; re-verify digests; record).
4. **`docs/JOINERY-STATUS.adoc`** in groove — one page, one table per layer:
   state · version · evidence link · CRG grade with its evidence · open items
   with owners. Plus the system row: gate status (scenarios/pending), the last
   capture path, the compatibility matrix, and the honest "not claimed" list
   (external trials, deployment, refinement, media-plane). The page is
   generated where possible (from `joinery-pins.json` + the gate's
   `scenarios.json`) so it cannot drift; a hand-written page that disagrees
   with the gate output is a defect.
5. **Tags and release notes**: tag each repo at the tested commit with the
   version; write release notes that say what is *not* included. No tag until
   its repo's CI is green (record run ids).
6. **Gate evidence attached**: the J-02 run with zero PENDING, its evidence
   bundle path, and the J-03 capture referenced from all three repos' status
   docs.

## Acceptance criteria

```sh
bun groove/scripts/check-joinery-pins.mjs
bash groove/tests/joinery/gate.sh --all        # zero PENDING; evidence bundle written
python3 - <<'PY'
# version matrix and status page agree with the three Cargo.toml/tag facts
PY
gh run list -R metadatastician/groove --limit 5      # green, ids recorded
gh run list -R metadatastician/cleave --limit 5
gh run list -R metadatastician/spline --limit 5
git -C <repo> describe --tags --exact-match          # tags exist at tested commits
```

## Evidence to capture

The version matrix; the gate run + bundle; per-repo CI run ids; the status
page rendered; the tag list; every claim's evidence link checked once by hand
(a link that 404s is a defect).

## Non-goals / do not do

- Do not tag a repo whose CI is red or whose status doc still claims something
  the ledger does not support.
- Do not invent a version policy change without an ADR (groove ADRs 0003/0011
  govern).
- Do not hand-write numbers the gate can produce; generate from the gate's
  `scenarios.json` and the pins file.
- Do not announce the release anywhere; that is the owner's call in J-05.

## Stop conditions

- **Owner decision** on the version numbers if the defaults do not fit the
  estate's conventions; prepare the matrix and stop.
- If the gate still reports PENDING, stop: a release train with pending
  scenarios is a release of the gate, not of the system — report which prompt
  blocks.

## Definition of done

Version matrix agreed and recorded; per-repo releases coherent (versions,
changelogs, README/spec statuses, tags, green CI ids); pins refreshed and
checked; `JOINERY-STATUS.adoc` generated and consistent with the gate; PR
bodies list commands, results, and the exact remaining open items handed to
J-05.
