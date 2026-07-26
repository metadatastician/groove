// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// groove-provider — the reference Groove provider (SPEC §2, §4, §5) and the
// target the executable conformance suite (spec/CONFORMANCE.adoc) runs
// against. Hand-rolled HTTP/1.1 over tokio, same style as the CLI's probe
// client: the reference implementation should be readable end to end.

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::Instant;

use groove::registry;
use groove::timefmt::rfc3339_now;

/// How often the lease sweeper looks for due leases. Short enough that the
/// conformance suite can use sub-second TTLs.
const SWEEP_INTERVAL: Duration = Duration::from_millis(50);

/// Lease TTL bounds (SPEC §4.6): reject degenerate and absurd TTLs.
const MIN_TTL: Duration = Duration::from_millis(10);
const MAX_TTL: Duration = Duration::from_secs(24 * 60 * 60);

/// Provider configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// Port to bind on both [::1] and 127.0.0.1. 0 = ephemeral (the actual
    /// port is reported by `Server::port`).
    pub port: u16,
    /// Manifest to serve; None = the built-in groove-ref manifest.
    pub manifest: Option<Value>,
    /// Echo attestation records to stdout as JSON lines.
    pub log_attestations: bool,
    /// Ed25519 seed: serve the manifest with a detached `signature`
    /// member (SPEC §2.1.5, ADR 0010). None = serve unsigned.
    pub signing_seed: Option<[u8; 32]>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: default_port(),
            manifest: None,
            log_attestations: false,
            signing_seed: None,
        }
    }
}

/// The registry assignment for the reference provider.
pub fn default_port() -> u16 {
    registry::find_service("groove-ref").map(|e| e.port).unwrap_or(6465)
}

/// The built-in manifest, per SPEC §2.1.2 and the groove-ref registry entry.
pub fn builtin_manifest(port: u16) -> Value {
    json!({
        "groove_version": "1",
        "service_id": "groove-ref",
        "service_version": env!("CARGO_PKG_VERSION"),
        "mode": "active",
        "capabilities": {
            "attestation": {
                "type": "attestation",
                "protocol": "http",
                "version": "1.0.0"
            }
        },
        "consumes": [],
        "endpoints": {
            "groove": format!("http://[::1]:{port}/.well-known/groove")
        }
    })
}

/// Lease mode on a connection (SPEC §4.6): the wire-observable shadow of
/// cleave RC-6 — soft expires, hard refreshes on heartbeat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LeaseMode {
    Soft,
    Hard,
}

