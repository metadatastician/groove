// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// The executable Groove conformance suite: exactly one test per stable ID in
// spec/CONFORMANCE.adoc, run against a live reference provider on ephemeral
// ports. CI enforces the ID↔test mapping (spec-consistency job).

use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use groove_provider::{serve, Config, Server};

async fn start() -> Server {
    serve(Config {
        port: 0,
        manifest: None,
        log_attestations: false,
        signing_seed: None,
    })
    .await
    .expect("provider starts on ephemeral dual-stack port")
}

/// Loopback address for requests: [::1] where available (clients probe it
/// first per TRANSPORT §7.6), else 127.0.0.1 (v6-less environments).
fn loopback(s: &Server) -> String {
    if s.has_v6() {
        format!("[::1]:{}", s.port())
    } else {
        format!("127.0.0.1:{}", s.port())
    }
}

/// Minimal HTTP/1.1 client: returns (status, headers, body).
async fn http(addr: &str, request: &str) -> (u16, String, String) {
    let mut stream = TcpStream::connect(addr).await.expect("connect");
    stream.write_all(request.as_bytes()).await.expect("write");
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await.expect("read");
    let text = String::from_utf8_lossy(&response).to_string();
    let (head, body) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
    let status: u16 = head
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    (status, head.to_string(), body.to_string())
}

async fn get(s: &Server, path: &str, accept: &str) -> (u16, String, String) {
    let req = format!(
        "GET {path} HTTP/1.1\r\nHost: localhost\r\nAccept: {accept}\r\nConnection: close\r\n\r\n"
    );
    http(&loopback(s), &req).await
}

async fn post(s: &Server, path: &str, body: &Value) -> (u16, String, String) {
    let payload = body.to_string();
    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
        payload.len()
    );
    http(&loopback(s), &req).await
}

fn consumer(consumes: &[&str]) -> Value {
    json!({
        "groove_version": "1",
        "service_id": "conformance-consumer",
        "service_version": "0.0.1",
        "mode": "active",
        "capabilities": {},
        "consumes": consumes,
    })
}

async fn connect_ok(s: &Server) -> String {
    let (status, _, body) = post(s, "/.well-known/groove/connect", &consumer(&[])).await;
    assert_eq!(status, 200, "compatible connect must succeed: {body}");
    serde_json::from_str::<Value>(&body).expect("connect body is JSON")["handle"]
        .as_str()
        .expect("connect body carries a handle")
        .to_string()
}

// ---------------------------------------------------------------- Level 1

#[tokio::test]
async fn conf_l1_01() {
    let s = start().await;
    let (status, head, _) = get(&s, "/.well-known/groove", "*/*").await;
    assert_eq!(status, 200);
    assert!(
        head.to_ascii_lowercase().contains("content-type: application/groove+json"),
        "default content type must be application/groove+json, got:\n{head}"
    );
}

#[tokio::test]
async fn conf_l1_02() {
    let s = start().await;
    let (_, _, body) = get(&s, "/.well-known/groove", "application/groove+json").await;
    let m: Value = serde_json::from_str(&body).expect("manifest is JSON");
    assert_eq!(m["groove_version"], "1");
    assert!(m["service_id"].is_string());
    assert!(m["service_version"].is_string());
    assert!(m["mode"].is_string());
    assert!(m["capabilities"].is_object(), "capabilities is an object (map)");
}

#[tokio::test]
async fn conf_l1_03() {
    let s = start().await;
    let (_, _, body) = get(&s, "/.well-known/groove", "application/groove+json").await;
    let m: Value = serde_json::from_str(&body).expect("manifest is JSON");
    let caps = m["capabilities"].as_object().expect("capabilities object");
    assert!(!caps.is_empty(), "must offer at least one capability");
    let valid = caps
        .values()
        .filter_map(|c| c["type"].as_str())
        .any(groove::registry::is_valid_capability);
    assert!(valid, "at least one capability type must be registered");
}

