//! Canonical length-delimited XLMP framing for binary transports.
//!
//! WebSocket binary messages, libp2p streams, and other non-HTTP transports can
//! carry this frame without changing the canonical XLMP envelope or MessageID.

use crate::XlmpEnvelope;
use thiserror::Error;
use xlemma_core::canonical_json_bytes;

pub const XLMP_FRAME_HEADER_BYTES: usize = 4;
pub const XLMP_MAX_FRAME_BYTES: usize = 1024 * 1024;

/// Encode one canonical XLMP envelope as a four-byte big-endian length followed
/// by RFC 8785 JSON. The transport frame is not included in the MessageID.
pub fn encode_xlmp_frame(envelope: &XlmpEnvelope) -> Result<Vec<u8>, XlmpFrameError> {
    envelope
        .validate_integrity()
        .map_err(|error| XlmpFrameError::InvalidEnvelope(error.to_string()))?;
    let payload = canonical_json_bytes(envelope)?;
    if payload.is_empty() || payload.len() > XLMP_MAX_FRAME_BYTES {
        return Err(XlmpFrameError::InvalidLength(payload.len()));
    }
    let length =
        u32::try_from(payload.len()).map_err(|_| XlmpFrameError::InvalidLength(payload.len()))?;
    let mut frame = Vec::with_capacity(XLMP_FRAME_HEADER_BYTES + payload.len());
    frame.extend_from_slice(&length.to_be_bytes());
    frame.extend_from_slice(&payload);
    Ok(frame)
}

/// Decode exactly one frame and require its JSON payload to already be in
/// canonical form. This prevents two transports from authenticating different
/// byte representations as though they were one wire object.
pub fn decode_xlmp_frame(frame: &[u8]) -> Result<XlmpEnvelope, XlmpFrameError> {
    if frame.len() < XLMP_FRAME_HEADER_BYTES {
        return Err(XlmpFrameError::TruncatedHeader);
    }
    let declared = u32::from_be_bytes(
        frame[..XLMP_FRAME_HEADER_BYTES]
            .try_into()
            .expect("the frame header length was checked"),
    ) as usize;
    if declared == 0 || declared > XLMP_MAX_FRAME_BYTES {
        return Err(XlmpFrameError::InvalidLength(declared));
    }
    let actual = frame.len() - XLMP_FRAME_HEADER_BYTES;
    if declared != actual {
        return Err(XlmpFrameError::LengthMismatch { declared, actual });
    }

    let payload = &frame[XLMP_FRAME_HEADER_BYTES..];
    let envelope: XlmpEnvelope = serde_json::from_slice(payload)?;
    let canonical = canonical_json_bytes(&envelope)?;
    if payload != canonical {
        return Err(XlmpFrameError::NonCanonicalPayload);
    }
    envelope
        .validate_integrity()
        .map_err(|error| XlmpFrameError::InvalidEnvelope(error.to_string()))?;
    Ok(envelope)
}

#[derive(Debug, Error)]
pub enum XlmpFrameError {
    #[error("XLMP frame header is truncated")]
    TruncatedHeader,
    #[error("XLMP frame length {0} is zero or exceeds the protocol limit")]
    InvalidLength(usize),
    #[error("XLMP frame declared {declared} bytes but carried {actual}")]
    LengthMismatch { declared: usize, actual: usize },
    #[error("XLMP frame contains malformed JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("XLMP frame canonicalization failed: {0}")]
    Canonicalization(#[from] xlemma_core::CanonicalizationError),
    #[error("XLMP frame payload is not canonical RFC 8785 JSON")]
    NonCanonicalPayload,
    #[error("XLMP frame contains an invalid envelope: {0}")]
    InvalidEnvelope(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use xlemma_core::MessageId;

    fn published_envelope() -> XlmpEnvelope {
        serde_json::from_str(include_str!(
            "../../../examples/no-arbitrage/xlmp-envelope.json"
        ))
        .unwrap()
    }

    #[test]
    fn binary_transport_round_trip_preserves_message_identity() {
        let original = published_envelope();
        let decoded = decode_xlmp_frame(&encode_xlmp_frame(&original).unwrap()).unwrap();
        assert_eq!(decoded, original);
        assert_eq!(decoded.message_id, original.message_id);
    }

    #[test]
    fn noncanonical_json_frame_is_rejected() {
        let envelope = published_envelope();
        let mut payload = serde_json::to_vec_pretty(&envelope).unwrap();
        let mut frame = Vec::new();
        frame.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        frame.append(&mut payload);
        assert!(matches!(
            decode_xlmp_frame(&frame),
            Err(XlmpFrameError::NonCanonicalPayload)
        ));
    }

    #[test]
    fn trailing_or_truncated_frame_data_is_rejected() {
        let envelope = published_envelope();
        let mut trailing = encode_xlmp_frame(&envelope).unwrap();
        trailing.push(0);
        assert!(matches!(
            decode_xlmp_frame(&trailing),
            Err(XlmpFrameError::LengthMismatch { .. })
        ));

        let mut truncated = encode_xlmp_frame(&envelope).unwrap();
        truncated.pop();
        assert!(matches!(
            decode_xlmp_frame(&truncated),
            Err(XlmpFrameError::LengthMismatch { .. })
        ));
    }

    #[test]
    fn valid_frame_with_mutated_message_id_is_rejected() {
        let mut envelope = published_envelope();
        envelope.message_id = MessageId::derive(&"wrong-message").unwrap();
        assert!(matches!(
            encode_xlmp_frame(&envelope),
            Err(XlmpFrameError::InvalidEnvelope(_))
        ));
    }
}
