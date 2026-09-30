<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
<!-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell -->

# Joinery Programme — standing conventions

These rules are binding on every prompt in this pack. Each prompt repeats a
condensed form so it stays usable when pasted outside a groove checkout; this
file is the long form.

## 1. The one non-negotiable: evidence over claims

Every status word you write — *verified*, *tested*, *complete*, *green*,
*safe* — must be backed in the same PR by **a command that was run, an
artifact that exists at a path, and a date**. This is the estate's
`PROOF-NEEDS` discipline (`groove/docs/REPO-LAYOUT.adoc`, "the one
template-era keeper").

Forbidden, always:

- invented completion percentages or grades;
- counting template theorem names as progress;
- quoting a historical CI run as a fresh result without saying so;
- downgrading an honest "OPEN" to make a ledger look finished;
- a test that would pass if the feature under test were deleted (vacuous
  green), or a gate with `continue-on-error` added to get past it.

When something is still open after your work, the ledger must say **open**, and
your report must say why.

## 2. Gates, IDs and ledgers

- **Never weaken a gate to make it pass.** No `continue-on-error`, no
  `|| true` on a substantive check, no `--check` loop instead of an
  `ipkg --typecheck`, no per-file Idris check (it exits 0 on module-load
  errors — measured, see `groove/proofs/README.adoc`).
- **Conformance IDs are append-only.** Adding `CONF-*-nn` requires an
  executable test of the same name in the same PR (groove's `spec-consistency`
  CI job enforces ID↔test pairing). Never renumber; never delete an ID the
  reference implementation passes.
- **Update the honest ledger in the same PR**: `READINESS.md` /
  `docs/status/*.adoc` / `docs/PROOF-NEEDS.adoc` / `docs/KERNEL.adoc` as the
  repo demands — even when the update is "still open, now blocked on X".
- An **aspirational** section stays labelled aspirational until an executable
  test exists. "Target" is not "implemented".

## 3. Layer discipline (the joinery canon, groove ADR 0009)

`cleave` = the surface (lifetime, rank, posture) · `groove` = the protocol
(discovery, negotiation, session) · `spline` = the data-form (the bytes that
pass through). Consumers — burble, gossamer, panll, … — are **not** layers and
keep their own names and metaphors.

Hard boundaries:

- **No posture, rank, lease-state or lifecycle witness in wire bytes**
  (cleave TS-7; spline ADR 0004). Noninterference is demonstrated by controlled
  cross-posture captures, not by a text search of one capture.
- **Do not invent a wire format.** spline aligns to what consumers already
  speak (Bebop control plane, WebRTC/RTP media plane). No new serialiser.
- **Do not lift code** from `protocol-squisher` / `squisher-corpus` (a frozen
  provenance archive — evidence, not a source) or from the phantom gossamer
  proof modules; rebuild proofs against the actual types (cleave G-6).
- **Do not make groove normative text depend on cleave or spline** while the
  LAYERING annex is still annex status. Coupling lands as an inter-repo
  contract (prompt J-01), then as implemented-and-tested annex text.

## 4. Repository hygiene (groove ADR 0008, `docs/REPO-LAYOUT.adoc`)

- ≤5 workflows per repo, each exercising a real artifact. A workflow that
  cannot fail for a substantive reason is deleted.
- **No per-directory AI-manifest files**; at most one root manifest.
  `.machine_readable/` trees, `docs-template/`, `coordination.k9`, `session/`,
  empty `docs/theory` / `docs/whitepapers` scaffold trees: gone, and not
  coming back.
- Community-health files (`GOVERNANCE`, `CONTRIBUTING`, `SECURITY`,
  `CODE_OF_CONDUCT`) belong once in the org-level `.github` repo. Per repo:
  at most a minimal `SECURITY.md`.
- Never hand-edit a generated table; it derives from
  `groove/registry/groove-registry.json` (ADR 0006). Extend the registry first,
  regenerate, then commit both.
- SPDX header on every new file. Code MPL-2.0, prose CC-BY-SA-4.0.

## 5. Process

- One prompt = one focused branch and PR series off `main`; conventional
  commits; never force-push `main`; never push another branch than the one the
  session name in the prompt specifies.
- Run the repo's own verification block before claiming a result (each prompt
  names the commands; `docs/BETA-RUNBOOK.md`, `docs/status/READINESS.adoc` and
  `tests/*.sh` are authoritative per repo).
- Pin third-party actions and toolchains the way the repo already does; run
  `gh actions-lock --no-fix` in groove when workflows change.
- Secrets: never log, commit or paste key material, tokens or handles. Redact
  debug output. `GROOVE_SIGNING_KEY` is injected, never passed on a command
  line.
- Loopback is not authentication. Origin/Host checks mitigate browser paths,
  not hostile local processes.

## 6. Stopping honestly

Stop and report (don't guess) when:

- the prompt's work needs an **owner decision** the prompt marks as such;
- a dependency prompt has not landed and its deliverable is a precondition;
- the evidence you can produce would be weaker than the claim the docs
  currently make — say so instead of quietly matching the docs;
- a gate can only go green by weakening it. That is a stop condition, not a
  work item.

Report format at the end of every run: commands run · observed results ·
files changed · evidence paths · what is now **closed** · what remains
**open** · owner decisions needed. "One point, no further" — expand the claim
only as far as the evidence reaches.
