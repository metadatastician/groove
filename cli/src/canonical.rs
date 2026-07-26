// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Canonical JSON for manifest signing (SPEC §2.1.5, ADR 0010).
//
// The JCS (RFC 8785) subset sufficient for Groove manifests: UTF-8 output,
// object keys sorted lexicographically, minimal separators, RFC 8259
// string escaping. Deliberate restrictions, enforced not assumed:
//   * numbers MUST be integers (floats are rejected — manifests carry
//     versions, ports and TTLs, never measurements), so the ECMAScript
//     number-formatting corner of JCS never arises;
//   * object keys are compared as UTF-8 byte sequences, which coincides
//     with JCS's UTF-16 ordering on the ASCII keys manifests use.

use anyhow::{bail, Result};
use serde_json::Value;

/// Serialise `value` in canonical form. Errors on any float — a manifest
/// carrying one is not signable under this profile.
pub fn canonical_json(value: &Value) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(256);
    write_canonical(value, &mut out)?;
    Ok(out)
}

fn write_canonical(value: &Value, out: &mut Vec<u8>) -> Result<()> {
    match value {
        Value::Null => out.extend_from_slice(b"null"),
        Value::Bool(true) => out.extend_from_slice(b"true"),
        Value::Bool(false) => out.extend_from_slice(b"false"),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                out.extend_from_slice(i.to_string().as_bytes());
            } else if let Some(u) = n.as_u64() {
                out.extend_from_slice(u.to_string().as_bytes());
            } else {
                bail!("canonical JSON profile forbids non-integer numbers (got {n})");
            }
        }
        Value::String(s) => {
            // serde_json string escaping is RFC 8259-minimal, matching the
            // JCS escaping rules for the code points manifests contain.
            out.extend_from_slice(
                serde_json::to_string(s).expect("string serialises").as_bytes(),
            );
        }
        Value::Array(items) => {
            out.push(b'[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                write_canonical(item, out)?;
            }
            out.push(b']');
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_unstable();
            out.push(b'{');
            for (i, key) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                out.extend_from_slice(
                    serde_json::to_string(key).expect("key serialises").as_bytes(),
                );
                out.push(b':');
                write_canonical(&map[key.as_str()], out)?;
            }
            out.push(b'}');
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn keys_sorted_minimal_separators() {
        let v = json!({"b": 1, "a": {"z": true, "m": [1, 2, "x"]}, "c": null});
        let c = String::from_utf8(canonical_json(&v).unwrap()).unwrap();
        assert_eq!(c, r#"{"a":{"m":[1,2,"x"],"z":true},"b":1,"c":null}"#);
    }

    #[test]
    fn insertion_order_does_not_matter() {
        let a = json!({"x": 1, "y": 2});
        let b: Value = serde_json::from_str(r#"{"y": 2, "x": 1}"#).unwrap();
        assert_eq!(canonical_json(&a).unwrap(), canonical_json(&b).unwrap());
    }

    #[test]
    fn floats_are_rejected() {
        assert!(canonical_json(&json!({"ratio": 0.5})).is_err());
    }

    #[test]
    fn strings_escaped_rfc8259() {
        let v = json!({"s": "a\"b\\c\nd"});
        let c = String::from_utf8(canonical_json(&v).unwrap()).unwrap();
        assert_eq!(c, r#"{"s":"a\"b\\c\nd"}"#);
    }
}
