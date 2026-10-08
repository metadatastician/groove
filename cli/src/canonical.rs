// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Canonical JSON for manifest signing (SPEC §2.1.5, ADR 0010).
//
// The canonical bytes are JCS (RFC 8785) as produced by the estate's JSON
// canon, `hyperpolymath/ijson-jcs`, over a value that is I-JSON (RFC 7493):
// UTF-8 output, object keys sorted by UTF-16 code units, minimal separators,
// JCS string escaping, integers bounded to ±(2^53−1). One restriction is
// groove's own and is enforced here before ijson-jcs runs:
//   * numbers MUST be integers (floats are rejected — manifests carry
//     versions, ports and TTLs, never measurements), so the ECMAScript
//     number-formatting corner of JCS never reaches a signature.

use anyhow::{Result, bail};
use serde_json::Value;

/// Serialise `value` in canonical form. Errors on any float, and on any
/// value that is not I-JSON (an integer outside ±(2^53−1), a noncharacter
/// in a string) — a manifest carrying one is not signable under this profile.
pub fn canonical_json(value: &Value) -> Result<Vec<u8>> {
    reject_non_integers(value)?;
    Ok(ijson_jcs::to_jcs(value)?)
}

/// Refuse any non-integer number anywhere in `value` (the sign-path guard
/// that ijson-jcs, which formats doubles per ECMAScript, does not apply).
fn reject_non_integers(value: &Value) -> Result<()> {
    match value {
        Value::Number(n) if n.is_f64() => {
            bail!("canonical JSON profile forbids non-integer numbers (got {n})")
        }
        Value::Array(items) => items.iter().try_for_each(reject_non_integers),
        Value::Object(map) => map.values().try_for_each(reject_non_integers),
        _ => Ok(()),
    }
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
        assert!(canonical_json(&json!({"deep": [{"x": 1.0}]})).is_err());
    }

    #[test]
    fn strings_escaped_rfc8259() {
        let v = json!({"s": "a\"b\\c\nd"});
        let c = String::from_utf8(canonical_json(&v).unwrap()).unwrap();
        assert_eq!(c, r#"{"s":"a\"b\\c\nd"}"#);
    }

    /// RFC 8785 §3.2.3: keys sort by UTF-16 code units. U+1F600 is the
    /// surrogate pair D83D DE00, which sorts before U+E000; UTF-8 byte
    /// order (F0… vs EE…) would put it after.
    #[test]
    fn keys_sorted_by_utf16_code_units() {
        let v = json!({"\u{E000}": 1, "\u{1F600}": 2});
        let c = String::from_utf8(canonical_json(&v).unwrap()).unwrap();
        assert_eq!(c, "{\"\u{1F600}\":2,\"\u{E000}\":1}");
    }

    /// RFC 7493 §2.2: integers must be exactly representable as a double.
    #[test]
    fn integers_beyond_2_pow_53_are_rejected() {
        assert!(canonical_json(&json!({"n": 9007199254740991u64})).is_ok());
        assert!(canonical_json(&json!({"n": -9007199254740991i64})).is_ok());
        assert!(canonical_json(&json!({"n": 9007199254740992u64})).is_err());
        assert!(canonical_json(&json!({"n": -9007199254740992i64})).is_err());
    }

    /// RFC 7493 §2.1: a noncharacter in a string is not I-JSON.
    #[test]
    fn noncharacters_are_rejected() {
        assert!(canonical_json(&json!({"s": "\u{FFFE}"})).is_err());
        assert!(canonical_json(&json!({"s": "\u{FFFD}"})).is_ok());
    }
}
