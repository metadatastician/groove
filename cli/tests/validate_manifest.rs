// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Golden tests for manifest validation against examples/minimal-manifest.json.

use std::fs;
use std::path::Path;

use groove::sign::sign_manifest;
use groove::validate::{Finding, validate_manifest_content, verify_signature_finding};

fn minimal_manifest() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli/ has a parent")
        .join("examples/minimal-manifest.json");
    fs::read_to_string(path).expect("examples/minimal-manifest.json exists")
}

#[test]
fn minimal_manifest_validates_clean() {
    let findings = validate_manifest_content(&minimal_manifest(), "examples/minimal-manifest.json");
    assert!(
        findings.is_empty(),
        "the shipped minimal manifest must validate clean, got: {findings:#?}"
    );
}

#[test]
fn unknown_capability_type_is_flagged() {
    let mutated = minimal_manifest().replace("\"attestation\"", "\"notacap\"");
    let findings = validate_manifest_content(&mutated, "mutated.json");
    assert!(
        findings.iter().any(|f| f.description.contains("notacap")),
        "an unregistered capability type must produce a finding, got: {findings:#?}"
    );
}

#[test]
fn missing_groove_version_is_flagged() {
    let mutated = minimal_manifest().replace("\"groove_version\": \"1\",", "");
    let findings = validate_manifest_content(&mutated, "mutated.json");
    assert!(
        findings
            .iter()
            .any(|f| f.description.contains("groove_version")),
        "a missing groove_version must produce a finding, got: {findings:#?}"
    );
}

#[test]
fn invalid_json_is_critical() {
    let findings = validate_manifest_content("{ not json", "broken.json");
    assert!(findings.iter().any(|f| f.severity == "critical"));
}

#[test]
fn capabilities_array_is_schema_violation() {
    let mutated = minimal_manifest().replace(
        r#""capabilities": {
    "attestation": {
      "type": "attestation",
      "protocol": "http",
      "version": "1.0.0"
    }
  }"#,
        r#""capabilities": []"#,
    );
    let findings = validate_manifest_content(&mutated, "mutated.json");
    assert!(
        findings.iter().any(|f| f.description.contains("object")),
        "capabilities-as-array must be flagged as a schema violation, got: {findings:#?}"
    );
}

/// The minimal manifest signed with a fixed test seed, as JSON text.
fn signed_minimal_manifest() -> String {
    let manifest: serde_json::Value =
        serde_json::from_str(&minimal_manifest()).expect("minimal manifest is JSON");
    let signed = sign_manifest(&manifest, &[7u8; 32]).expect("minimal manifest signs");
    serde_json::to_string(&signed).expect("signed manifest serialises")
}

/// The `DOG-03-SIG` findings that are `critical`.
fn critical_sig_findings(findings: &[Finding]) -> Vec<&Finding> {
    findings
        .iter()
        .filter(|f| f.check == "DOG-03-SIG" && f.severity == "critical")
        .collect()
}

/// Positive control for the refusals below: the same manifest, unmutated,
/// verifies under `--verify` and raises nothing critical.
#[test]
fn verify_accepts_a_validly_signed_manifest() {
    let findings = verify_signature_finding(&signed_minimal_manifest(), "signed.json");
    assert!(
        critical_sig_findings(&findings).is_empty(),
        "a validly signed manifest must verify, got: {findings:#?}"
    );
    assert!(
        findings.iter().any(|f| f.description.contains("verifies")),
        "expected a verification note, got: {findings:#?}"
    );
}

/// A repeated key is refused at verify (RFC 7493 §2.3). A last-wins parser
/// keeps the second `service_id`, so the signature verifies, while a
/// first-wins consumer reads "spoofed" under the same signature.
#[test]
fn verify_refuses_a_repeated_key_under_a_valid_signature() {
    let signed = signed_minimal_manifest();
    let mutated = signed.replacen('{', r#"{"service_id":"spoofed","#, 1);
    let last_wins: serde_json::Value = serde_json::from_str(&mutated).unwrap();
    assert_eq!(
        last_wins["service_id"], "groove-ref",
        "the planted duplicate must be shadowed"
    );

    let findings = verify_signature_finding(&mutated, "dup.json");
    let critical = critical_sig_findings(&findings);
    assert_eq!(
        critical.len(),
        1,
        "expected exactly one critical finding, got: {findings:#?}"
    );
    assert!(
        critical[0].description.contains("I-JSON"),
        "got: {findings:#?}"
    );
}

/// An integer outside ±(2^53−1) is not I-JSON (RFC 7493 §2.2) and is
/// critical under `--verify`.
#[test]
fn verify_refuses_an_integer_beyond_2_pow_53() {
    let mutated = minimal_manifest().replacen('{', r#"{"ttl":9007199254740993,"#, 1);
    let findings = verify_signature_finding(&mutated, "big.json");
    assert_eq!(
        critical_sig_findings(&findings).len(),
        1,
        "an integer outside ±(2^53−1) must be critical under --verify, got: {findings:#?}"
    );
}

/// Unparseable JSON stays Check 1's finding; verify adds nothing.
#[test]
fn verify_leaves_invalid_json_to_check_1() {
    assert!(verify_signature_finding("{ not json", "broken.json").is_empty());
}

/// An unsigned manifest is parsed Strict too: a repeated `service_id` would
/// otherwise pick which registry pin applies, so it is critical, not the
/// `low` "unsigned" note.
#[test]
fn verify_refuses_a_repeated_key_in_an_unsigned_manifest() {
    let mutated = minimal_manifest().replacen('{', r#"{"service_id":"spoofed","#, 1);
    let findings = verify_signature_finding(&mutated, "dup-unsigned.json");
    let critical = critical_sig_findings(&findings);
    assert_eq!(critical.len(), 1, "got: {findings:#?}");
    assert!(critical[0].description.contains("I-JSON"));
    assert!(
        !findings.iter().any(|f| f.description.contains("unsigned")),
        "a non-I-JSON manifest must not be reported as merely unsigned: {findings:#?}"
    );
}
