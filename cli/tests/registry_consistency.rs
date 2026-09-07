// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Registry-consistency guard (ADR 0006): every port table in the repo must
// derive from registry/groove-registry.json. These tests make hand-edited
// drift a build failure instead of a documentation archaeology project.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use groove::registry;

fn repo_root() -> PathBuf {
    // cli/ -> repo root
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli/ has a parent")
        .to_path_buf()
}

#[test]
fn embedded_registry_parses_and_is_version_1() {
    let reg = registry::parse_registry(registry::REGISTRY_JSON).expect("registry parses");
    assert_eq!(reg.registry_version, 1);
    assert!(!reg.services.is_empty());
    assert!(!reg.capability_types.is_empty());
}

#[test]
fn embedded_registry_matches_file_on_disk() {
    let on_disk = fs::read_to_string(repo_root().join("registry/groove-registry.json"))
        .expect("registry/groove-registry.json exists");
    assert_eq!(
        registry::REGISTRY_JSON,
        on_disk,
        "the binary embeds a stale registry — rebuild after editing registry/groove-registry.json"
    );
}

#[test]
fn probeable_ports_are_unique() {
    let mut seen: HashSet<u16> = HashSet::new();
    for entry in registry::REGISTRY.iter().filter(|e| e.is_probeable()) {
        assert!(
            seen.insert(entry.port),
            "port {} is assigned to more than one probeable service (second: '{}')",
            entry.port,
            entry.id
        );
    }
}

#[test]
fn service_capabilities_are_known_types() {
    for entry in registry::REGISTRY.iter() {
        for cap in entry.offers.iter().chain(entry.consumes.iter()) {
            assert!(
                registry::is_valid_capability(cap),
                "service '{}' references unknown capability type '{}'",
                entry.id,
                cap
            );
        }
    }
}

/// Every port literal that appears in a port-like context in spec prose must be
/// a registry assignment or fall inside a probe band. Port-like contexts:
/// `:6473`, `port: 6473`, `port=6473`, and well-known table rows `| 6470 |`.
#[test]
fn spec_port_literals_are_registered_or_in_band() {
    let spec_dir = repo_root().join("spec");
    let registered: HashSet<u16> = registry::REGISTRY.iter().map(|e| e.port).collect();
    let in_band = |p: u16| {
        registry::probe_bands()
            .iter()
            .any(|&(lo, hi)| (lo..=hi).contains(&p))
    };
    let re = regex::Regex::new(r"(?::|port[ =:]+|\| ?)(\d{4,5})\b").unwrap();

    let mut violations = Vec::new();
    for entry in fs::read_dir(&spec_dir).expect("spec/ exists") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("adoc") {
            continue;
        }
        let text = fs::read_to_string(&path).expect("spec file readable");
        for (lineno, line) in text.lines().enumerate() {
            for cap in re.captures_iter(line) {
                let Ok(port) = cap[1].parse::<u16>() else {
                    continue;
                };
                // Only audit plausible service ports; skip e.g. Access-Control-Max-Age.
                if !(1024..=49151).contains(&port) {
                    continue;
                }
                if !registered.contains(&port) && !in_band(port) {
                    violations.push(format!(
                        "{}:{}: port {} not in registry or probe bands",
                        path.file_name().unwrap().to_string_lossy(),
                        lineno + 1,
                        port
                    ));
                }
            }
        }
    }
    assert!(
        violations.is_empty(),
        "spec prose references unregistered ports:\n{}",
        violations.join("\n")
    );
}

/// The JS harness's default probe range must be one of the registry's bands.
#[test]
fn harness_probe_range_matches_a_probe_band() {
    let harness = fs::read_to_string(repo_root().join("harness/groove-harness.js"))
        .expect("harness/groove-harness.js exists");
    let re = regex::Regex::new(r"probeRange \|\| \[(\d+), ?(\d+)\]").unwrap();
    let caps = re
        .captures(&harness)
        .expect("harness declares a default probeRange");
    let lo: u16 = caps[1].parse().unwrap();
    let hi: u16 = caps[2].parse().unwrap();
    assert!(
        registry::probe_bands().contains(&(lo, hi)),
        "harness default probeRange [{lo}, {hi}] is not a registry probe band"
    );
}
