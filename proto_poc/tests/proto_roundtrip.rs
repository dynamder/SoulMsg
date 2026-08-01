use prost::Message;
use proto_poc::{chat, chat_old, default_schema, EnvelopeError, Policy, ProtoEnvelope};

fn schema() -> proto_poc::ProtoSchema {
    default_schema()
}

#[test]
fn test_roundtrip_chat_message() {
    let schema = schema();
    let meta = schema
        .by_name("chat.ChatMessage")
        .expect("ChatMessage in schema");

    let msg = chat::ChatMessage {
        sender: "Alice".to_string(),
        content: "Hello, World!".to_string(),
        timestamp: 1699999999,
    };

    let envelope = ProtoEnvelope::new(&msg, meta);
    let bytes = envelope.to_bytes();

    // [name_hash:32][version_hash:32][payload]
    assert_eq!(bytes.len(), proto_poc::HEADER_LEN + msg.encoded_len());

    let decoded: chat::ChatMessage = ProtoEnvelope::try_deserialize(&bytes, meta).unwrap();
    assert_eq!(decoded.sender, "Alice");
    assert_eq!(decoded.content, "Hello, World!");
    assert_eq!(decoded.timestamp, 1699999999);
}

#[test]
fn test_nested_message_roundtrip() {
    let schema = schema();
    let meta = schema
        .by_name("chat.RobotState")
        .expect("RobotState in schema");

    let msg = chat::RobotState {
        name: "R2-D2".to_string(),
        position: Some(chat::Position {
            x: 100.5,
            y: 200.5,
            z: 300.5,
        }),
        status: 42,
    };

    let envelope = ProtoEnvelope::new(&msg, meta);
    let bytes = envelope.to_bytes();
    let decoded: chat::RobotState = ProtoEnvelope::try_deserialize(&bytes, meta).unwrap();

    assert_eq!(decoded.name, "R2-D2");
    let pos = decoded.position.expect("position present");
    assert_eq!(pos.x, 100.5);
    assert_eq!(pos.y, 200.5);
    assert_eq!(pos.z, 300.5);
    assert_eq!(decoded.status, 42);
}

#[test]
fn test_type_mismatch_is_rejected() {
    let schema = schema();
    let chat_meta = schema.by_name("chat.ChatMessage").unwrap();
    let pos_meta = schema.by_name("chat.Position").unwrap();

    let msg = chat::ChatMessage {
        sender: "Alice".to_string(),
        content: "Hi".to_string(),
        timestamp: 1,
    };
    let bytes = ProtoEnvelope::new(&msg, chat_meta).to_bytes();

    // Try to decode a ChatMessage envelope as a Position -> name_hash mismatch.
    let result: Result<chat::Position, EnvelopeError> =
        ProtoEnvelope::try_deserialize(&bytes, pos_meta);
    assert!(matches!(result, Err(EnvelopeError::TypeMismatch { .. })));
}

#[test]
fn test_version_mismatch_is_rejected() {
    let schema = schema();
    let chat_meta = schema.by_name("chat.ChatMessage").unwrap();
    let old_meta = schema.by_name("chat_old.ChatMessage").unwrap();

    let msg = chat::ChatMessage {
        sender: "Alice".to_string(),
        content: "Hi".to_string(),
        timestamp: 1,
    };
    let bytes = ProtoEnvelope::new(&msg, chat_meta).to_bytes();

    // Same message name but different schema (extra field) -> version_hash mismatch.
    let result: Result<chat_old::ChatMessage, EnvelopeError> =
        ProtoEnvelope::try_deserialize(&bytes, old_meta);
    assert!(matches!(result, Err(EnvelopeError::VersionMismatch { .. })));
}

#[test]
fn test_hash_stability_across_runs() {
    let schema = schema();
    let chat_meta = schema.by_name("chat.ChatMessage").unwrap();
    assert_eq!(chat_meta.name_hash, chat_meta.name_hash);
    assert_eq!(chat_meta.version_hash, chat_meta.version_hash);
}

#[test]
fn test_name_hash_stable_version_hash_changes() {
    let schema = schema();
    let chat = schema.by_name("chat.ChatMessage").unwrap();
    let old = schema.by_name("chat_old.ChatMessage").unwrap();

    // Both messages are named "ChatMessage" (different packages). The name hash
    // is derived from the short name only, so it stays stable across schema
    // edits; the version hash captures the (full) definition and must differ.
    assert_eq!(chat.name_hash, old.name_hash);
    assert_ne!(chat.version_hash, old.version_hash);
}

#[test]
fn test_short_data_is_not_an_envelope() {
    let schema = schema();
    let meta = schema.by_name("chat.ChatMessage").unwrap();
    let result: Result<chat::ChatMessage, EnvelopeError> =
        ProtoEnvelope::try_deserialize(&[0u8; 32], meta);
    assert!(matches!(result, Err(EnvelopeError::NotAnEnvelope(_))));
}

#[test]
fn test_trailing_garbage_is_rejected() {
    let schema = schema();
    let meta = schema.by_name("chat.ChatMessage").unwrap();
    let msg = chat::ChatMessage {
        sender: "A".to_string(),
        content: "B".to_string(),
        timestamp: 1,
    };
    let mut bytes = ProtoEnvelope::new(&msg, meta).to_bytes();
    bytes.push(0xFF);
    bytes.push(0xFF);

    let result: Result<chat::ChatMessage, EnvelopeError> =
        ProtoEnvelope::try_deserialize(&bytes, meta);
    // The payload length framing rejects the trailing bytes.
    assert!(matches!(result, Err(EnvelopeError::NotAnEnvelope(_))));
}

