<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
<!-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell -->

# The Joinery Programme

**A prompt pack that takes `cleave`, `groove` and `spline` from where they
actually are to where their own honest ledgers say "done" — individually
finished and jointly integrated.**

Revision 2026-09-29. Every factual claim below was verified against the three
repositories on that date; the files and commands are named so you can
re-verify before you rely on anything here.

---

## How to use this pack

1. **One prompt = one work session = one focused PR series.** Paste the whole
   prompt file into an agent (or hand it to a human) working in a checkout of
   the named repository. Each prompt is self-contained: mission, verified
   ground truth, deliverables with paths, acceptance commands, evidence to
   capture, non-goals, stop conditions, definition of done.
2. **`CONVENTIONS.md` is binding.** Each prompt carries a condensed copy of
   the rules; the long form lives in `CONVENTIONS.md`. The two rules that
   matter most: *evidence over claims* (no status word without a command, a
   path and a date) and *never weaken a gate to make it pass*.
3. **Phase order matters at the seams, not inside the tracks.** Run
   Phase 0 (J-01, J-02) first — everything else's acceptance tests reference
   the contract it defines. Then Phase 1 closes the three interlocking open
   gates (spline criterion (d), groove beta gate 1, cleave TS-7) with one real
   consumer pairing. Phases 2–3 finish the three repositories individually and
   can run as three parallel lanes (groove / cleave / spline) plus hygiene
   work. Phase 4–5 are the release train and the honest acceptance review.
4. **Parallel lanes, with one conflict rule:** never run two prompts against
   the same repository at the same time. Different repositories in parallel is
   the intended mode.
5. **A prompt that reports `OPEN` is doing its job.** The failure mode this
   pack is built to prevent is not unfinished work — it is unfinished work
   described as done.
6. **Optional — open them as issues.** `index.json` is machine-readable
   (id, repo, phase, depends_on, blocks, owner_decision, size), so one issue
   per prompt is a loop. From a groove checkout:

   ```sh
   python3 - <<'PY'
   import json, subprocess
   d = json.load(open("docs/joinery-programme/index.json"))
   for p in d["prompts"]:
       repo = d["repos"][p["repo"]]
       subprocess.run(["gh", "issue", "create", "-R", repo,
                       "--title", f'{p["id"]}: {p["title"]}',
                       "--body-file", "docs/joinery-programme/" + p["file"]], check=True)
   PY
   ```

   Single prompt, by hand:
   `gh issue create -R metadatastician/groove --title "J-01: ..." --body-file docs/joinery-programme/J-01-joinery-interface-control-document.md`

---

## 1. Where the trio actually is (verified 2026-09-29)

### groove — `metadatastician/groove` @ `25b2c82`, v0.3.0

| Real today | Evidence path |
|---|---|
| Spec, transport, IPv6T, modularity, innervation, layering annex | `spec/*.adoc` |
| Registry as SSOT (ports, services, capability/protocol types, key pins) | `registry/groove-registry.json` (ADR 0006) |
| Reference provider: discovery, connect/heartbeat/disconnect, leases, hash-chained attestation, signatures, redaction, bounded sessions | `provider/src/lib.rs` (946 lines), `provider/tests/{conformance.rs,boundaries.rs,signing.rs,probe_integration.rs}` |
| Executable conformance: 4 × `CONF-L1-*`, 7 × `CONF-L2-*`, 5 × optional lease IDs `CONF-L2-08..12`; CI enforces ID↔test pairing | `spec/CONFORMANCE.adoc`, `.github/workflows/ci.yml` (`spec-consistency` job) |
| CLI: init / validate / probe / registry / check-compat / mesh / sign, incl. registry-pinned signature verification | `cli/src/*.rs`, `cli/src/sign.rs` |
| Proofs wired and gated: 12 Idris2 modules, `ipkg --typecheck`, vendored-dep integrity job | `proofs/`, `groove-proofs.ipkg`, `.github/workflows/proofs.yml` |
| Browser extension (Firefox MV2) + generated target table + vendored client drift check | `clients/browser-extension/`, `clients/js/groove-client.js` |