#[tokio::test]
async fn conf_l1_04() {
    let s = start().await;

    let (status, head, body) = get(&s, "/.well-known/groove", "application/groove+a2ml").await;
    assert_eq!(status, 200);
    assert!(head.to_ascii_lowercase().contains("application/groove+a2ml"));
    assert!(body.trim_start().starts_with("@groove-manifest"), "A2ML body: {body}");

    // Unknown Accept still gets a parseable JSON manifest (bare-by-default).
    let (status, _, body) = get(&s, "/.well-known/groove", "text/weird").await;
    assert_eq!(status, 200);
    let m: Value = serde_json::from_str(&body).expect("fallback body is JSON");
    assert_eq!(m["groove_version"], "1");
}

// ---------------------------------------------------------------- Level 2

#[tokio::test]
async fn conf_l2_01() {
    let s = start().await;
    // groove-ref offers "attestation"; consuming it is structurally compatible.
    let (status, _, body) =
        post(&s, "/.well-known/groove/connect", &consumer(&["attestation"])).await;
    assert_eq!(status, 200, "{body}");
    let v: Value = serde_json::from_str(&body).expect("JSON body");
    assert!(v["handle"].is_string(), "connect returns a handle: {body}");
}

#[tokio::test]
async fn conf_l2_02() {
    let s = start().await;
    let (status, _, body) =
        post(&s, "/.well-known/groove/connect", &consumer(&["voice"])).await;
    assert_eq!(status, 409, "incompatible connect must 409: {body}");
    let v: Value = serde_json::from_str(&body).expect("JSON body");
    assert!(
        v["reasons"].as_array().is_some_and(|r| !r.is_empty()),
        "409 carries machine-readable reasons: {body}"
    );
    // No handle was minted.
    let (_, _, mesh) = get(&s, "/.well-known/groove/mesh", "application/json").await;
    let m: Value = serde_json::from_str(&mesh).expect("mesh JSON");
    assert_eq!(m["connections"].as_array().map(Vec::len), Some(0));
}

#[tokio::test]
async fn conf_l2_03() {
    let s = start().await;
    let started = Instant::now();
    let (status, _, _) = get(&s, "/.well-known/groove/heartbeat", "*/*").await;
    let elapsed = started.elapsed();
    assert_eq!(status, 204);
    assert!(
        elapsed < Duration::from_millis(500),
        "heartbeat took {elapsed:?}, must be < 500ms"
    );
}

#[tokio::test]
async fn conf_l2_04() {
    let s = start().await;
    let handle = connect_ok(&s).await;

    let (status, _, _) =
        post(&s, "/.well-known/groove/disconnect", &json!({ "handle": handle })).await;
    assert_eq!(status, 200, "first disconnect succeeds");

    let (status, _, _) =
        post(&s, "/.well-known/groove/disconnect", &json!({ "handle": handle })).await;
    assert_eq!(status, 410, "handle is linear: second disconnect gets 410 Gone");
}

#[tokio::test]
async fn conf_l2_05() {
    let s = start().await;
    let handle = connect_ok(&s).await;
    post(&s, "/.well-known/groove/disconnect", &json!({ "handle": handle })).await;

    let (status, _, body) = get(&s, "/.well-known/groove", "application/groove+json").await;
    assert_eq!(status, 200, "provider keeps functioning bare after disconnect");
    let m: Value = serde_json::from_str(&body).expect("manifest still parses");
    assert_eq!(m["groove_version"], "1");
}

#[tokio::test]
async fn conf_l2_06() {
    let s = start().await;
    let mut hosts = vec![format!("127.0.0.1:{}", s.port())];
    if s.has_v6() {
        // [::1] first, per TRANSPORT §7.6.
        hosts.insert(0, format!("[::1]:{}", s.port()));
    } else {
        // Full dual-stack verification requires an IPv6 loopback; CI has one.
        eprintln!("conf_l2_06: no IPv6 loopback in this environment — verifying 127.0.0.1 only");
    }
    for host in hosts {
        let req = "GET /.well-known/groove HTTP/1.1\r\nHost: localhost\r\nAccept: application/groove+json\r\nConnection: close\r\n\r\n";
        let (status, _, body) = http(&host, req).await;
        assert_eq!(status, 200, "manifest endpoint must answer on {host}");
        let m: Value = serde_json::from_str(&body).expect("JSON on both stacks");
        assert_eq!(m["service_id"], "groove-ref");
    }
}

