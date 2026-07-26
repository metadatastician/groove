// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Truth-table tests for bidirectional capability compatibility.

use groove::registry::{check_compat_entries, ServiceEntry};

fn entry(id: &str, offers: &[&str], consumes: &[&str]) -> ServiceEntry {
    ServiceEntry {
        id: id.to_string(),
        port: 0,
        offers: offers.iter().map(|s| s.to_string()).collect(),
        consumes: consumes.iter().map(|s| s.to_string()).collect(),
        description: String::new(),
        status: "active".to_string(),
        notes: None,
        public_key: None,
    }
}

#[test]
fn mutually_satisfied_pair_is_compatible() {
    let a = entry("a", &["voice"], &["integrity"]);
    let b = entry("b", &["integrity"], &["voice"]);
    let result = check_compat_entries(&a, &b);
    assert!(result.compatible);
    assert_eq!(result.matched.len(), 2);
    assert!(result.reasons.is_empty());
}

#[test]
fn one_direction_only_is_incompatible_with_one_reason() {
    let a = entry("a", &[], &["integrity"]); // a needs integrity, offers nothing
    let b = entry("b", &["integrity"], &["voice"]); // b needs voice — unmet
    let result = check_compat_entries(&a, &b);
    assert!(!result.compatible);
    assert_eq!(result.reasons.len(), 1);
    assert!(result.reasons[0].contains("voice"));
    // The satisfied direction still reports its match.
    assert_eq!(result.matched.len(), 1);
}

#[test]
fn disjoint_pair_lists_every_unmet_consume() {
    let a = entry("a", &["text"], &["octad-storage", "scanning"]);
    let b = entry("b", &["voice"], &["theorem-proving"]);
    let result = check_compat_entries(&a, &b);
    assert!(!result.compatible);
    assert_eq!(result.reasons.len(), 3, "{:?}", result.reasons);
    assert!(result.matched.is_empty());
}

#[test]
fn no_consumes_on_either_side_is_trivially_compatible() {
    let a = entry("a", &["voice"], &[]);
    let b = entry("b", &[], &[]);
    let result = check_compat_entries(&a, &b);
    assert!(result.compatible);
    assert!(result.matched.is_empty());
}
