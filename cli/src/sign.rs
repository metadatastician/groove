// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Manifest signing and verification (SPEC §2.1.5, ADR 0010).
//
// Ed25519 over the canonical JSON (canonical.rs) of the manifest object
// with its `signature` member removed. Trust anchoring is the registry:
// when registry/groove-registry.json pins a `public_key` for a service, a
// mismatching or missing signature is a hard failure — the capability-
// spoofing countermeasure of cleave PROOF-NEEDS G-5/O-8.

use anyhow::{anyhow, bail, Context, Result};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde_json::{json, Value};

use crate::canonical::canonical_json;
use crate::timefmt::rfc3339_now;

/// Outcome of verifying a manifest's signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyOutcome {
    /// No signature member, and no registry pin demanded one.
    Unsigned,
    /// Signature verifies against the key embedded in the manifest; no
    /// registry pin exists, so this binds the manifest to its key but
    /// authenticates nothing by itself.
    ValidSelf,
    /// Signature verifies AND the key matches the registry pin.
    ValidPinned,
}

/// Decode a base64 32-byte Ed25519 seed (e.g. from GROOVE_SIGNING_KEY).
pub fn decode_seed(b64: &str) -> Result<[u8; 32]> {
    let bytes = B64.decode(b64.trim()).context("signing seed is not valid base64")?;
    let seed: [u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| anyhow!("signing seed must be exactly 32 bytes, got {}", bytes.len()))?;
    Ok(seed)
}

/// The base64 public key for a seed (what a registry pin should carry).
pub fn public_key_b64(seed: &[u8; 32]) -> String {
    B64.encode(SigningKey::from_bytes(seed).verifying_key().to_bytes())
}

/// Return a copy of `manifest` carrying a detached `signature` member
/// (SPEC §2.1.5). The signed payload is the canonical JSON of the manifest
/// WITHOUT the signature member; `signed_at` is informational and sits
/// inside the (excluded) member.
pub fn sign_manifest(manifest: &Value, seed: &[u8; 32]) -> Result<Value> {
    let Value::Object(_) = manifest else {
        bail!("manifest must be a JSON object");
    };
    let mut unsigned = manifest.clone();
    unsigned.as_object_mut().expect("checked object").remove("signature");

    let payload = canonical_json(&unsigned)?;
    let key = SigningKey::from_bytes(seed);
    let sig = key.sign(&payload);

    let mut signed = unsigned;
    signed["signature"] = json!({
        "alg": "ed25519",
        "public_key": B64.encode(key.verifying_key().to_bytes()),
        "sig": B64.encode(sig.to_bytes()),
        "signed_at": rfc3339_now(),
    });
    Ok(signed)
}

/// Verify a manifest, optionally against a registry-pinned public key.
///
/// * pinned + unsigned manifest → error (a pinned service MUST sign).
/// * pinned + key mismatch → error.
/// * bad signature → error.
/// * otherwise → the achieved [`VerifyOutcome`].
pub fn verify_manifest(manifest: &Value, pinned_public_key_b64: Option<&str>) -> Result<VerifyOutcome> {
    let signature = manifest.get("signature");
    let Some(signature) = signature.filter(|s| !s.is_null()) else {
        if pinned_public_key_b64.is_some() {
            bail!(
                "registry pins a public key for this service but the manifest is unsigned \
                 (SPEC §2.1.5: treat as failed discovery)"
            );
        }
        return Ok(VerifyOutcome::Unsigned);
    };

    let alg = signature["alg"].as_str().unwrap_or_default();
    if alg != "ed25519" {
        bail!("unsupported signature alg '{alg}' (this profile is ed25519-only)");
    }
    let key_b64 = signature["public_key"]
        .as_str()
        .ok_or_else(|| anyhow!("signature.public_key missing"))?;
    let sig_b64 = signature["sig"].as_str().ok_or_else(|| anyhow!("signature.sig missing"))?;

    if let Some(pin) = pinned_public_key_b64 {
        if pin.trim() != key_b64 {
            bail!(
                "manifest signature key does not match the registry pin \
                 (SPEC §2.1.5: treat as failed discovery)"
            );
        }
    }

    let key_bytes: [u8; 32] = B64
        .decode(key_b64)
        .context("signature.public_key is not valid base64")?
        .as_slice()
        .try_into()
        .map_err(|_| anyhow!("signature.public_key must decode to 32 bytes"))?;
    let sig_bytes: [u8; 64] = B64
        .decode(sig_b64)
        .context("signature.sig is not valid base64")?
        .as_slice()
        .try_into()
        .map_err(|_| anyhow!("signature.sig must decode to 64 bytes"))?;

    let mut unsigned = manifest.clone();
    unsigned.as_object_mut().ok_or_else(|| anyhow!("manifest must be a JSON object"))?.remove("signature");
    let payload = canonical_json(&unsigned)?;

    let verifying = VerifyingKey::from_bytes(&key_bytes).context("invalid Ed25519 public key")?;
    verifying
        .verify_strict(&payload, &Signature::from_bytes(&sig_bytes))
        .context("manifest signature does not verify over the canonical payload")?;

    Ok(if pinned_public_key_b64.is_some() {
        VerifyOutcome::ValidPinned
    } else {
        VerifyOutcome::ValidSelf
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const SEED: [u8; 32] = [7u8; 32];

    fn manifest() -> Value {
        json!({
            "groove_version": "1",
            "service_id": "test-svc",
            "service_version": "0.0.1",
            "mode": "active",
            "capabilities": { "attest": { "type": "attestation" } },
            "consumes": [],
        })
    }

    #[test]
    fn sign_then_verify_roundtrips() {
        let signed = sign_manifest(&manifest(), &SEED).unwrap();
        assert_eq!(signed["signature"]["alg"], "ed25519");
        assert_eq!(verify_manifest(&signed, None).unwrap(), VerifyOutcome::ValidSelf);
        let pin = public_key_b64(&SEED);
        assert_eq!(verify_manifest(&signed, Some(&pin)).unwrap(), VerifyOutcome::ValidPinned);
    }

    #[test]
    fn tampering_breaks_the_signature() {
        let mut signed = sign_manifest(&manifest(), &SEED).unwrap();
        signed["capabilities"]["voice"] = json!({ "type": "voice" }); // spoof a capability
        assert!(verify_manifest(&signed, None).is_err());
    }

    #[test]
    fn key_ordering_does_not_affect_verification() {
        // Re-parse with different member order: canonicalisation makes the
        // signature survive any JSON re-serialisation.
        let signed = sign_manifest(&manifest(), &SEED).unwrap();
        let reordered: Value =
            serde_json::from_str(&serde_json::to_string(&signed).unwrap()).unwrap();
        assert!(verify_manifest(&reordered, None).is_ok());
    }

    #[test]
    fn pin_mismatch_and_pinned_unsigned_fail() {
        let signed = sign_manifest(&manifest(), &[9u8; 32]).unwrap();
        let wrong_pin = public_key_b64(&SEED);
        assert!(verify_manifest(&signed, Some(&wrong_pin)).is_err());
        assert!(verify_manifest(&manifest(), Some(&wrong_pin)).is_err());
        assert_eq!(verify_manifest(&manifest(), None).unwrap(), VerifyOutcome::Unsigned);
    }
}