| Open today | Evidence path |
|---|---|
| Beta gate 1: no real Burble–Gossamer typed-token capture (stale/forged rejection, cleanup both sides) | `READINESS.md` |
| Beta gate 2: browser harness is experimental (regex A2ML, 32-bit toy hash, bookkeeping not a handshake) — not beta-qualified | `harness/groove-harness.js`, `READINESS.md` |
| Beta gates 3–4: no home-context + six external CRG trials; no deployment owner/environment or restart/rollback rehearsal | `READINESS.md`, `docs/BETA-RUNBOOK.md` |
| Levels 3 and 4 are explicitly "aspirational (not yet testable)"; no `CONF-L3/L4` IDs exist | `spec/CONFORMANCE.adoc` |
| PROOFS-1 residual: modules still `Gossamer.ABI.*`, stated against the vendored gossamer ABI, not the current manifest data model | `proofs/README.adoc` |
| `CHANGELOG.md` has no 0.3.0 entry although the version line advanced (ADR 0011) | `CHANGELOG.md`, `cli/Cargo.toml` |
| 12 workflow files where ADR 0008 allows ≤5; `governance.yml` / `hypatia-scan.yml` open question | `.github/workflows/`, `docs/dev-notes/known-issues.md` |
| GRV6 reference pinned to Zig 0.14.1; migration to Zig 0.16 Io API untracked in code | `reference/ipv6t/`, `.github/workflows/ci.yml` (zig job comment) |

### cleave — `metadatastician/cleave`, v0.1.0

| Real today | Evidence path |
|---|---|
| Kernel demonstrated at **one** dial point: ranked owns-arena (RC-1), linear handle (move + debug drop-bomb), soft/hard leases (RC-6), postorder teardown to ⊥ with residue 0 (RC-13 degenerate), posture as a function of stage (KERN-7) | `src/*.rs` (945 lines), `tests/kern.rs` (KERN-1..7 + companions), `tests/boundaries.rs` |
| Kernel contract document, honest obligation ledger, honest gap list | `docs/KERNEL.adoc`, `docs/PROOF-NEEDS.adoc` |
| Idris2 mirrors typecheck in CI (5 modules, zero `believe_me`/`assert_total`/`postulate`) | `proofs/`, `cleave-proofs.ipkg`, `tests/check_proofs.sh`, `.github/workflows/kernel.yml` |
| Normative invariants RC-1..13 and TS-1..7 with falsifier tables | `docs/standards/RANKED-OWNERSHIP-CLEAVE.adoc` |

| Open today | Evidence path |
|---|---|
| Full surface: no second dial point (no SNIF/wasmtime stage) | `docs/architecture/CLEAVE-ENGINE-DESIGN.adoc`, `docs/KERNEL.adoc` (non-goals) |
| O-6 joint completion and O-7 clean-XOR-rupture need two peers and a wire — neither exists | `docs/PROOF-NEEDS.adoc`, `docs/status/PROOF-STATUS.adoc` |
| TS-3 per-operation floors, TS-5 cross-surface atomicity, TS-6 ephapax `let!` layer, TS-7 vacuous in-process | `docs/standards/RANKED-OWNERSHIP-CLEAVE.adoc` §TS |
| G-4: RC-1..13 vacuously green outside the kernel — no real Hard-owns-Soft graph in any consumer | `docs/PROOF-NEEDS.adoc` |
| Groove coupling is aspirational only (the annex lives in groove; cleave has no adapter) | `groove/spec/LAYERING.adoc`, ADR 0007 in groove |
| Leases are caller-ticked; no autonomous scheduler; single-process tests cannot establish remote behaviour | `docs/status/READINESS.adoc` |
| ADR series has only the template plus ADR 0001 | `docs/decisions/` |
| 11 workflow files; generic RSR-era docs (`RSR_OUTLINE.adoc`, `STATE-VISUALIZER.adoc`, `docs/{governance,legal,onboarding,practice,reports}/`) still present | `.github/workflows/`, `docs/` |

### spline — `metadatastician/spline`

