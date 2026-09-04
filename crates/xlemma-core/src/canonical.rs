use serde::Serialize;
use serde_json::{Map, Value};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CanonicalizationError {
    #[error("failed to serialize protocol object: {0}")]
    Serialize(#[from] serde_json::Error),
}

/// Produces a deterministic JSON byte representation for protocol hashing.
///
/// This is a small reference canonicalizer. Production deployments SHOULD use
/// an independently tested RFC 8785 implementation and publish test vectors.
pub fn canonical_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, CanonicalizationError> {
    let value = serde_json::to_value(value)?;
    let sorted = sort_value(value);
    Ok(serde_json::to_vec(&sorted)?)
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

fn sort_value(value: Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut entries: Vec<_> = object.into_iter().collect();
            entries.sort_by(|(left, _), (right, _)| left.cmp(right));
            let mut sorted = Map::new();
            for (key, value) in entries {
                let previous = sorted.insert(key, sort_value(value));
                debug_assert!(previous.is_none());
            }
            Value::Object(sorted)
        }
        Value::Array(values) => Value::Array(values.into_iter().map(sort_value).collect()),
        scalar => scalar,
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
}