// ------------------------------------------- Level 2 lease extension (§4.6)
// Lease modes (SPEC §4.6): soft expires to a zero-residue wipe; hard
// refreshes on heartbeat and degrades through the soft path when the
// heartbeat stops. The wire-observable shadow of cleave RC-6.

/// A consumer manifest carrying a lease request.
fn leased_consumer(mode: &str, ttl_ms: u64) -> Value {
    let mut c = consumer(&[]);
    c["lease"] = json!({ "mode": mode, "ttl_ms": ttl_ms });
    c
}

async fn attestation_events(s: &Server) -> Vec<Value> {
    let (_, _, body) = get(s, "/.well-known/groove/attestations", "application/json").await;
    serde_json::from_str(&body).expect("attestations JSON")
}

#[tokio::test]
async fn conf_l2_08() {
    let s = start().await;
    let (status, _, body) =
        post(&s, "/.well-known/groove/connect", &leased_consumer("soft", 60_000)).await;
    assert_eq!(status, 200, "{body}");
    let v: Value = serde_json::from_str(&body).expect("JSON body");
    assert!(v["handle"].is_string(), "leased connect returns a handle: {body}");
    assert_eq!(v["lease"]["mode"], "soft", "accepted lease is echoed: {body}");
    assert_eq!(v["lease"]["ttl_ms"], 60_000);

    // A malformed lease is rejected, not silently ignored.
    let (status, _, body) =
        post(&s, "/.well-known/groove/connect", &leased_consumer("squishy", 1000)).await;
    assert_eq!(status, 400, "unknown lease mode must 400: {body}");
}

#[tokio::test]
async fn conf_l2_09() {
    let s = start().await;
    let (status, _, body) =
        post(&s, "/.well-known/groove/connect", &leased_consumer("soft", 200)).await;
    assert_eq!(status, 200, "{body}");
    let handle = serde_json::from_str::<Value>(&body).expect("JSON")["handle"]
        .as_str()
        .expect("handle")
        .to_string();

    // A soft lease's refresh is refused: soft MUST be allowed to expire.
    let (status, _, body) =
        get(&s, &format!("/.well-known/groove/heartbeat?handle={handle}"), "*/*").await;
    assert_eq!(status, 409, "soft refresh must be refused: {body}");

    // Past TTL (plus sweep slack) the handle is expired: linear 410.
    tokio::time::sleep(Duration::from_millis(450)).await;
    let (status, _, _) =
        post(&s, "/.well-known/groove/disconnect", &json!({ "handle": handle })).await;
    assert_eq!(status, 410, "expired soft handle answers 410 Gone");

    // The expiry is attested as a zero-residue wipe.
    let records = attestation_events(&s).await;
    let expiry = records
        .iter()
        .find(|r| r["event"] == "groove:lease-expired")
        .unwrap_or_else(|| panic!("no lease-expired attestation in {records:?}"));
    assert_eq!(expiry["residue"], 0, "expiry must attest residue 0: {expiry}");
    assert_eq!(expiry["lease"]["mode"], "soft");
}

#[tokio::test]
async fn conf_l2_10() {
    let s = start().await;
    let (status, _, body) =
        post(&s, "/.well-known/groove/connect", &leased_consumer("hard", 200)).await;
    assert_eq!(status, 200, "{body}");
    let handle = serde_json::from_str::<Value>(&body).expect("JSON")["handle"]
        .as_str()
        .expect("handle")
        .to_string();

    // Heartbeat every half-TTL across more than three TTL windows: a lease
    // being renewed is never reaped.
    for _ in 0..7 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let (status, _, body) =
            get(&s, &format!("/.well-known/groove/heartbeat?handle={handle}"), "*/*").await;
        assert_eq!(status, 204, "hard heartbeat refreshes: {body}");
    }

    // Still connected: graceful disconnect succeeds.
    let (status, _, _) =
        post(&s, "/.well-known/groove/disconnect", &json!({ "handle": handle })).await;
    assert_eq!(status, 200, "hard lease survived ≥3 TTL windows of heartbeats");
}

