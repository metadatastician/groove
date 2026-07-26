// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Signed-manifest integration (SPEC §2.1.5, ADR 0010): a provider started
// with a signing seed serves a manifest whose detached signature verifies —
// including after the JSON has been re-parsed (canonicalisation, not bytes).

use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use groove::sign::{public_key_b64, verify_manifest, VerifyOutcome};
use groove_provider::{serve, Config};

const SEED: [u8; 32] = [42u8; 32];

#[tokio::test]
async fn signed_manifest_verifies_and_tampering_fails() {
    let s = serve(Config {
        port: 0,
        manifest: None,
        log_attestations: false,
        signing_seed: Some(SEED),
    })
    .await
    .expect("signed provider starts");

    let addr = if s.has_v6() {
        format!("[::1]:{}", s.port())
    } else {
        format!("127.0.0.1:{}", s.port())
    };
    let req = "GET /.well-known/groove HTTP/1.1\r\nHost: localhost\r\nAccept: application/groove+json\r\nConnection: close\r\n\r\n";
    let mut stream = TcpStream::connect(&addr).await.expect("connect");
    stream.write_all(req.as_bytes()).await.expect("write");
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await.expect("read");
    let text = String::from_utf8_lossy(&response);
    let body = text.split_once("\r\n\r\n").expect("has body").1;

    let manifest: Value = serde_json::from_str(body).expect("manifest JSON");
    assert_eq!(manifest["signature"]["alg"], "ed25519");

    // Self-consistent, and valid against the correct pin.
    assert_eq!(verify_manifest(&manifest, None).unwrap(), VerifyOutcome::ValidSelf);
    let pin = public_key_b64(&SEED);
    assert_eq!(verify_manifest(&manifest, Some(&pin)).unwrap(), VerifyOutcome::ValidPinned);

    // A spoofed capability breaks the signature; a wrong pin is rejected.
    let mut spoofed = manifest.clone();
    spoofed["capabilities"]["voice"] = serde_json::json!({ "type": "voice" });
    assert!(verify_manifest(&spoofed, None).is_err(), "spoofing must break the signature");
    let wrong_pin = public_key_b64(&[1u8; 32]);
    assert!(verify_manifest(&manifest, Some(&wrong_pin)).is_err(), "pin mismatch must fail");
}
