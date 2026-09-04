use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CanonicalizationError {
    #[error("failed to serialize protocol object: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("JCS integer is outside the exact IEEE-754 range")]
    UnsafeInteger,
}

/// Produces a deterministic JSON byte representation for protocol hashing.
///
/// This uses RFC 8785 JSON Canonicalization Scheme (JCS). Because JCS numbers
/// have IEEE-754 binary64 semantics, integers outside JavaScript's exact range
/// are rejected instead of being silently rounded. Protocol schemas must carry
/// larger integers as decimal strings.
pub fn canonical_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, CanonicalizationError> {
    let value = serde_json::to_value(value)?;
    ensure_jcs_safe(&value)?;
    Ok(serde_json_canonicalizer::to_vec(&value)?)
}

pub fn canonical_json_hash<T: Serialize>(
    domain: &str,
    value: &T,
) -> Result<[u8; 32], CanonicalizationError> {
    let bytes = canonical_json_bytes(value)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"xlemma\0");
    hasher.update(domain.as_bytes());
    hasher.update(b"\0");
    hasher.update(&bytes);
    Ok(*hasher.finalize().as_bytes())
}

fn ensure_jcs_safe(value: &serde_json::Value) -> Result<(), CanonicalizationError> {
    const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
    match value {
        serde_json::Value::Number(number) => {
            let safe = if let Some(value) = number.as_u64() {
                value <= MAX_SAFE_INTEGER
            } else if let Some(value) = number.as_i64() {
                value >= -(MAX_SAFE_INTEGER as i64)
            } else {
                number.as_f64().is_some_and(f64::is_finite)
            };
            if safe {
                Ok(())
            } else {
                Err(CanonicalizationError::UnsafeInteger)
            }
        }
        serde_json::Value::Array(values) => values.iter().try_for_each(ensure_jcs_safe),
        serde_json::Value::Object(values) => values.values().try_for_each(ensure_jcs_safe),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn object_key_order_does_not_change_hash() {
        let left = json!({"b": 2, "a": 1});
        let right = json!({"a": 1, "b": 2});
        assert_eq!(
            canonical_json_hash("test", &left).unwrap(),
            canonical_json_hash("test", &right).unwrap()
        );
    }

    #[test]
    fn follows_rfc_8785_number_and_utf16_key_ordering() {
        let value = json!({
            "\u{20ac}": "euro",
            "\r": "carriage return",
            "1": 333_333_333.333_333_3,
            "\u{0080}": "control",
            "\u{00f6}": "latin",
            "\u{1f600}": "emoji"
        });
        assert_eq!(
            String::from_utf8(canonical_json_bytes(&value).unwrap()).unwrap(),
            "{\"\\r\":\"carriage return\",\"1\":333333333.3333333,\"\u{0080}\":\"control\",\"\u{00f6}\":\"latin\",\"\u{20ac}\":\"euro\",\"\u{1f600}\":\"emoji\"}"
        );
    }

    #[test]
    fn rejects_integers_that_jcs_cannot_represent_exactly() {
        assert!(matches!(
            canonical_json_bytes(&json!(9_007_199_254_740_992_u64)),
            Err(CanonicalizationError::UnsafeInteger)
        ));
    }
}