#[tokio::test]
async fn conf_l2_11() {
    let s = start().await;
    let (_, _, body) =
        post(&s, "/.well-known/groove/connect", &leased_consumer("hard", 100)).await;
    let handle = serde_json::from_str::<Value>(&body).expect("JSON")["handle"]
        .as_str()
        .expect("handle")
        .to_string();

    // Within the grace windows the hard lease survives unheartbeaten...
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert_eq!(s.handle_count(), 1, "hard lease degrades only after 3 whole missed windows");

    // ...after three whole missed TTL windows it degrades through the
    // soft-expiry path.
    tokio::time::sleep(Duration::from_millis(300)).await;
    let (status, _, _) =
        post(&s, "/.well-known/groove/disconnect", &json!({ "handle": handle })).await;
    assert_eq!(status, 410, "degraded hard handle answers 410 Gone");

    let records = attestation_events(&s).await;
    let expiry = records
        .iter()
        .find(|r| r["event"] == "groove:lease-expired")
        .unwrap_or_else(|| panic!("no lease-expired attestation in {records:?}"));
    assert_eq!(expiry["residue"], 0);
    assert_eq!(expiry["lease"]["mode"], "hard");
}

#[tokio::test]
async fn conf_l2_12() {
    let s = start().await;

    // connect (attested) → expiry (attested) → connect → disconnect
    // (attested): the chain must stay unbroken across lease events.
    let (_, _, body) =
        post(&s, "/.well-known/groove/connect", &leased_consumer("soft", 150)).await;
    let _expired_handle = serde_json::from_str::<Value>(&body).expect("JSON")["handle"].clone();
    tokio::time::sleep(Duration::from_millis(350)).await;

    let handle = connect_ok(&s).await;
    post(&s, "/.well-known/groove/disconnect", &json!({ "handle": handle })).await;

    let records = attestation_events(&s).await;
    let events: Vec<&str> = records.iter().filter_map(|r| r["event"].as_str()).collect();
    assert!(events.contains(&"groove:lease-expired"), "chain carries the expiry: {events:?}");
    assert!(events.contains(&"groove:disconnected"));
    assert_eq!(records[0]["prev_hash"], "sha256:genesis");
    for pair in records.windows(2) {
        assert_eq!(
            pair[1]["prev_hash"], pair[0]["hash"],
            "lease events must hash-chain with §5.1 records: {} then {}",
            pair[0], pair[1]
        );
    }
}

#[tokio::test]
async fn conf_l2_07() {
    let s = start().await;
    let handle = connect_ok(&s).await;
    post(&s, "/.well-known/groove/disconnect", &json!({ "handle": handle })).await;

    let (status, _, body) = get(&s, "/.well-known/groove/attestations", "application/json").await;
    assert_eq!(status, 200);
    let records: Vec<Value> = serde_json::from_str(&body).expect("attestations JSON");
    assert!(records.len() >= 2, "connect + disconnect each attest");

    let events: Vec<&str> = records.iter().filter_map(|r| r["event"].as_str()).collect();
    assert!(events.contains(&"groove:connected"));
    assert!(events.contains(&"groove:disconnected"));

    for r in &records {
        for field in ["event", "provider", "consumer", "timestamp", "hash", "prev_hash"] {
            assert!(!r[field].is_null(), "record missing {field}: {r}");
        }
    }
    assert_eq!(records[0]["prev_hash"], "sha256:genesis");
    for pair in records.windows(2) {
        assert_eq!(
            pair[1]["prev_hash"], pair[0]["hash"],
            "records must hash-chain: {} then {}",
            pair[0], pair[1]
        );
    }
}