| Real today | Evidence path |
|---|---|
| Four seed ADRs: plane multiplexing (= "parallelise"), consumer schema ownership, typed cap-token boundary, promotion criteria + stability rule | `docs/decisions/0002..0005` |
| Historical promotion ledger: (a) burble Bebop default, (b) gossamer second decoder, (c) groove CI conformance — recorded as **shipped then**, not reproduced now | `docs/alignment/voice-signal-plane.adoc` |
| Machine-readable alignment + recorded frames + executable checker (in groove) | `groove/registry/bebop-voice-signal-alignment.json`, `groove/spec/conformance/bebop/voice_signal_frames.hex`, `groove/scripts/check-bebop-alignment.mjs` |
| A one-shot local capture (2026-09-07, uncommitted worktrees) recorded honestly as *not* promotion evidence | `docs/alignment/voice-signal-plane.adoc` §Local consumer capture |

| Open today | Evidence path |
|---|---|
| Criterion (d) — the live typed-token pairing with posture provably absent from bytes: **OPEN** | `docs/alignment/voice-signal-plane.adoc`, `docs/status/BETA-ACCEPTANCE.adoc` |
| ADR 0005 forbids `src/` until all four criteria hold; there is no framing implementation, only prose + fixtures | `docs/decisions/0005-promotion-criteria.adoc` |
| Stability rule unmet: the mapping is unversioned and one implementation deep | `docs/decisions/0005-promotion-criteria.adoc` §What "stable" would additionally need |
| `docs/status/TEST-NEEDS.adoc` claims test artifacts that do not exist (`scripts/validate-template.sh`, `benches/template_bench.sh`, `src/interface/ffi/…`), and `docs/QUICKSTART.adoc` tells the reader to clone `rsr-template-repo` | verified missing 2026-09-29 |
| README/READINESS reference `.machine_readable/descriptiles/`, which does not exist in the tree | verified 2026-09-29 |
| 9 workflow files; template-era `docs/{theory,whitepapers,wikis,practice,developer,reports,governance,legal,onboarding}/` trees | `.github/workflows/`, `docs/` |

### The system — the integration nobody owns yet

There is **no inter-repository contract**, no end-to-end gate, and no release
train. Cross-repo checking today is one direction only: spline's
`tests/check_alignment.sh` shells into a groove checkout, and groove's CI runs
the alignment checker as spline criterion (c). cleave's CI does not touch
either repo; groove's LAYERING annex is explicitly aspirational; the trio has
never been exercised together in one run.

---

## 2. What "complete" means (each repo's own bar — do not move it)

- **groove is complete when** the four `READINESS.md` release gates are
  closed with evidence: the real typed-token capture (1); a decided, honest
  client surface (2); home-context use plus six diverse external CRG trials
  with feedback (3); green remote CI, a named deployment owner/environment and
  a rehearsed restart/rollback (4). Levels 1–2 stay executable; Levels 3–4 are
  either executable or explicitly descoped by ADR; proofs are re-founded or
  the residual is restated honestly.
- **cleave is complete when** the bar in `docs/KERNEL.adoc`'s definition of
  done holds for the **full surface**, not one point: a second wired dial
  point with a real capability, soft *and* hard groove, staircase teardown
  actually invoked, the linear handle actually linear, proofs actually
  guarding running code — plus O-6/O-7 closed over a real two-peer wire, and
  TS-7 demonstrated as noninterference over actual bytes.
- **spline is complete when** ADR 0005's four criteria all hold (d last), the
  stability rule is met (versioned mapping with a compatibility rule; two
  independent implementations passing the same fixtures), the wire layer
  exists as an implementation that is posture-blind *by construction*, and
  beta acceptance (`docs/status/BETA-ACCEPTANCE.adoc`) is satisfied. Until
  then: fixtures and docs, no `src/`.
- **The trio is complete when** J-01's contract is pinned and checked, J-02's
  gate runs all three together with zero `PENDING` scenarios, J-03's live
  pairing is captured on real consumer revisions, and one release train (J-04)
  ships coherent versions with one honest status page. Owner decisions in
  J-05 (external trials, deployment) are the last mile and cannot be faked by
  a local run.

