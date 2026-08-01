//! Property tests for `soul_msg::Envelope`: decoding arbitrary bytes must never
//! panic, valid messages must round-trip, and strict mode must reject hash
//! tampering while lenient mode tolerates it.

use proptest::prelude::*;
use prost::Message;
use prost_types::field_descriptor_proto::Type;
use prost_types::{DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet};
use soul_msg::{Envelope, EnvelopeError, Policy, Schema};

#[derive(Clone, PartialEq, Message)]
struct ChatMessage {
    #[prost(string, tag = "1")]
    sender: String,
    #[prost(string, tag = "2")]
    content: String,
    #[prost(int64, tag = "3")]
    timestamp: i64,
}

fn descriptor_set() -> Vec<u8> {
    fn field(name: &str, number: i32, r#type: Type) -> FieldDescriptorProto {
        FieldDescriptorProto {
            name: Some(name.to_string()),
            number: Some(number),
            r#type: Some(r#type as i32),
            ..Default::default()
        }
    }
    let chat_msg = DescriptorProto {
        name: Some("ChatMessage".to_string()),
        field: vec![
            field("sender", 1, Type::String),
            field("content", 2, Type::String),
            field("timestamp", 3, Type::Int64),
        ],
        ..Default::default()
    };
    let file = FileDescriptorProto {
        name: Some("chat.proto".to_string()),
        package: Some("chat".to_string()),
        syntax: Some("proto3".to_string()),
        message_type: vec![chat_msg],
        ..Default::default()
    };
    FileDescriptorSet { file: vec![file] }.encode_to_vec()
}

fn meta() -> soul_msg::MessageRef {
    Schema::from_descriptor_set(&descriptor_set())
        .unwrap()
        .by_name("chat.ChatMessage")
        .unwrap()
        .clone()
}

fn chat_message(sender: String, content: String, timestamp: i64) -> ChatMessage {
    ChatMessage {
        sender,
        content,
        timestamp,
    }
}

proptest! {
    /// Decoding arbitrary bytes (strict and lenient) never panics.
    #[test]
    fn decoding_arbitrary_bytes_is_total(bytes in any::<Vec<u8>>()) {
        let m = meta();
        let _ = Envelope::try_deserialize::<ChatMessage>(&bytes, &m);
        let _ = Envelope::try_deserialize_with_policy::<ChatMessage>(&bytes, &m, Policy::Strict);
        let _ = Envelope::try_deserialize_with_policy::<ChatMessage>(&bytes, &m, Policy::Lenient);
    }

    /// A valid message round-trips exactly.
    #[test]
    fn valid_message_roundtrip(sender in any::<String>(), content in any::<String>(), timestamp in any::<i64>()) {
        let msg = chat_message(sender, content, timestamp);
        let m = meta();
        let bytes = Envelope::new(&msg, &m).to_bytes();
        let decoded: ChatMessage = Envelope::try_deserialize(&bytes, &m).unwrap();
        assert_eq!(decoded, msg);
    }

    /// Strict rejects a tampered hash; lenient still decodes when framing is intact.
    #[test]
    fn hash_tamper_policy(flip in 0usize..32usize) {
        let msg = chat_message("Alice".into(), "hi".into(), 1);
        let m = meta();
        let mut bytes = Envelope::new(&msg, &m).to_bytes();
        bytes[flip] ^= 0xFF; // corrupt a byte inside the name/version hash

        let strict = Envelope::try_deserialize::<ChatMessage>(&bytes, &m);
        assert!(
            matches!(strict, Err(EnvelopeError::TypeMismatch { .. }) | Err(EnvelopeError::VersionMismatch { .. })),
            "strict must reject hash tampering, got {:?}",
            strict
        );

        let lenient = Envelope::try_deserialize_with_policy::<ChatMessage>(&bytes, &m, Policy::Lenient);
        assert!(lenient.is_ok(), "lenient must decode intact framing");
    }

    /// Tampering the payload does not break framing; strict still decodes it
    /// (the envelope provides version identity, not payload integrity).
    #[test]
    fn payload_tamper_still_decodes(flip in 0usize..8usize) {
        let msg = chat_message("Alice".into(), "hi".into(), 1);
        let m = meta();
        let mut bytes = Envelope::new(&msg, &m).to_bytes();
        let idx = soul_msg::HEADER_LEN + flip % (bytes.len() - soul_msg::HEADER_LEN);
        bytes[idx] ^= 0xFF;

        let decoded = Envelope::try_deserialize::<ChatMessage>(&bytes, &m);
        // Framing stays valid and the hashes are untouched, so the outcome is
        // either a successful (possibly altered) decode or a payload decode
        // error — never a type/version mismatch.
        assert!(
            !matches!(
                decoded,
                Err(EnvelopeError::TypeMismatch { .. }) | Err(EnvelopeError::VersionMismatch { .. })
            ),
            "payload corruption must not trigger a hash mismatch"
        );
    }
}