impl LeaseMode {
    fn as_str(self) -> &'static str {
        match self {
            LeaseMode::Soft => "soft",
            LeaseMode::Hard => "hard",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct LeaseState {
    mode: LeaseMode,
    ttl: Duration,
    expires_at: Instant,
}

impl LeaseState {
    /// Is this lease due for reaping at `now`? Soft: at TTL. Hard: only
    /// after three whole missed TTL windows (a connection still being
    /// renewed is never reaped).
    fn due(&self, now: Instant) -> bool {
        match self.mode {
            LeaseMode::Soft => now >= self.expires_at,
            LeaseMode::Hard => now >= self.expires_at + self.ttl + self.ttl,
        }
    }
}

/// One live connection handle (linear: removed on disconnect or expiry).
struct HandleEntry {
    consumer_id: String,
    lease: Option<LeaseState>,
}

struct State {
    manifest: Value,
    /// Live connection handles (linear: removed on disconnect or lease
    /// expiry; a removed handle answers 410 forever after).
    handles: HashMap<String, HandleEntry>,
    /// Hash-chained provenance records (SPEC §5.1).
    attestations: Vec<Value>,
    last_hash: String,
    handle_counter: u64,
    log_attestations: bool,
}

impl State {
    fn offers(&self) -> Vec<String> {
        self.manifest["capabilities"]
            .as_object()
            .map(|caps| {
                caps.values()
                    .filter_map(|c| c["type"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn attest(&mut self, event: &str, consumer: &Value, capabilities: Vec<String>) {
        self.attest_with(event, consumer, capabilities, None);
    }

    /// Attest with extra fields merged into the record body before hashing
    /// (used by lease expiry to carry `residue`/`lease`, SPEC §4.6).
    fn attest_with(
        &mut self,
        event: &str,
        consumer: &Value,
        capabilities: Vec<String>,
        extra: Option<Value>,
    ) {
        let mut record_body = json!({
            "event": event,
            "provider": {
                "id": self.manifest["service_id"],
                "version": self.manifest["service_version"],
            },
            "consumer": {
                "id": consumer.get("service_id").cloned().unwrap_or(Value::Null),
                "version": consumer.get("service_version").cloned().unwrap_or(Value::Null),
            },
            "capabilities": capabilities,
            "timestamp": rfc3339_now(),
            "prev_hash": self.last_hash,
        });
        if let Some(Value::Object(fields)) = extra {
            for (k, v) in fields {
                record_body[k] = v;
            }
        }
        let hash = format!(
            "sha256:{:x}",
            Sha256::digest(serde_json::to_vec(&record_body).expect("record serialises"))
        );
        let mut record = record_body;
        record["hash"] = json!(hash);
        if self.log_attestations {
            println!("{record}");
        }
        self.last_hash = hash;
        self.attestations.push(record);
    }

    /// Reap every due lease at `now` (SPEC §4.6): remove the handle — the
    /// provider holds no other per-consumer state, so removal IS the
    /// zero-residue wipe — and attest `groove:lease-expired` with
    /// `residue: 0`. Hard leases arrive here only after three whole missed
    /// TTL windows (degradation through the same soft path).
    fn sweep_leases(&mut self, now: Instant) {
        let due: Vec<String> = self
            .handles
            .iter()
            .filter(|(_, e)| e.lease.is_some_and(|l| l.due(now)))
            .map(|(h, _)| h.clone())
            .collect();
        for handle in due {
            if let Some(entry) = self.handles.remove(&handle) {
                let lease = entry.lease.expect("swept handles carry a lease");
                self.attest_with(
                    "groove:lease-expired",
                    &json!({ "service_id": entry.consumer_id }),
                    Vec::new(),
                    Some(json!({
                        "residue": 0,
                        "handle": handle,
                        "lease": {
                            "mode": lease.mode.as_str(),
                            "ttl_ms": lease.ttl.as_millis() as u64,
                        },
                    })),
                );
            }
        }
    }
}

/// A running provider. Dropping it does not stop the tasks; hold it for the
/// lifetime you need (tests) or block forever (binary).
pub struct Server {
    port: u16,
    has_v6: bool,
    state: Arc<Mutex<State>>,
}

impl Server {
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Whether the [::1] listener is up. False only on hosts without an IPv6
    /// loopback (CONF-L2-06 cannot be satisfied there; clients fall back to
    /// 127.0.0.1).
    pub fn has_v6(&self) -> bool {
        self.has_v6
    }

    pub fn attestation_count(&self) -> usize {
        self.state.lock().expect("state lock").attestations.len()
    }

    /// Number of live connection handles (leased or legacy).
    pub fn handle_count(&self) -> usize {
        self.state.lock().expect("state lock").handles.len()
    }
}

/// Bind [::1] and 127.0.0.1 on the same port and serve. With `config.port` =
/// 0, an ephemeral port is chosen and claimed on both stacks. On hosts with
/// no IPv6 loopback the provider degrades to 127.0.0.1 only.
pub async fn serve(config: Config) -> Result<Server> {
    let (listeners, port, has_v6) = bind_dual_stack(config.port).await?;

    let mut manifest = config.manifest.unwrap_or_else(|| builtin_manifest(port));
    if let Some(seed) = &config.signing_seed {
        manifest = groove::sign::sign_manifest(&manifest, seed)
            .context("sign manifest (SPEC §2.1.5)")?;
    }
    let state = Arc::new(Mutex::new(State {
        manifest,
        handles: HashMap::new(),
        attestations: Vec::new(),
        last_hash: "sha256:genesis".to_string(),
        handle_counter: 0,
        log_attestations: config.log_attestations,
    }));

    for listener in listeners {
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let state = Arc::clone(&state);
                        tokio::spawn(async move {
                            let _ = handle_connection(stream, state).await;
                        });
                    }
                    Err(_) => break,
                }
            }
        });
    }

    // Lease sweeper (SPEC §4.6): reap due leases on a short interval so
    // soft expiry / hard degradation are observable behaviours, not
    // request-time side effects.
    {
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(SWEEP_INTERVAL);
            loop {
                ticker.tick().await;
                state.lock().expect("state lock").sweep_leases(Instant::now());
            }
        });
    }

    Ok(Server { port, has_v6, state })
}