---

## 3. Sequencing

```
Phase 0   J-01  contract + pins ─┬─ J-02  trio gate (scenarios may be PENDING)
                                 │
Phase 1   S-01 ─┐                │
          G-01 ─┼─ J-03  live consumer pairing ──► closes spline (d), groove G1, cleave TS-7
          C-01 ─┘                │
                                 │
Phase 2   groove lane: G-02 G-03 G-04 G-05 G-07
          cleave lane: C-02 C-03 C-04 C-06
          spline lane: S-02 S-03 S-04 S-06
                                 │
Phase 3   G-06 (operations) · C-05 (proofs/refinement) · S-05 (interop/fault/beta)
                                 │
Phase 4   J-04  release train (versions, pins, status page)
                                 │
Phase 5   J-05  external trials + final honest acceptance
```

Hard dependencies: J-02 and J-04 need J-01 · J-03 needs S-01+G-01+C-01 ·
S-03/S-04 need S-02 (which needs J-03) · C-04 needs C-01+C-02 · J-04 needs
J-02+S-04+G-02+C-04 · J-05 needs J-04+G-06+S-05.

---

## 4. The integration contract in one page

Four coupling points carry the whole system. J-01 writes them down as
numbered interface points with producer, consumer, versioning and error
mapping; J-02 tests them end-to-end.

1. **Session ↔ surface.** A groove session (§4 state machine: connect →
   heartbeat → disconnect, with the §4.6 lease as the wire-visible occupancy
   discipline) *is*, under the annex, a cleave surface: connect mints,
   disconnect linearly consumes, graceful teardown reaches ⊥ = zero residue.
   Today groove implements the wire behaviour natively and cleave has no
   adapter (prompt C-01).
2. **Leases are shared vocabulary.** `SPEC.adoc` §4.6 already cites cleave
   RC-6 by name: soft expires to a zero-residue wipe, hard refreshes on
   heartbeat and degrades after three whole missed TTL windows. One vocabulary,
   two implementations with one conformance fixture set (prompts C-01, J-02).
3. **Handles and tokens.** groove mints an authority handle; on the wire it is
   an opaque string (spline ADR 0004); at each endpoint it is a linear handle
   consumed exactly once (cleave RC-13/O-5); expiry *is* consumption performed
   by the provider (§4.6). Posture never appears in bytes (cleave TS-7).
4. **Planes.** spline owns the carriage: control = Bebop (typed, reliable,
   ordered), media = WebRTC/RTP (lossy, low-latency, no head-of-line block);
   "parallelise" means multiplexing independent typed planes of one groove
   session over one connection (spline ADR 0002). groove specifies *which*
   planes a session carries; cleave knows nothing about bytes; spline knows
   nothing about rank, lifetime or posture.

**Must never cross:** rank/ownership witnesses, posture triples, lease state
as such, or lifecycle bookkeeping into any byte encoding; a new serialiser
competing with Bebop/WebRTC; normative groove text depending on cleave/spline
while the layering remains an annex.

---

## 5. Guardrails (the traps this estate has already paid for)

- "Wire first — unwired is not done" is spline's own rule; a design that is
  right and unexercised counts as open.
