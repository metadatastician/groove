// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Canonical Groove registry, loaded at compile time from
// registry/groove-registry.json — THE single source of truth for port
// assignments, capability types, and service metadata (ADR 0006).
// All other port tables (browser extension targets, spec prose) are
// generated from or validated against that file, never hand-copied.

use std::sync::LazyLock;

use anyhow::{bail, Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};

/// The embedded registry file. Kept public so tests and tooling can assert
/// against the exact bytes the binary was built with.
pub const REGISTRY_JSON: &str = include_str!("../../registry/groove-registry.json");

/// A registered Groove service with canonical port and capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEntry {
    pub id: String,
    pub port: u16,
    pub offers: Vec<String>,
    pub consumes: Vec<String>,
    pub description: String,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// Optional Ed25519 public key pin (base64, 32 bytes) for manifest
    /// signature verification (SPEC §2.1.5, ADR 0010). When present,
    /// consumers MUST verify this service's manifest signature against it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
}

fn default_status() -> String {
    "active".to_string()
}

impl ServiceEntry {
    /// Entries that represent live, probeable assignments.
    pub fn is_probeable(&self) -> bool {
        self.status != "rejected-proposal"
    }
}

/// The parsed shape of registry/groove-registry.json.
#[derive(Debug, Deserialize)]
pub struct RegistryFile {
    pub registry_version: u32,
    pub probe_bands: Vec<(u16, u16)>,
    pub services: Vec<ServiceEntry>,
    pub capability_types: Vec<String>,
    pub protocol_types: Vec<String>,
}

static REGISTRY_FILE: LazyLock<RegistryFile> = LazyLock::new(|| {
    parse_registry(REGISTRY_JSON)
        .expect("embedded registry/groove-registry.json must parse — fix the file, not the code")
});

/// Parse a registry document. Public so tests can round-trip candidate edits.
pub fn parse_registry(json: &str) -> Result<RegistryFile> {
    let reg: RegistryFile =
        serde_json::from_str(json).context("registry JSON does not match the expected schema")?;
    if reg.registry_version != 1 {
        bail!("unsupported registry_version {}", reg.registry_version);
    }
    Ok(reg)
}

/// The canonical Groove service registry.
///
/// Port assignments are authoritative. If any other file disagrees, it is that
/// file which is wrong (and the registry-consistency test should have caught it).
pub static REGISTRY: LazyLock<&'static [ServiceEntry]> =
    LazyLock::new(|| REGISTRY_FILE.services.as_slice());

/// All valid capability type wire names per the Groove schema.
pub static CAPABILITY_TYPES: LazyLock<&'static [String]> =
    LazyLock::new(|| REGISTRY_FILE.capability_types.as_slice());

/// Valid wire protocol types.
pub static PROTOCOL_TYPES: LazyLock<&'static [String]> =
    LazyLock::new(|| REGISTRY_FILE.protocol_types.as_slice());

/// Inclusive port bands that discovery sweeps in addition to registry ports.
pub fn probe_bands() -> &'static [(u16, u16)] {
    REGISTRY_FILE.probe_bands.as_slice()
}

/// Ports worth probing: every probeable registry assignment.
pub fn probe_ports() -> Vec<u16> {
    REGISTRY
        .iter()
        .filter(|e| e.is_probeable())
        .map(|e| e.port)
        .collect()
}

/// Print the canonical registry table.
pub fn print_registry() {
    println!("{}", "Groove Service Registry (canonical)".bold());
    println!("{}", "=".repeat(96));
    println!(
        "{:<14} {:>5}  {:<10}  {:<34}  {}",
        "SERVICE".bold(),
        "PORT".bold(),
        "STATUS".bold(),
        "OFFERS".bold(),
        "CONSUMES".bold()
    );
    println!("{}", "-".repeat(96));

    for entry in REGISTRY.iter() {
        let offers = entry.offers.join(", ");
        let consumes = entry.consumes.join(", ");
        let status = match entry.status.as_str() {
            "rejected-proposal" => entry.status.yellow().to_string(),
            "reference" => entry.status.green().to_string(),
            other => other.to_string(),
        };
        println!(
            "{:<14} {:>5}  {:<10}  {:<34}  {}",
            entry.id, entry.port, status, offers, consumes
        );
    }

    println!("{}", "-".repeat(96));
    println!(
        "{} registered services, {} capability types; probe bands: {}",
        REGISTRY.len(),
        CAPABILITY_TYPES.len(),
        probe_bands()
            .iter()
            .map(|(lo, hi)| format!("{lo}-{hi}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!(
        "{}",
        "source: registry/groove-registry.json (embedded at build time)".dimmed()
    );
}

/// Look up a service by ID.
pub fn find_service(service_id: &str) -> Option<&'static ServiceEntry> {
    REGISTRY.iter().find(|e| e.id == service_id)
}

/// Look up a service by port.
pub fn find_by_port(port: u16) -> Vec<&'static ServiceEntry> {
    REGISTRY.iter().filter(|e| e.port == port).collect()
}

/// Check if a capability type wire name is valid.
pub fn is_valid_capability(cap: &str) -> bool {
    CAPABILITY_TYPES.iter().any(|c| c == cap)
}

/// Check if a protocol type is valid.
pub fn is_valid_protocol(proto: &str) -> bool {
    PROTOCOL_TYPES.iter().any(|p| p == proto)
}

/// Result of a compatibility check between two services.
#[derive(Debug, Serialize)]
pub struct CompatResult {
    pub compatible: bool,
    pub matched: Vec<CapMatch>,
    pub reasons: Vec<String>,
}

/// A single matched capability flow.
#[derive(Debug, Serialize)]
pub struct CapMatch {
    pub provider: String,
    pub consumer: String,
    pub capability: String,
}

/// Check if two capability sets can compose via Groove.
///
/// Composition requires bidirectional capability satisfaction:
/// A.consumes ⊆ B.offers AND B.consumes ⊆ A.offers
///
/// Partial composition (one direction only) is reported as incompatible
/// with an explanatory reason.
pub fn check_compat_entries(a: &ServiceEntry, b: &ServiceEntry) -> CompatResult {
    let mut matched = Vec::new();
    let mut reasons = Vec::new();

    // Check A.consumes ⊆ B.offers
    for cap in &a.consumes {
        if b.offers.contains(cap) {
            matched.push(CapMatch {
                provider: b.id.clone(),
                consumer: a.id.clone(),
                capability: cap.clone(),
            });
        } else {
            reasons.push(format!(
                "{} consumes '{}' but {} does not offer it",
                a.id, cap, b.id
            ));
        }
    }

    // Check B.consumes ⊆ A.offers
    for cap in &b.consumes {
        if a.offers.contains(cap) {
            matched.push(CapMatch {
                provider: a.id.clone(),
                consumer: b.id.clone(),
                capability: cap.clone(),
            });
        } else {
            reasons.push(format!(
                "{} consumes '{}' but {} does not offer it",
                b.id, cap, a.id
            ));
        }
    }

    CompatResult {
        compatible: reasons.is_empty(),
        matched,
        reasons,
    }
}

/// Check if two registered services can compose via Groove.
pub fn check_compat(a_id: &str, b_id: &str) -> Result<CompatResult> {
    let Some(a) = find_service(a_id) else {
        bail!(
            "Service '{}' not found in registry. Use a registered service_id or implement file-based loading.",
            a_id
        );
    };
    let Some(b) = find_service(b_id) else {
        bail!("Service '{}' not found in registry. Use a registered service_id.", b_id);
    };
    Ok(check_compat_entries(a, b))
}