async fn bind_dual_stack(port: u16) -> Result<(Vec<TcpListener>, u16, bool)> {
    // Is there an IPv6 loopback at all? (Some containers have none.)
    let v6_supported = TcpListener::bind(("::1", 0)).await.is_ok();

    if port != 0 {
        let v4 = TcpListener::bind(("127.0.0.1", port))
            .await
            .with_context(|| format!("bind 127.0.0.1:{port}"))?;
        if v6_supported {
            let v6 = TcpListener::bind(("::1", port))
                .await
                .with_context(|| format!("bind [::1]:{port}"))?;
            return Ok((vec![v6, v4], port, true));
        }
        return Ok((vec![v4], port, false));
    }

    if !v6_supported {
        let v4 = TcpListener::bind(("127.0.0.1", 0)).await.context("bind 127.0.0.1:0")?;
        let p = v4.local_addr()?.port();
        return Ok((vec![v4], p, false));
    }

    // Ephemeral dual-stack: pick on v6, mirror on v4; the pair must share a
    // port number so discovery sees one service (CONF-L2-06).
    for _ in 0..16 {
        let v6 = TcpListener::bind(("::1", 0)).await.context("bind [::1]:0")?;
        let p = v6.local_addr()?.port();
        if let Ok(v4) = TcpListener::bind(("127.0.0.1", p)).await {
            return Ok((vec![v6, v4], p, true));
        }
    }
    anyhow::bail!("could not find a port free on both [::1] and 127.0.0.1")
}

