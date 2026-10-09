// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Golden tests for manifest validation against examples/minimal-manifest.json.

use std::fs;
use std::path::Path;

use groove::probe::parse_probed_manifest;
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

/// The `DOG-03` findings that are `critical`.
fn critical_check1_findings(findings: &[Finding]) -> Vec<&Finding> {
    findings
        .iter()
        .filter(|f| f.check == "DOG-03" && f.severity == "critical")
        .collect()
}

/// Every `critical` finding, whichever check raised it.
fn all_critical(findings: &[Finding]) -> Vec<&Finding> {
    findings
        .iter()
        .filter(|f| f.severity == "critical")
        .collect()
}

/// A repeated key is refused at Check 1 (RFC 7493 §2.3), without
/// `--verify`. A last-wins parser keeps the second `service_id`, so the
/// signature verifies, while a first-wins consumer reads "spoofed" under
/// the same signature.
#[test]
fn check_1_refuses_a_repeated_key_under_a_valid_signature() {
    let signed = signed_minimal_manifest();
    let mutated = signed.replacen('{', r#"{"service_id":"spoofed","#, 1);
    let last_wins: serde_json::Value = serde_json::from_str(&mutated).unwrap();
    assert_eq!(
        last_wins["service_id"], "groove-ref",
        "the planted duplicate must be shadowed"
    );

    let findings = validate_manifest_content(&mutated, "dup.json");
    let critical = critical_check1_findings(&findings);
    assert_eq!(
        critical.len(),
        1,
        "expected exactly one critical DOG-03 finding, got: {findings:#?}"
    );
    assert!(
        critical[0].description.contains("I-JSON"),
        "got: {findings:#?}"
    );
    assert_eq!(
        findings.len(),
        1,
        "no other check may run on a manifest that is not I-JSON, got: {findings:#?}"
    );
}

/// An integer outside ±(2^53−1) is not I-JSON (RFC 7493 §2.2) and is
/// critical at Check 1, with or without `--verify`.
#[test]
fn check_1_refuses_an_integer_beyond_2_pow_53() {
    let mutated = minimal_manifest().replacen('{', r#"{"ttl":9007199254740993,"#, 1);
    let findings = validate_manifest_content(&mutated, "big.json");
    assert_eq!(
        critical_check1_findings(&findings).len(),
        1,
        "an integer outside ±(2^53−1) must be critical at Check 1, got: {findings:#?}"
    );
}

/// An integer beyond the `u64`/`i64` range, which `serde_json` reads as a
/// float, is refused at Check 1 like any other integer outside ±(2^53−1).
/// ijson-jcs before `f135af2` let these through Strict mode.
#[test]
fn check_1_refuses_an_integer_beyond_the_u64_and_i64_range() {
    for literal in ["18446744073709551616", "-9223372036854775809"] {
        let mutated = minimal_manifest().replacen('{', &format!(r#"{{"ttl":{literal},"#), 1);
        let findings = validate_manifest_content(&mutated, "huge.json");
        assert_eq!(
            critical_check1_findings(&findings).len(),
            1,
            "{literal} is outside ±(2^53−1) and must be critical at Check 1, got: {findings:#?}"
        );
    }
}

/// Boundary control for the refusal above: 2^53−1 itself is I-JSON, so the
/// same mutation one below the bound raises nothing at Check 1.
#[test]
fn check_1_accepts_an_integer_at_2_pow_53_minus_1() {
    let mutated = minimal_manifest().replacen('{', r#"{"ttl":9007199254740991,"#, 1);
    let findings = validate_manifest_content(&mutated, "bound.json");
    assert!(
        critical_check1_findings(&findings).is_empty(),
        "2^53−1 is within the I-JSON bound, got: {findings:#?}"
    );
}

/// Unparseable JSON stays Check 1's finding; verify adds nothing.
#[test]
fn verify_leaves_invalid_json_to_check_1() {
    assert!(verify_signature_finding("{ not json", "broken.json").is_empty());
}

/// `groove validate --verify` runs Check 1 and then verify on the same
/// text. A manifest that is not I-JSON, signed or not, yields exactly one
/// critical finding (`DOG-03`) between them, and is never reported as
/// merely unsigned: a repeated `service_id` would otherwise pick which
/// registry pin applies.
#[test]
fn validate_with_verify_reports_a_non_i_json_manifest_once() {
    let cases = [
        (
            "signed",
            signed_minimal_manifest().replacen('{', r#"{"service_id":"spoofed","#, 1),
        ),
        (
            "unsigned",
            minimal_manifest().replacen('{', r#"{"service_id":"spoofed","#, 1),
        ),
        (
            "big integer",
            minimal_manifest().replacen('{', r#"{"ttl":9007199254740993,"#, 1),
        ),
    ];
    for (label, mutated) in cases {
        let mut findings = validate_manifest_content(&mutated, "m.json");
        findings.extend(verify_signature_finding(&mutated, "m.json"));
        let critical = all_critical(&findings);
        assert_eq!(critical.len(), 1, "{label}: got: {findings:#?}");
        assert_eq!(critical[0].check, "DOG-03", "{label}: got: {findings:#?}");
        assert!(
            !findings.iter().any(|f| f.description.contains("unsigned")),
            "{label}: a non-I-JSON manifest must not be reported as merely unsigned: {findings:#?}"
        );
    }
}

/// Positive control for the test above: the unmutated signed manifest
/// passes the same `--verify` sequence with nothing critical.
#[test]
fn validate_with_verify_accepts_a_validly_signed_manifest() {
    let signed = signed_minimal_manifest();
    let mut findings = validate_manifest_content(&signed, "signed.json");
    findings.extend(verify_signature_finding(&signed, "signed.json"));
    assert!(all_critical(&findings).is_empty(), "got: {findings:#?}");
}

/// `probe` and `mesh` parse Strict: a manifest with a repeated key is left
/// out of the listing rather than shown with the last value.
#[test]
fn probe_parse_refuses_a_repeated_key() {
    let mutated = minimal_manifest().replacen('{', r#"{"service_id":"spoofed","#, 1);
    assert!(parse_probed_manifest(&mutated).is_none());
}

/// Positive control for the test above: the unmutated manifest parses.
#[test]
fn probe_parse_accepts_the_minimal_manifest() {
    let manifest = parse_probed_manifest(&minimal_manifest()).expect("minimal manifest is I-JSON");
    assert_eq!(manifest["service_id"], "groove-ref");
}