#[test]
fn test_strict_rejects_type_mismatch() {
    let schema = schema();
    let chat_meta = schema.by_name("chat.ChatMessage").unwrap();
    let pos_meta = schema.by_name("chat.Position").unwrap();

    let msg = chat::ChatMessage {
        sender: "Alice".to_string(),
        content: "Hi".to_string(),
        timestamp: 1,
    };
    let bytes = ProtoEnvelope::new(&msg, chat_meta).to_bytes();

    let result: Result<chat::Position, EnvelopeError> =
        ProtoEnvelope::try_deserialize_with_policy(&bytes, pos_meta, Policy::Strict);
    assert!(matches!(result, Err(EnvelopeError::TypeMismatch { .. })));
}

#[test]
fn test_lenient_skips_hash_checks() {
    let schema = schema();
    let chat_meta = schema.by_name("chat.ChatMessage").unwrap();
    let pos_meta = schema.by_name("chat.Position").unwrap();

    let msg = chat::ChatMessage {
        sender: "Alice".to_string(),
        content: "Hi".to_string(),
        timestamp: 1,
    };
    let bytes = ProtoEnvelope::new(&msg, chat_meta).to_bytes();

    // Strict reports a TypeMismatch on the name hash.
    let strict: Result<chat::Position, EnvelopeError> =
        ProtoEnvelope::try_deserialize(&bytes, pos_meta);
    assert!(matches!(strict, Err(EnvelopeError::TypeMismatch { .. })));

    // Lenient never consults the hashes: the outcome comes only from the payload
    // decoder. ChatMessage's wire types are not a Position's, so prost rejects
    // the reinterpretation at the payload level (not via the hashes).
    let lenient: Result<chat::Position, EnvelopeError> =
        ProtoEnvelope::try_deserialize_with_policy(&bytes, pos_meta, Policy::Lenient);
    assert!(!matches!(
        lenient,
        Err(EnvelopeError::TypeMismatch { .. }) | Err(EnvelopeError::VersionMismatch { .. })
    ));
    assert!(matches!(lenient, Err(EnvelopeError::DeserializeError(_))));
}

#[test]
fn test_lenient_ignores_version_mismatch() {
    let schema = schema();
    let chat_meta = schema.by_name("chat.ChatMessage").unwrap();
    let old_meta = schema.by_name("chat_old.ChatMessage").unwrap();

    let msg = chat::ChatMessage {
        sender: "Alice".to_string(),
        content: "Hi".to_string(),
        timestamp: 1,
    };
    let bytes = ProtoEnvelope::new(&msg, chat_meta).to_bytes();

    // Strict rejects the older schema variant...
    let strict: Result<chat_old::ChatMessage, EnvelopeError> =
        ProtoEnvelope::try_deserialize(&bytes, old_meta);
    assert!(matches!(strict, Err(EnvelopeError::VersionMismatch { .. })));

    // ...while lenient decodes it with protobuf's evolution semantics
    // (the extra `version` field is simply absent).
    let lenient: Result<chat_old::ChatMessage, EnvelopeError> =
        ProtoEnvelope::try_deserialize_with_policy(&bytes, old_meta, Policy::Lenient);
    assert!(lenient.is_ok());
    assert_eq!(lenient.unwrap().sender, "Alice");
}

#[test]
fn test_lenient_backward_compatible_evolution() {
    let schema = schema();
    let chat_meta = schema.by_name("chat.ChatMessage").unwrap();
    let old_meta = schema.by_name("chat_old.ChatMessage").unwrap();

    // Sender uses the newer schema (extra `version` field, value 7).
    let old_msg = chat_old::ChatMessage {
        sender: "Bob".to_string(),
        content: "hi".to_string(),
        timestamp: 2,
        version: 7,
    };
    let bytes = ProtoEnvelope::new(&old_msg, old_meta).to_bytes();

    // Strict rejects: versions differ.
    let strict: Result<chat::ChatMessage, EnvelopeError> =
        ProtoEnvelope::try_deserialize(&bytes, chat_meta);
    assert!(matches!(strict, Err(EnvelopeError::VersionMismatch { .. })));

    // Lenient decodes with protobuf's tolerant evolution: the extra field is
    // dropped and the shared fields are preserved.
    let lenient: Result<chat::ChatMessage, EnvelopeError> =
        ProtoEnvelope::try_deserialize_with_policy(&bytes, chat_meta, Policy::Lenient);
    let decoded = lenient.expect("lenient decode");
    assert_eq!(decoded.sender, "Bob");
    assert_eq!(decoded.content, "hi");
    assert_eq!(decoded.timestamp, 2);
}

#[test]
fn test_lenient_still_rejects_bad_framing() {
    let schema = schema();
    let meta = schema.by_name("chat.ChatMessage").unwrap();

    // Too short for the header.
    let short: Result<chat::ChatMessage, EnvelopeError> =
        ProtoEnvelope::try_deserialize_with_policy(&[0u8; 32], meta, Policy::Lenient);
    assert!(matches!(short, Err(EnvelopeError::NotAnEnvelope(_))));

    // Trailing garbage violates the payload length framing even in lenient mode.
    let msg = chat::ChatMessage {
        sender: "A".to_string(),
        content: "B".to_string(),
        timestamp: 1,
    };
    let mut bytes = ProtoEnvelope::new(&msg, meta).to_bytes();
    bytes.push(0xFF);
    let trailing: Result<chat::ChatMessage, EnvelopeError> =
        ProtoEnvelope::try_deserialize_with_policy(&bytes, meta, Policy::Lenient);
    assert!(matches!(trailing, Err(EnvelopeError::NotAnEnvelope(_))));
}
