//! Integration tests for the `#[smsg("path/to/file.proto")]` macro: it must
//! parse `.proto` at compile time (no protoc, no build script) and generate
//! prost structs + `EnvelopeMeta` impls that match the runtime hash algorithm.

use smsg_macro::smsg;
use soul_msg::{Envelope, EnvelopeError, EnvelopeMeta, Policy};

#[smsg("tests/fixtures/chat.proto")]
pub mod chat {}

#[smsg("tests/fixtures/chat_old.proto")]
pub mod chat_old {}

/// Recomputes the expected (full_name, name_hash, version_hash) for a message by
/// parsing the same `.proto` with protox and running the canonical hashing.
fn expected_meta(path: &str, package: &str, short: &str) -> (String, [u8; 32], [u8; 32]) {
    let dir = std::path::Path::new(path).parent().unwrap();
    let set = protox::compile([path], [dir]).expect("protox compile");
    let file = set
        .file
        .iter()
        .find(|f| f.package() == package)
        .expect("file by package");
    let msg = file
        .message_type
        .iter()
        .find(|m| m.name() == short)
        .expect("message by name");
    let full = smsg_core::full_message_name(package, short);
    (
        full.clone(),
        smsg_core::compute_name_hash(short),
        smsg_core::compute_message_version_hash(msg, &full),
    )
}

#[test]
fn meta_matches_runtime_algorithm() {
    let (full, nh, vh) = expected_meta("tests/fixtures/chat.proto", "chat", "ChatMessage");
    assert_eq!(chat::ChatMessage::FULL_NAME, full);
    assert_eq!(chat::ChatMessage::NAME_HASH, nh);
    assert_eq!(chat::ChatMessage::VERSION_HASH, vh);

    let (full, nh, vh) = expected_meta("tests/fixtures/chat.proto", "chat", "RobotState");
    assert_eq!(chat::RobotState::FULL_NAME, full);
    assert_eq!(chat::RobotState::NAME_HASH, nh);
    assert_eq!(chat::RobotState::VERSION_HASH, vh);
}

#[test]
fn typed_roundtrip_without_schema() {
    let msg = chat::ChatMessage {
        sender: "Alice".to_string(),
        content: "Hello, World!".to_string(),
        timestamp: 1_699_999_999,
    };
    let bytes = Envelope::new_typed(&msg).to_bytes();
    let decoded: chat::ChatMessage = Envelope::try_deserialize_typed(&bytes).unwrap();
    assert_eq!(decoded, msg);
}

#[test]
fn typed_nested_roundtrip() {
    let msg = chat::RobotState {
        name: "R2-D2".to_string(),
        position: Some(chat::Position {
            x: 100.5,
            y: 200.5,
            z: 300.5,
        }),
        status: 42,
    };
    let bytes = Envelope::new_typed(&msg).to_bytes();
    let decoded: chat::RobotState = Envelope::try_deserialize_typed(&bytes).unwrap();
    assert_eq!(decoded, msg);
}

#[test]
fn typed_strict_rejects_type_mismatch() {
    let pos = chat::Position {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    let bytes = Envelope::new_typed(&pos).to_bytes();
    let result: Result<chat::ChatMessage, EnvelopeError> = Envelope::try_deserialize_typed(&bytes);
    assert!(matches!(result, Err(EnvelopeError::TypeMismatch { .. })));
}

#[test]
fn lenient_tolerates_version_evolution() {
    // Same message name, extra field in the older variant -> version differs.
    let msg = chat::ChatMessage {
        sender: "Alice".to_string(),
        content: "hi".to_string(),
        timestamp: 1,
    };
    let bytes = Envelope::new_typed(&msg).to_bytes();

    let strict: Result<chat_old::ChatMessage, EnvelopeError> =
        Envelope::try_deserialize_typed(&bytes);
    assert!(matches!(strict, Err(EnvelopeError::VersionMismatch { .. })));

    let lenient: Result<chat_old::ChatMessage, EnvelopeError> =
        Envelope::try_deserialize_with_policy_typed(&bytes, Policy::Lenient);
    assert!(lenient.is_ok());
}
