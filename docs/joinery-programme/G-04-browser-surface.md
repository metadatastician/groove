<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# G-04 · Browser surface: implement and integrate the harness, or retire it by ADR

| | |
|---|---|
| **Repository** | `metadatastician/groove` |
| **Phase** | 2 — groove track |
| **Depends on** | nothing |
| **Blocks** | G-06, J-04 |
| **Owner decision** | **yes** — whether the browser surface is in or out of the beta contract |
| **Size** | medium (1–3 PRs) |

## Role

You are an autonomous coding agent working in a checkout of
`metadatastician/groove`. Beta gate 2 says the supported client surface must be
decided and the experimental harness must be implemented and integrated *before
it can be included* — or it is not silently counted as working.

## Mission

End the ambiguity: either make `harness/groove-harness.js` and the Firefox
extension real, tested members of the beta surface (real HTTP handshake, real
attestation verification, real content negotiation, real Firefox evidence), or
retire the harness from the supported surface with an ADR and update the
ledgers. Both outcomes are acceptable. "Experimental, unverified, still
described as a client" is not.

## Ground truth (verified 2026-09-29 — re-verify)

- `READINESS.md` names the harness's three defects: regex A2ML parsing yields
  empty capabilities; local connection bookkeeping is not a wire handshake;
  a toy hash is not cryptographic attestation.
- `harness/groove-harness.js` `_hashRecord` is a 32-bit string hash
  (`hash = ((hash << 5) - hash) + chr; hash |= 0`) — not a hash chain of the
  kind §5.1 requires.
- The Firefox extension is the better-tested client: `clients/browser-extension/`
  with `tests/client.test.mjs`, `tests/validate_structure.sh` (vendored-client
  drift), `tests/scripts/gen-targets.mjs --check` (registry drift),
  `web-ext@8.9.0` lint in CI, and a real Rust-provider lifecycle test in
  `clients/integration/provider.test.mjs`. READINESS still requires a real
  Firefox session against an actual consumer deployment.
- Browser-extension security posture: hard lease 15 s TTL renewed every 5 s;
  browser responses capped at 64 KiB / 2 s; the provider rejects non-loopback
  `Host` and `Origin`; no broad CORS allowlist may be added to make a test
  pass.
- ADR 0005 governs the extension's conformance dialect and generated port
  table (from the registry SSOT, ADR 0006).

## Read first

`READINESS.md` (gate 2) · `harness/groove-harness.js` ·
`harness/groove-panel-template/panel.html` · `clients/browser-extension/README.md`
and its tests · `clients/js/groove-client.js` ·
`docs/decisions/0005-browser-extension.adoc` · `spec/SPEC.adoc` §2.1/§4/§5 ·
`docs/BETA-RUNBOOK.md` (limits and origin rules).

## Deliverables — choose one path and execute it fully

### Path A — implement and integrate (recommended if the harness has a consumer)

1. Replace regex A2ML parsing with a real, tested parser **or** drop A2ML from
   the harness path and make the JSON path the documented one (do not keep a
   parser that silently yields empty capabilities). If A2ML is to be a real
   optional encoding, the parser gets its own tests and negative controls —
   coordinated with G-05's A2ML decision.
2. Replace the toy hash with a real SHA-256 hash chain over the §5.1 record
   shape (`event, provider, consumer, timestamp, hash, prev_hash`), verified
   against a real provider's chain in an integration test — not a
   self-consistent toy.
3. Replace bookkeeping with a real wire handshake: the harness must perform
   `GET /.well-known/groove`, connect with a lease, heartbeat, disconnect, and
   honour refusals (`400/409/410`) exactly as the provider emits them.
4. Integration test against a spawned `groove-provider` (build it in CI),
   covering: discovery, connect, hard-lease renewal, expiry degradation,
   stale/forged handle refusal, teardown, and hash-chain continuity.
5. Panel integration: the panel template uses the harness end-to-end; a
   checked-in script/test exercises the template path (a browser-less DOM
   harness is acceptable for CI; a real Firefox run is still required for the
   claim).
6. Update `READINESS.md` gate 2 with the decision and evidence; state
   explicitly what remains unverified (real Firefox session, deployment).

### Path B — retire the harness

1. ADR (`docs/decisions/0013-browser-harness-disposition.adoc`): the harness
   is removed from the supported client surface, with the reasons quoted from
   READINESS; the Firefox extension remains the browser client.
2. Delete `harness/` (or move it to `docs/archive/` with a one-line pointer)
   and remove harness claims from README/READINESS/anything referencing it.
3. Update the beta-surface section of `READINESS.md` with the decision.

Either path must also:

7. **Qualify the extension as far as locally possible**: run the real Firefox
   path (web-ext lint is not a GUI test) against a local provider or consumer
   and record the session (versions, steps, observations, failures). If no
   deployment exists to test against, say so and mark the remaining item open.

## Acceptance criteria

```sh
bun test clients/browser-extension/tests/client.test.mjs
bun test clients/integration/provider.test.mjs
bash clients/browser-extension/tests/validate_structure.sh
bunx --bun web-ext@8.9.0 lint --source-dir clients/browser-extension --ignore-files 'tests/**' 'scripts/**'
bun clients/browser-extension/scripts/gen-targets.mjs --check
# Path A only:
bun test harness/tests/*.mjs            # real handshake + hash chain against a spawned provider
# and: tamper with a provider hash chain → harness test FAILS
```

## Evidence to capture

Decision record; the tamper-detection demonstration (Path A); the Firefox
session record (version, steps, screenshots/log excerpts, failures); the
updated gate-2 row in `READINESS.md`.

## Non-goals / do not do

- Do not add a CORS allowlist, relax origin/Host checks, or disable TLS/Host
  validation to make a browser test pass.
- Do not claim "Firefox support" from lint plus unit tests — the ledger's own
  bar is a real session.
- Do not leave both a retired harness and its claims in the docs (Path B).

## Stop conditions

- **Owner decision** on Path A vs B if the harness's intended consumer
  (PanLL/Gossamer panels) cannot be identified in this repo or a linked one;
  the default without a consumer is Path B.
- If a real Firefox session cannot be run in this environment, record the
  exact blocker and leave the gate at "partially evidenced" — do not upgrade
  the claim.

## Definition of done

One path executed end-to-end; gate 2 row in `READINESS.md` reflects reality
with dated evidence; README/status tables consistent; any remaining open item
named; PR body lists commands, results and the decision rationale.