- **Vacuous green**: RC-1..13 are vacuously true outside the kernel (G-4); a
  gate that cannot fail for a substantive reason is deleted (ADR 0008); a
  per-file Idris `--check` exits 0 on module-load errors (claims a check that
  isn't one).
- **Historical evidence decays**: the (a)/(b)/(c) ledger and the 2026-09-07
  capture say so themselves. Re-run, date, and record; never present a
  remembered result as fresh.
- **Local ≠ external**: six external CRG trials and a deployment owner cannot
  be substituted by local test configurations (groove, cleave and spline
  ledgers all repeat this).
- **Loopback is not authentication**; origin/Host checks do not defend against
  hostile local processes.
- **No template residue**: per-directory AI manifests, empty doc trees,
  20-workflow CI stacks, template quickstarts pointing at other repos. The
  `PROOF-NEEDS` honest ledger is the one template-era artifact worth keeping.
- **No borrowed foundations**: `protocol-squisher` is evidence of what
  stalled, not a source; phantom gossamer proofs must be rebuilt, not lifted.

---

## 6. Prompt roster

| Phase | ID | Repo | Prompt | Depends | Owner decision |
|---|---|---|---|---|---|
| 0 | **J-01** | groove | Joinery interface control document + pin registry | — | no |
| 0 | **J-02** | groove | Joinery integration gate (trio end-to-end; honest PENDING) | J-01 | no |
| 1 | **S-01** | spline | Close criterion (d): acceptance harness, fixtures, reconciliation | — | no |
| 1 | **G-01** | groove | Handles/leases bound to real capability operations | — | no |
| 1 | **C-01** | cleave | A groove session as a cleave surface | J-01 | no |
| 1 | **J-03** | spline | The live Burble–Gossamer pairing capture | S-01,G-01,C-01 | **yes** |
| 2 | **S-02** | spline | Promotion review — unblock `src/` | J-03 | **yes** |
| 2 | **S-03** | spline | Version the mapping; two independent implementations | S-02 | no |
| 2 | **S-04** | spline | Implement the wire layer (multiplexing, opaque tokens, posture-blind) | S-03,J-01 | no |
| 2 | **S-05** | spline | Interop matrix, fault injection, beta acceptance | S-04 | no |
| 2 | **S-06** | spline | Hygiene: template residue, false test claims, ≤5 workflows | — | no |
| 2 | **G-02** | groove | Levels 3 and 4 executable — or descoped by ADR | — | no |
| 2 | **G-03** | groove | Proofs re-founding + Level 3 Idris2/Zig FFI | G-02 | **yes** |
| 2 | **G-04** | groove | Browser surface: implement+integrate, or retire by ADR | — | **yes** |
| 2 | **G-05** | groove | Spec/registry coherence, A2ML, registration dossier, GRV6 | — | **yes** |
| 2 | **G-07** | groove | Hygiene: workflows, governance/hypatia, CHANGELOG | — | no |
| 3 | **G-06** | groove | Beta operations: load, multi-process, rollout rehearsal | G-04,J-03 | **yes** |
| 2 | **C-02** | cleave | Two peers: O-6 joint completion + O-7 clean-XOR-rupture | — | no |
| 2 | **C-03** | cleave | Second dial point: SNIF/wasmtime with crash isolation | — | **yes** |
| 2 | **C-04** | cleave | Full dial + RC-1..13/TS-3/5/6/7 + G-4 + scheduler | C-01,C-02 | **yes** |
| 3 | **C-05** | cleave | Proofs and the refinement boundary | C-04 | no |
| 2 | **C-06** | cleave | Hygiene and the missing ADR series | — | no |
| 4 | **J-04** | groove | Release train: versions, pins, changelogs, status page | J-02,S-04,G-02,C-04 | **yes** |
| 5 | **J-05** | groove | External trials, CRG evidence, final honest acceptance | J-04,G-06,S-05 | **yes** |

## 7. Honest limits of this programme

- It cannot manufacture external evidence: CRG grade B/A needs six diverse
  external targets and field feedback (owner-led, prompt J-05), and
  production-beta approval needs a named deployment owner and a rehearsal in
  the real environment (prompt G-06).
- Consumer-repo work (burble, gossamer) is referenced, not owned here; where a
  prompt needs it, it says so and stops rather than editing repos it does not
  own without a session that targets them.
- No prompt claims proof-to-runtime refinement, universal noninterference,
  distributed liveness under loss, or OS-resource observation. Each points at
  the honest ledger row it is allowed to touch.
- Completion of these prompts means *each layer's own bar is met with dated
  evidence and the seams are tested*. It does not mean the estate has users,
  adopters or a production deployment — those are J-05's business and the
  owner's decision.

## 8. Maintaining this pack

When a prompt lands, update `index.json` (status field is deliberately absent:
the repositories' own ledgers are the status) and, if the roster changes,
this README's tables. Delete the pack once every prompt has landed — it is a
planning artifact, not a permanent document; groove's `docs/REPO-LAYOUT.adoc`
applies to it too.