/// Minimal HTTP/1.x exchange: one request, one response, close.
async fn handle_connection(mut stream: TcpStream, state: Arc<Mutex<State>>) -> Result<()> {
    let mut buf = Vec::with_capacity(8192);
    let mut chunk = [0u8; 4096];

    // Read until end of headers.
    let header_end = loop {
        let n = stream.read(&mut chunk).await?;
        if n == 0 {
            return Ok(()); // peer closed
        }
        buf.extend_from_slice(&chunk[..n]);
        if let Some(pos) = find_subsequence(&buf, b"\r\n\r\n") {
            break pos + 4;
        }
        if buf.len() > 65_536 {
            return respond(&mut stream, 400, "text/plain", "headers too large").await;
        }
    };

    let head = String::from_utf8_lossy(&buf[..header_end]).to_string();
    let mut lines = head.lines();
    let request_line = lines.next().unwrap_or_default().to_string();
    let mut accept = String::new();
    let mut content_length = 0usize;
    for line in lines {
        let Some((name, value)) = line.split_once(':') else { continue };
        match name.trim().to_ascii_lowercase().as_str() {
            "accept" => accept = value.trim().to_string(),
            "content-length" => content_length = value.trim().parse().unwrap_or(0),
            _ => {}
        }
    }

    if content_length > 1_048_576 {
        return respond(&mut stream, 400, "text/plain", "body too large").await;
    }

    // Read the body if any.
    let mut body = buf[header_end..].to_vec();
    while body.len() < content_length {
        let n = stream.read(&mut chunk).await?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&chunk[..n]);
    }

    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let full_path = parts.next().unwrap_or_default();
    let (path, query) = full_path.split_once('?').unwrap_or((full_path, ""));

    match (method, path) {
        ("GET", "/.well-known/groove") => {
            let manifest = state.lock().expect("state lock").manifest.clone();
            if prefers_a2ml(&accept) {
                let a2ml = render_a2ml(&manifest);
                respond(&mut stream, 200, "application/groove+a2ml", &a2ml).await
            } else {
                // Absent, unknown, wildcard, or JSON-preferring Accept all get
                // JSON: bare-by-default (SPEC §2.1.3).
                let body = serde_json::to_string_pretty(&manifest)?;
                respond(&mut stream, 200, "application/groove+json", &body).await
            }
        }

        ("POST", "/.well-known/groove/connect") => {
            let Ok(consumer) = serde_json::from_slice::<Value>(&body) else {
                return respond(&mut stream, 400, "application/json", r#"{"error":"body must be a JSON consumer manifest"}"#).await;
            };
            let consumes: Vec<String> = consumer["consumes"]
                .as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();

            // All lock use stays in this sync block (guard must not live
            // across an await).
            let (status, body) = {
                let mut st = state.lock().expect("state lock");
                let offers = st.offers();
                let unmet: Vec<String> = consumes
                    .iter()
                    .filter(|c| !offers.contains(c))
                    .map(|c| {
                        format!(
                            "consumer consumes '{c}' but {} does not offer it",
                            st.manifest["service_id"].as_str().unwrap_or("provider")
                        )
                    })
                    .collect();

                if !unmet.is_empty() {
                    (409, serde_json::to_string(&json!({ "reasons": unmet }))?)
                } else {
                    // Optional lease (SPEC §4.6): absent = legacy §4.3
                    // semantics, unchanged.
                    match parse_lease(&consumer) {
                        Err(reason) => {
                            (400, serde_json::to_string(&json!({ "error": reason }))?)
                        }
                        Ok(lease) => {
                            st.handle_counter += 1;
                            let handle = format!(
                                "grv-{}-{}",
                                st.handle_counter,
                                rfc3339_now().replace([':', '-'], "")
                            );
                            let consumer_id =
                                consumer["service_id"].as_str().unwrap_or("anonymous").to_string();
                            st.handles.insert(
                                handle.clone(),
                                HandleEntry { consumer_id, lease },
                            );
                            st.attest("groove:connected", &consumer, consumes.clone());
                            let provider_id = st.manifest["service_id"].clone();
                            let mut response = json!({
                                "handle": handle,
                                "provider": provider_id,
                            });
                            if let Some(l) = lease {
                                response["lease"] = json!({
                                    "mode": l.mode.as_str(),
                                    "ttl_ms": l.ttl.as_millis() as u64,
                                });
                            }
                            (200, serde_json::to_string(&response)?)
                        }
                    }
                }
            };
            respond(&mut stream, status, "application/json", &body).await
        }

        ("GET", "/.well-known/groove/heartbeat") => {
            // Bare heartbeat: liveness probe, 204 (SPEC §4.3, CONF-L2-03).
            // With ?handle=H (SPEC §4.6): refresh a hard lease; a soft
            // lease MUST be allowed to expire, so its refresh is refused.
            let Some(handle) = query_param(query, "handle") else {
                return respond_no_content(&mut stream).await;
            };
            let outcome = {
                let mut st = state.lock().expect("state lock");
                // Expiry is wall-clock, not sweeper-tick (SPEC §4.6): reap
                // due leases before the lookup so an expired handle is
                // already consumed (404), and a hard lease past three whole
                // missed TTL windows cannot be resurrected by this refresh.
                st.sweep_leases(Instant::now());
                match st.handles.get_mut(&handle) {
                    None => Some((404, r#"{"error":"unknown or already-consumed handle"}"#)),
                    Some(entry) => match &mut entry.lease {
                        None => None, // legacy connection: heartbeat is a no-op probe
                        Some(l) if l.mode == LeaseMode::Hard => {
                            l.expires_at = Instant::now() + l.ttl;
                            None
                        }
                        Some(_) => Some((
                            409,
                            r#"{"error":"soft lease must be allowed to expire; refresh refused"}"#,
                        )),
                    },
                }
            };
            match outcome {
                None => respond_no_content(&mut stream).await,
                Some((status, body)) => respond(&mut stream, status, "application/json", body).await,
            }
        }

        ("POST", "/.well-known/groove/disconnect") => {
            let handle = serde_json::from_slice::<Value>(&body)
                .ok()
                .and_then(|v| v["handle"].as_str().map(String::from))
                .unwrap_or_default();

            let (status, body) = {
                let mut st = state.lock().expect("state lock");
                // Expiry IS consumption (SPEC §4.6): reap due leases first
                // so a disconnect with an expired handle answers 410 and the
                // zero-residue groove:lease-expired attestation is emitted
                // rather than lost.
                st.sweep_leases(Instant::now());
                match st.handles.remove(&handle) {
                    Some(entry) => {
                        // Linear consumption: the handle is gone; a second
                        // disconnect with it gets 410 (CONF-L2-04).
                        st.attest(
                            "groove:disconnected",
                            &json!({ "service_id": entry.consumer_id }),
                            Vec::new(),
                        );
                        (200, r#"{"disconnected":true}"#)
                    }
                    None => (410, r#"{"error":"unknown or already-consumed handle"}"#),
                }
            };
            respond(&mut stream, status, "application/json", body).await
        }

        ("GET", "/.well-known/groove/mesh") => {
            let body = {
                let st = state.lock().expect("state lock");
                let connections: Vec<Value> = st
                    .handles
                    .iter()
                    .map(|(h, entry)| {
                        json!({
                            "handle": h,
                            "consumer": entry.consumer_id,
                            "lease": entry.lease.map(|l| json!({
                                "mode": l.mode.as_str(),
                                "ttl_ms": l.ttl.as_millis() as u64,
                            })),
                        })
                    })
                    .collect();
                serde_json::to_string(&json!({ "connections": connections }))?
            };
            respond(&mut stream, 200, "application/json", &body).await
        }

        ("GET", "/.well-known/groove/attestations") => {
            let body = {
                let st = state.lock().expect("state lock");
                serde_json::to_string(&st.attestations)?
            };
            respond(&mut stream, 200, "application/json", &body).await
        }

        _ => respond(&mut stream, 404, "text/plain", "not found").await,
    }
}

/// Parse the optional `lease` member of a connect body (SPEC §4.6).
/// Absent → Ok(None) (legacy semantics). Present → both `mode` and
/// `ttl_ms` are required, mode ∈ {soft, hard}, TTL within bounds.
fn parse_lease(consumer: &Value) -> std::result::Result<Option<LeaseState>, String> {
    let lease = match consumer.get("lease") {
        None | Some(Value::Null) => return Ok(None),
        Some(l) => l,
    };
    let mode = match lease["mode"].as_str() {
        Some("soft") => LeaseMode::Soft,
        Some("hard") => LeaseMode::Hard,
        Some(other) => return Err(format!("lease.mode must be 'soft' or 'hard', got '{other}'")),
        None => return Err("lease.mode is required when lease is present".to_string()),
    };
    let ttl_ms = lease["ttl_ms"]
        .as_u64()
        .ok_or_else(|| "lease.ttl_ms is required when lease is present".to_string())?;
    let ttl = Duration::from_millis(ttl_ms);
    if !(MIN_TTL..=MAX_TTL).contains(&ttl) {
        return Err(format!(
            "lease.ttl_ms must be between {} and {}",
            MIN_TTL.as_millis(),
            MAX_TTL.as_millis()
        ));
    }
    Ok(Some(LeaseState { mode, ttl, expires_at: Instant::now() + ttl }))
}

/// Extract a query parameter from a raw query string (no percent-decoding:
/// groove handles are URL-safe by construction).
fn query_param(query: &str, name: &str) -> Option<String> {
    query.split('&').find_map(|pair| {
        let (k, v) = pair.split_once('=')?;
        (k == name && !v.is_empty()).then(|| v.to_string())
    })
}

/// Content negotiation (SPEC §2.1.3): serve A2ML only when the Accept header
/// q-prefers it over the JSON encodings. Ties and everything else → JSON
/// (bare-by-default).
fn prefers_a2ml(accept: &str) -> bool {
    let mut q_a2ml = 0.0f32;
    let mut q_json = 0.0f32;
    for entry in accept.split(',') {
        let mut parts = entry.split(';');
        let media = parts.next().unwrap_or("").trim();
        let q: f32 = parts
            .filter_map(|p| p.trim().strip_prefix("q="))
            .next()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1.0);
        match media {
            "application/groove+a2ml" => q_a2ml = q_a2ml.max(q),
            "application/groove+json" | "application/json" | "*/*" | "application/*" => {
                q_json = q_json.max(q)
            }
            _ => {}
        }
    }
    q_a2ml > q_json
}

/// Render the manifest in the optional A2ML @-block encoding (SPEC §2.1.3).
/// Serve-only: nothing in scope parses this yet (ADR 0002).
pub fn render_a2ml(manifest: &Value) -> String {
    let id = manifest["service_id"].as_str().unwrap_or("unknown");
    let version = manifest["service_version"].as_str().unwrap_or("0.0.0");
    let mut out = String::new();
    out.push_str("@groove-manifest(version=\"0.1.0\"):\n");
    out.push_str(&format!(
        "  @system(id=\"{id}\", name=\"{id}\", version=\"{version}\")\n"
    ));
    out.push_str("  @offers:\n");
    if let Some(caps) = manifest["capabilities"].as_object() {
        for (key, cap) in caps {
            let cap_type = cap["type"].as_str().unwrap_or("custom");
            let cap_version = cap["version"].as_str().unwrap_or("1.0.0");
            out.push_str(&format!(
                "    @capability(id=\"{key}\", type=\"{cap_type}\", version=\"{cap_version}\")\n"
            ));
        }
    }
    out.push_str("  @end\n");
    if let Some(consumes) = manifest["consumes"].as_array() {
        if !consumes.is_empty() {
            out.push_str("  @consumes:\n");
            for c in consumes.iter().filter_map(|v| v.as_str()) {
                out.push_str(&format!("    @capability(id=\"{c}\")\n"));
            }
            out.push_str("  @end\n");
        }
    }
    out.push_str("@end\n");
    out
}

async fn respond(stream: &mut TcpStream, status: u16, content_type: &str, body: &str) -> Result<()> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        409 => "Conflict",
        410 => "Gone",
        _ => "Unknown",
    };
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes()).await?;
    stream.shutdown().await.ok();
    Ok(())
}

async fn respond_no_content(stream: &mut TcpStream) -> Result<()> {
    stream
        .write_all(b"HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n")
        .await?;
    stream.shutdown().await.ok();
    Ok(())
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}
