<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Groove readiness — 2026-09-05

Release decision: **not production-beta approved yet**. This assessment covers
this repository's Rust CLI/provider, JavaScript client, Zig reference and Idris
models. It does not assert estate-wide deployment or successful external trials.

| Component | Verified locally | Remaining boundary |
| --- | --- | --- |
| Rust CLI and provider, v0.3.0 | Locked debug/release tests, malformed-request and lifecycle regressions, manifest signatures, finite conformance suite | Production load, multi-process integration and target-specific deployment |
| Provider security | Random bearer tokens; metadata redaction; strict request framing; bounded sessions, sockets, audit history and read deadline | Loopback is not authentication against malicious local software; deployment must decide trust and pinning |
| Browser client | 12 Bun unit/stub tests plus a real Rust-provider lifecycle test, 9 structural checks, web-ext lint with zero errors/warnings | A real Firefox session against the actual consumer deployment is still needed; these are not GUI tests |
| GRV6 Zig reference | 10 tests pass on Zig 0.14.1; CI now requires this job | Not the same evidence as an independent production provider |
| Bebop alignment | 11 recorded variants and three checker self-checks | Not a live Burble–Gossamer capture or universal codec proof |
| Idris models | Package-based checks and valid/invalid controls; see proofs/README.adoc for compiler evidence | ABI-era structural models, not proof-to-runtime refinement |
| Browser harness | Inspected, not beta-qualified | Regex A2ML parsing yields empty capabilities; local connection bookkeeping is not a wire handshake; toy hash is not cryptographic attestation |

## Required release decisions and gates

1. Capture the real Burble–Gossamer typed-token boundary specified by Spline
   ADR 0005(d), including stale/forged-token rejection and cleanup on both sides.
2. Decide the supported beta client surface. The experimental harness must be
   implemented and integrated before it can be included; it is not silently
   counted as a working browser/native integration.
3. Complete home-context use and the CRG policy's six diverse external trials,
   incorporating feedback. Local test configurations are not external targets.
4. Review and merge the changes, obtain green remote CI, identify the deployment
   owner/environment, and rehearse shutdown/restart and rollback there.

## Reproduction and operations

See [the beta runbook](docs/BETA-RUNBOOK.md). Rust is pinned to 1.97.1, Bun to
1.3.14 and the Zig reference to 0.14.1. The locked `anyhow` dependency was updated
from 1.0.102 to 1.0.104 for
[RUSTSEC-2026-0190](https://rustsec.org/advisories/RUSTSEC-2026-0190.html);
the resulting lock passed the current RustSec audit with warnings denied.

The twelve Idris modules retain the upstream module names deliberately. Eight
vendored dependencies have an independent pinned-source integrity gate and must
not be changed locally. Mathematical limitations are recorded in
[proofs/README.adoc](proofs/README.adoc), not represented by an invented grade or
completion percentage.
