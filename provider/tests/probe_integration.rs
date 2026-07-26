// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// End-to-end: the CLI's probe client discovers the reference provider.

use std::time::Duration;

use serde_json::Value;

#[tokio::test]
async fn probe_discovers_reference_provider() {
    let server = groove_provider::serve(groove_provider::Config {
        port: 0,
        manifest: None,
        log_attestations: false,
        signing_seed: None,
    })
    .await
    .expect("provider starts");

    let mut hosts = vec!["127.0.0.1"];
    if server.has_v6() {
        hosts.insert(0, "::1"); // [::1] first, per TRANSPORT §7.6
    } else {
        eprintln!("probe_integration: no IPv6 loopback here — probing 127.0.0.1 only");
    }
    for host in hosts {
        let addr = if host.contains(':') {
            format!("[{host}]:{}", server.port())
        } else {
            format!("{host}:{}", server.port())
        };
        let manifest_json = groove::probe::probe_groove(&addr, Duration::from_millis(1000))
            .await
            .expect("probe I/O succeeds")
            .expect("provider answers with a manifest body");

        let manifest: Value = serde_json::from_str(&manifest_json).expect("manifest is JSON");
        assert_eq!(manifest["service_id"], "groove-ref", "on {addr}");
        assert_eq!(manifest["groove_version"], "1");
    }
}
