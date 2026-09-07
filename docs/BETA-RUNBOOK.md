<!-- SPDX-License-Identifier: MPL-2.0 -->
# Groove beta runbook

This is a reproducible local candidate procedure, not deployment approval.
The reference provider serves discovery and session bookkeeping. It does not
implement arbitrary advertised application capabilities on behalf of consumers.

## Build and verify

Use Rust 1.97.1, Bun 1.3.14, Zig 0.14.1, and the Idris compiler pinned by CI.
From the repository root:

```sh
cargo build --locked --workspace --release
cargo build --locked --workspace
cargo test --locked --workspace
cargo test --locked --workspace --release
cargo clippy --locked --workspace --all-targets -- -D warnings
bun test clients/browser-extension/tests/client.test.mjs
bun test clients/integration/provider.test.mjs
bash clients/browser-extension/tests/validate_structure.sh
bun scripts/check-bebop-alignment.mjs
bun scripts/check-bebop-alignment.test.mjs
bash tests/check_proofs.sh /absolute/path/to/evidence
gh actions-lock --no-fix
```

In `reference/ipv6t`, run `zig build test` with 0.14.1. Run `cargo audit
--deny warnings` against a current RustSec database before each release; retain
the database revision and output. Package the Firefox extension excluding
`tests/**` and `scripts/**`, and run web-ext 8.9.0 lint with those same exclusions.
Lint and Bun tests do not replace actual Firefox integration.

## Start, trust and limits

Run `target/release/groove-provider` as an unprivileged process. It binds only
loopback, defaults to the registry port (6465), and supports `--port` and
`--manifest`. Startup rejects invalid manifest shapes. Do not expose the port
through a public reverse proxy or forward it to an untrusted network.

Unsigned manifests are the default. A valid self-signature is not an identity
pin: consumers requiring identity must independently pin the provider key.
Use the deployment's protected secret injection for `GROOVE_SIGNING_KEY`; avoid
the command-line key option, shell history and logs. Configuration debug output
redacts the seed, but in-memory key zeroization is not claimed.

The provider rejects non-loopback Host headers and non-loopback HTTP(S) origins.
Requests without Origin are permitted for native clients. These checks mitigate
browser-origin and DNS-rebinding paths, not hostile local processes. Firefox
extension-origin behavior must be tested in the chosen target; no broad CORS
allowlist should be added merely to make a test pass.

Limits: 128 concurrent HTTP exchanges, 1,024 live sessions, a five-second overall
exchange deadline, and 4,096 retained audit records. Retained metadata is also
bounded: service IDs are limited to 128 bytes,
service versions to 64 bytes, and consumes lists to 64 entries. At session capacity, connect
returns 503; disconnect/lease expiry restores capacity. Saturated socket
capacity closes new exchanges. Requests are single-exchange HTTP/1.x; transfer
encoding is unsupported and rejected. Body/header framing errors are rejected,
including incomplete or duplicate Content-Length. Browser responses are capped
at 64 KiB and two seconds including the body.

The session token is bearer authority. Never persist it in telemetry or expose
it through mesh metadata. Lease-free sessions survive until disconnect or
process termination; callers must use disconnect or opt into a lease. The browser
extension now requests a hard lease with a 15-second TTL and renews it every
five seconds; provider expiry follows three missed TTL windows. A lost
disconnect response is ambiguous, so clients must recover without assuming
that the previous token is reusable.

## Observe, stop and recover

The CLI emits JSON-line audit records plus human-readable startup/shutdown lines.
Persist and filter these under the deployment's retention/access policy for a
complete history. The HTTP audit endpoint exposes only the bounded recent tail;
its first record may refer to an earlier, evicted hash. A hash chain without an
external trusted anchor is not tamper-proof storage. Audit state is in memory.

Send SIGINT for the explicit CLI shutdown path, or call `Server::shutdown()` in
an embedding. Dropping the server aborts its tracked tasks. Restart loses all
sessions and the in-memory audit chain; old tokens must fail and clients must
rediscover/reconnect. There is no persistence migration to run or session state
to restore. Before rollout, rehearse this against the real consumer pair and
verify both endpoints release their own resources.

Retain the previously approved binary/config and a separately stored key pin.
Rollback means stop the candidate and restart that approved version, followed
by discovery and fresh negotiation. Do not restore bearer tokens from logs.
No beta binary, deployment or rollback rehearsal has been approved by this file.
