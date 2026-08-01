//! Property tests for `smsg_core::frame`: envelope parsing must never panic on
//! arbitrary bytes, must round-trip cleanly, and must uphold the length framing.

use proptest::prelude::*;
use smsg_core::{envelope_bytes, peek, read_header, EnvelopeError, HEADER_LEN};

fn any32() -> impl Strategy<Value = [u8; 32]> {
    [any::<u8>(); 32]
}

proptest! {
    /// Parsing arbitrary bytes never panics and returns Ok or Err.
    #[test]
    fn parsing_arbitrary_bytes_is_total(bytes in any::<Vec<u8>>()) {
        let _ = read_header(&bytes);
        let _ = peek(&bytes);
    }

    /// A framed envelope round-trips: header matches, payload is preserved.
    #[test]
    fn envelope_roundtrip(nh in any32(), vh in any32(), payload in any::<Vec<u8>>()) {
        let bytes = envelope_bytes(&nh, &vh, &payload);
        let header = read_header(&bytes).unwrap();
        assert_eq!(header.name_hash, nh);
        assert_eq!(header.version_hash, vh);
        assert_eq!(header.payload_len, payload.len());
        assert_eq!(&bytes[HEADER_LEN..], &payload[..]);
    }

    /// If a header parses, the length framing is exact.
    #[test]
    fn framing_invariant(bytes in any::<Vec<u8>>()) {
        if let Ok(header) = read_header(&bytes) {
            assert_eq!(bytes.len(), HEADER_LEN + header.payload_len);
        }
    }

    /// `peek` is a lightweight dispatch helper: Ok iff long enough, hashes agree
    /// with `read_header` when the framing is valid.
    #[test]
    fn peek_contract(bytes in any::<Vec<u8>>()) {
        let peek_result = peek(&bytes);
        if bytes.len() >= HEADER_LEN {
            assert!(peek_result.is_ok(), "peek should succeed on long-enough data");
        } else {
            assert!(matches!(peek_result, Err(EnvelopeError::NotAnEnvelope(_))));
        }
        if let (Ok(h), Ok((nh, vh))) = (read_header(&bytes), &peek_result) {
            assert_eq!(h.name_hash, *nh);
            assert_eq!(h.version_hash, *vh);
        }
    }
}

#[test]
fn peek_on_short_data_is_error() {
    for len in [0, 31, 67] {
        let bytes = vec![0u8; len];
        assert!(matches!(peek(&bytes), Err(EnvelopeError::NotAnEnvelope(_))));
    }
}
