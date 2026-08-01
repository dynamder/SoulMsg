//! Envelope framing: a fixed-layout header wrapping a protobuf payload.
//!
//! Wire layout: `[name_hash:32][version_hash:32][payload_len:u32(LE)][payload]`.

use std::fmt;

/// Fixed header size: name_hash(32) + version_hash(32) + payload_len(4).
pub const HEADER_LEN: usize = 68;

/// Decode-time schema compatibility policy.
///
/// - `Strict` (default): the wire name/version hashes must equal the expected
///   message's hashes, otherwise `TypeMismatch` / `VersionMismatch` is returned.
/// - `Lenient`: hashes are ignored and the payload is decoded with protobuf's
///   tolerant evolution semantics. Structural framing is still enforced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Policy {
    #[default]
    Strict,
    Lenient,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnvelopeError {
    NotAnEnvelope(String),
    TypeMismatch {
        expected_name_hash: [u8; 32],
        actual_name_hash: [u8; 32],
    },
    VersionMismatch {
        expected_version_hash: [u8; 32],
        actual_version_hash: [u8; 32],
    },
    DeserializeError(String),
}

impl fmt::Display for EnvelopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnvelopeError::NotAnEnvelope(m) => write!(f, "Not an envelope: {}", m),
            EnvelopeError::TypeMismatch {
                expected_name_hash,
                actual_name_hash,
            } => write!(
                f,
                "Type mismatch: expected name_hash {:02x?}, got {:02x?}",
                expected_name_hash, actual_name_hash
            ),
            EnvelopeError::VersionMismatch {
                expected_version_hash,
                actual_version_hash,
            } => write!(
                f,
                "Version mismatch: expected version_hash {:02x?}, got {:02x?}",
                expected_version_hash, actual_version_hash
            ),
            EnvelopeError::DeserializeError(m) => write!(f, "Deserialize error: {}", m),
        }
    }
}

impl std::error::Error for EnvelopeError {}

/// Parsed envelope header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvelopeHeader {
    pub name_hash: [u8; 32],
    pub version_hash: [u8; 32],
    pub payload_len: usize,
}

/// Builds the full envelope bytes: header + payload.
pub fn envelope_bytes(name_hash: &[u8; 32], version_hash: &[u8; 32], payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(HEADER_LEN + payload.len());
    out.extend_from_slice(name_hash);
    out.extend_from_slice(version_hash);
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(payload);
    out
}

/// Reads the header from the buffer, validating the payload length framing
/// (the buffer must contain exactly `HEADER_LEN + payload_len` bytes).
pub fn read_header(bytes: &[u8]) -> Result<EnvelopeHeader, EnvelopeError> {
    if bytes.len() < HEADER_LEN {
        return Err(EnvelopeError::NotAnEnvelope(
            "Data too short for the envelope header.".to_string(),
        ));
    }
    let mut name_hash = [0u8; 32];
    name_hash.copy_from_slice(&bytes[0..32]);
    let mut version_hash = [0u8; 32];
    version_hash.copy_from_slice(&bytes[32..64]);
    let payload_len = u32::from_le_bytes(bytes[64..68].try_into().unwrap()) as usize;
    if bytes.len() != HEADER_LEN + payload_len {
        return Err(EnvelopeError::NotAnEnvelope(format!(
            "Payload length mismatch: header says {} bytes but {} remain",
            payload_len,
            bytes.len() - HEADER_LEN
        )));
    }
    Ok(EnvelopeHeader {
        name_hash,
        version_hash,
        payload_len,
    })
}

/// Parses just the two hashes from the buffer (for dispatch when the message
/// type is not known ahead of time). Does not require a full, framed envelope.
pub fn peek(bytes: &[u8]) -> Result<([u8; 32], [u8; 32]), EnvelopeError> {
    if bytes.len() < HEADER_LEN {
        return Err(EnvelopeError::NotAnEnvelope(
            "Data too short for the envelope header.".to_string(),
        ));
    }
    let mut name_hash = [0u8; 32];
    name_hash.copy_from_slice(&bytes[0..32]);
    let mut version_hash = [0u8; 32];
    version_hash.copy_from_slice(&bytes[32..64]);
    Ok((name_hash, version_hash))
}

#[cfg(test)]
mod tests {
    use super::*;

    const NH: [u8; 32] = [1u8; 32];
    const VH: [u8; 32] = [2u8; 32];

    #[test]
    fn test_envelope_roundtrip_header() {
        let payload = vec![0x0a, 0x05, b'h', b'e', b'l', b'l', b'o'];
        let bytes = envelope_bytes(&NH, &VH, &payload);
        assert_eq!(bytes.len(), HEADER_LEN + payload.len());
        let h = read_header(&bytes).unwrap();
        assert_eq!(h.name_hash, NH);
        assert_eq!(h.version_hash, VH);
        assert_eq!(h.payload_len, payload.len());
        assert_eq!(&bytes[HEADER_LEN..], &payload[..]);
    }

    #[test]
    fn test_too_short_rejected() {
        assert!(read_header(&[0u8; 32]).is_err());
        assert!(peek(&[0u8; 32]).is_err());
    }

    #[test]
    fn test_trailing_bytes_rejected() {
        let mut bytes = envelope_bytes(&NH, &VH, &[1, 2, 3]);
        bytes.push(0xFF);
        assert!(matches!(
            read_header(&bytes),
            Err(EnvelopeError::NotAnEnvelope(_))
        ));
    }

    #[test]
    fn test_peek_extracts_hashes() {
        let bytes = envelope_bytes(&NH, &VH, &[9]);
        let (nh, vh) = peek(&bytes).unwrap();
        assert_eq!(nh, NH);
        assert_eq!(vh, VH);
    }
}
