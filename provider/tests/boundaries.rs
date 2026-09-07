// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell

//! Live transport regressions: malformed framing must not mint authority;
//! stopping a provider must release the socket it owns.

use groove_provider::{Config, MAX_ACTIVE_HANDLES, REQUEST_TIMEOUT, Server, serve};

#[test]
fn config_debug_redacts_the_signing_seed() {
    let seed = [42u8; 32];
    let config = Config {
        signing_seed: Some(seed),
        ..Config::default()
    };
    let debug = format!("{config:?}");
    assert!(debug.contains("[REDACTED]"));
    assert!(!debug.contains(&format!("{seed:?}")));
}
use serde_json::{Value, json};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

async fn start() -> Server {
    serve(Config {
        port: 0,
        ..Config::default()
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn retained_session_metadata_is_bounded_before_minting() {
    let server = start().await;
    for (key, value) in [
        ("service_id", json!("a".repeat(129))),
        ("service_version", json!("1".repeat(65))),
        ("consumes", json!(vec!["attestation"; 65])),
    ] {
        let mut manifest: Value = serde_json::from_str(&consumer()).unwrap();
        manifest[key] = value;
        let body = manifest.to_string();
        let wire = format!(
            "POST /.well-known/groove/connect HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        assert_eq!(request(&server, &wire).await.0, 400);
        assert_eq!(server.handle_count(), 0);
    }
    let body = consumer();
    let wire = format!(
        "POST /.well-known/groove/connect HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );
    assert_eq!(request(&server, &wire).await.0, 200);
    assert_eq!(server.handle_count(), 1);
}

async fn request(server: &Server, request: &str) -> (u16, String) {
    let mut stream = TcpStream::connect(("127.0.0.1", server.port()))
        .await
        .unwrap();
    stream.write_all(request.as_bytes()).await.unwrap();
    stream.shutdown().await.unwrap();
    let mut response = String::new();
    tokio::time::timeout(Duration::from_secs(2), stream.read_to_string(&mut response))
        .await
        .expect("bounded response")
        .unwrap();
    let (head, body) = response.split_once("\r\n\r\n").expect("HTTP response");
    (
        head.split_whitespace().nth(1).unwrap().parse().unwrap(),
        body.into(),
    )
}

fn consumer() -> String {
    json!({"groove_version":"1", "service_id":"boundary-consumer",
        "service_version":"0.1.0", "mode":"active", "capabilities":{},
        "consumes":[]})
    .to_string()
}

#[tokio::test]
async fn rejects_invalid_and_incomplete_request_framing() {
    let server = start().await;
    let body = consumer();
    let malformed = [
        "Content-Length: not-a-number\r\n".to_string(),
        format!("Content-Length: {}\r\n", body.len() + 1),
        format!("Content-Length: 0\r\nContent-Length: {}\r\n", body.len()),
        format!(
            "Transfer-Encoding: chunked\r\nContent-Length: {}\r\n",
            body.len()
        ),
    ];
    // Positive control: this exact payload is valid with correct framing.
    let valid = format!(
        "POST /.well-known/groove/connect HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    assert_eq!(request(&server, &valid).await.0, 200);
    for framing in malformed {
        let wire = format!(
            "POST /.well-known/groove/connect HTTP/1.1\r\nHost: localhost\r\n{framing}\r\n{body}"
        );
        assert_eq!(request(&server, &wire).await.0, 400, "{framing}");
        assert_eq!(
            server.handle_count(),
            1,
            "bad framing must not mint a handle"
        );
    }
}

#[tokio::test]
async fn rejects_non_manifest_json_before_minting() {
    let server = start().await;
    for body in [
        "null",
        "[]",
        "42",
        "{}",
        r#"{"service_id":"x","consumes":[42]}"#,
    ] {
        let wire = format!(
            "POST /.well-known/groove/connect HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        );
        assert_eq!(
            request(&server, &wire).await.0,
            400,
            "invalid manifest: {body}"
        );
        assert_eq!(server.handle_count(), 0);
    }
}

#[tokio::test]
async fn discovery_metadata_does_not_disclose_live_bearer_handles() {
    let server = start().await;
    let body = consumer();
    let wire = format!(
        "POST /.well-known/groove/connect HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    let (status, body) = request(&server, &wire).await;
    assert_eq!(status, 200);
    let connected: Value = serde_json::from_str(&body).unwrap();
    let handle = connected["handle"].as_str().unwrap();
    for endpoint in ["mesh", "attestations"] {
        let wire =
            format!("GET /.well-known/groove/{endpoint} HTTP/1.1\r\nHost: localhost\r\n\r\n");
        let (status, body) = request(&server, &wire).await;
        assert_eq!(status, 200);
        assert!(
            !body.contains(handle),
            "public {endpoint} must not reveal bearer authority"
        );
    }
}

#[tokio::test]
async fn dropping_server_releases_its_listener() {
    let server = start().await;
    let port = server.port();
    assert!(TcpStream::connect(("127.0.0.1", port)).await.is_ok());
    drop(server);
    // Allow cancellation to be processed by the runtime.
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert!(
        TcpStream::connect(("127.0.0.1", port)).await.is_err(),
        "a dropped provider must stop accepting requests"
    );
}

#[tokio::test]
async fn rejects_cross_origin_and_dns_rebinding_requests() {
    let server = start().await;
    let body = consumer();
    for (headers, expected) in [
        ("Host: attacker.example\r\n", 400),
        (
            "Host: localhost\r\nOrigin: https://attacker.example\r\n",
            403,
        ),
        ("Host: localhost\r\nOrigin: null\r\n", 403),
    ] {
        let wire = format!(
            "POST /.well-known/groove/connect HTTP/1.1\r\n{headers}Content-Length: {}\r\n\r\n{body}",
            body.len()
        );
        assert_eq!(request(&server, &wire).await.0, expected);
        assert_eq!(server.handle_count(), 0);
    }
}

#[tokio::test]
async fn rejects_malformed_consumes_in_an_otherwise_valid_manifest() {
    let server = start().await;
    for invalid in [json!([42]), json!({"voice":true}), Value::Null] {
        let mut manifest: Value = serde_json::from_str(&consumer()).unwrap();
        manifest["consumes"] = invalid;
        let body = manifest.to_string();
        let wire = format!(
            "POST /.well-known/groove/connect HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        );
        assert_eq!(request(&server, &wire).await.0, 400);
    }
    assert_eq!(server.handle_count(), 0);
}

#[tokio::test]
async fn session_capacity_recovers_after_disconnect() {
    let server = start().await;
    let body = consumer();
    let wire = format!(
        "POST /.well-known/groove/connect HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    let mut handles = std::collections::HashSet::new();
    for _ in 0..MAX_ACTIVE_HANDLES {
        let (status, body) = request(&server, &wire).await;
        assert_eq!(status, 200);
        let value: Value = serde_json::from_str(&body).unwrap();
        let handle = value["handle"].as_str().unwrap().to_string();
        assert_eq!(handle.len(), 68, "grv- plus 256 bits encoded as hex");
        assert!(
            handles.insert(handle),
            "independent mints must not reuse a handle"
        );
    }
    assert_eq!(request(&server, &wire).await.0, 503);
    let body = json!({"handle": handles.iter().next().unwrap()}).to_string();
    let disconnect = format!(
        "POST /.well-known/groove/disconnect HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    assert_eq!(request(&server, &disconnect).await.0, 200);
    assert_eq!(request(&server, &wire).await.0, 200);
    assert_eq!(server.handle_count(), MAX_ACTIVE_HANDLES);
    server.shutdown().await;
}

#[tokio::test]
async fn incomplete_headers_cannot_hold_a_socket_forever() {
    let server = start().await;
    let mut stream = TcpStream::connect(("127.0.0.1", server.port()))
        .await
        .unwrap();
    stream
        .write_all(b"GET /.well-known/groove HTTP/1.1\r\n")
        .await
        .unwrap();
    let mut byte = [0];
    let read = tokio::time::timeout(
        REQUEST_TIMEOUT + Duration::from_secs(2),
        stream.read(&mut byte),
    )
    .await
    .expect("provider closed the stalled request within its deadline");
    assert!(read.is_err() || read.unwrap() == 0);
    assert_eq!(server.handle_count(), 0);
    server.shutdown().await;
}
