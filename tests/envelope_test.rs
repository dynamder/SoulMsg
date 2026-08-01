//! End-to-end tests for the soul_msg v2 envelope using hand-derived prost
//! messages and manually built descriptors (no prost-build needed here).

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

#[derive(Clone, PartialEq, Message)]
struct Position {
    #[prost(double, tag = "1")]
    x: f64,
    #[prost(double, tag = "2")]
    y: f64,
    #[prost(double, tag = "3")]
    z: f64,
}

#[derive(Clone, PartialEq, Message)]
struct RobotState {
    #[prost(string, tag = "1")]
    name: String,
    #[prost(message, tag = "2")]
    position: Option<Position>,
    #[prost(int32, tag = "3")]
    status: i32,
}

fn field(name: &str, number: i32, r#type: Type, type_name: &str) -> FieldDescriptorProto {
    FieldDescriptorProto {
        name: Some(name.to_string()),
        number: Some(number),
        r#type: Some(r#type as i32),
        type_name: if type_name.is_empty() {
            None
        } else {
            Some(type_name.to_string())
        },
        ..Default::default()
    }
}

fn file(package: &str, messages: Vec<DescriptorProto>) -> FileDescriptorProto {
    FileDescriptorProto {
        name: Some(format!("{}.proto", package)),
        package: Some(package.to_string()),
        syntax: Some("proto3".to_string()),
        message_type: messages,
        ..Default::default()
    }
}

fn descriptor_set() -> Vec<u8> {
    let chat_msg = DescriptorProto {
        name: Some("ChatMessage".to_string()),
        field: vec![
            field("sender", 1, Type::String, ""),
            field("content", 2, Type::String, ""),
            field("timestamp", 3, Type::Int64, ""),
        ],
        ..Default::default()
    };
    let pos = DescriptorProto {
        name: Some("Position".to_string()),
        field: vec![
            field("x", 1, Type::Double, ""),
            field("y", 2, Type::Double, ""),
            field("z", 3, Type::Double, ""),
        ],
        ..Default::default()
    };
    let robot = DescriptorProto {
        name: Some("RobotState".to_string()),
        field: vec![
            field("name", 1, Type::String, ""),
            field("position", 2, Type::Message, ".chat.Position"),
            field("status", 3, Type::Int32, ""),
        ],
        ..Default::default()
    };
    let set = FileDescriptorSet {
        file: vec![file("chat", vec![chat_msg, pos, robot])],
    };
    set.encode_to_vec()
}

fn schema() -> Schema {
    Schema::from_descriptor_set(&descriptor_set()).unwrap()
}

#[test]
fn test_schema_lookup_and_hashes() {
    let schema = schema();
    assert_eq!(schema.messages.len(), 3);
    let chat = schema.by_name("chat.ChatMessage").unwrap();
    assert_eq!(chat.full_name, "chat.ChatMessage");
    assert_eq!(chat.name_hash, smsg_core::compute_name_hash("ChatMessage"));
    // Same short name -> same name hash lookup by hash works.
    assert!(schema.by_name_hash(&chat.name_hash).is_some());
}

#[test]
fn test_roundtrip_runtime_meta() {
    let schema = schema();
    let meta = schema.by_name("chat.ChatMessage").unwrap();
    let msg = ChatMessage {
        sender: "Alice".to_string(),
        content: "Hello, World!".to_string(),
        timestamp: 1_699_999_999,
    };
    let bytes = Envelope::new(&msg, meta).to_bytes();
    assert_eq!(bytes.len(), soul_msg::HEADER_LEN + msg.encoded_len());

    let decoded: ChatMessage = Envelope::try_deserialize(&bytes, meta).unwrap();
    assert_eq!(decoded, msg);
}

#[test]
fn test_nested_message_roundtrip() {
    let schema = schema();
    let meta = schema.by_name("chat.RobotState").unwrap();
    let msg = RobotState {
        name: "R2-D2".to_string(),
        position: Some(Position {
            x: 1.5,
            y: 2.5,
            z: 3.5,
        }),
        status: 7,
    };
    let bytes = Envelope::new(&msg, meta).to_bytes();
    let decoded: RobotState = Envelope::try_deserialize(&bytes, meta).unwrap();
    assert_eq!(decoded, msg);
}

#[test]
fn test_type_mismatch_strict() {
    let schema = schema();
    let chat = schema.by_name("chat.ChatMessage").unwrap();
    let pos = schema.by_name("chat.Position").unwrap();
    let msg = ChatMessage {
        sender: "A".to_string(),
        content: "B".to_string(),
        timestamp: 1,
    };
    let bytes = Envelope::new(&msg, chat).to_bytes();
    let result: Result<Position, EnvelopeError> = Envelope::try_deserialize(&bytes, pos);
    assert!(matches!(result, Err(EnvelopeError::TypeMismatch { .. })));
}

#[test]
fn test_peek_for_dispatch() {
    let schema = schema();
    let meta = schema.by_name("chat.ChatMessage").unwrap();
    let msg = ChatMessage {
        sender: "A".to_string(),
        content: "B".to_string(),
        timestamp: 1,
    };
    let bytes = Envelope::new(&msg, meta).to_bytes();
    let (name_hash, version_hash) = Envelope::peek(&bytes).unwrap();
    assert_eq!(name_hash, meta.name_hash);
    assert_eq!(version_hash, meta.version_hash);
    assert!(schema.by_name_hash(&name_hash).is_some());
}

#[test]
fn test_lenient_ignores_version_mismatch() {
    let schema = schema();
    let chat = schema.by_name("chat.ChatMessage").unwrap();
    let msg = ChatMessage {
        sender: "Alice".to_string(),
        content: "hi".to_string(),
        timestamp: 1,
    };
    let bytes = Envelope::new(&msg, chat).to_bytes();

    // Corrupt the version hash so strict would reject.
    let mut corrupted = bytes.clone();
    corrupted[40] ^= 0xFF;

    let strict: Result<ChatMessage, EnvelopeError> = Envelope::try_deserialize(&corrupted, chat);
    assert!(matches!(strict, Err(EnvelopeError::VersionMismatch { .. })));

    let lenient: Result<ChatMessage, EnvelopeError> =
        Envelope::try_deserialize_with_policy(&corrupted, chat, Policy::Lenient);
    assert_eq!(lenient.unwrap(), msg);
}

#[test]
fn test_bad_framing_rejected_in_both_modes() {
    let schema = schema();
    let chat = schema.by_name("chat.ChatMessage").unwrap();
    let msg = ChatMessage {
        sender: "A".to_string(),
        content: "B".to_string(),
        timestamp: 1,
    };
    let mut bytes = Envelope::new(&msg, chat).to_bytes();
    bytes.push(0xFF);

    let strict: Result<ChatMessage, EnvelopeError> = Envelope::try_deserialize(&bytes, chat);
    assert!(matches!(strict, Err(EnvelopeError::NotAnEnvelope(_))));
    let lenient: Result<ChatMessage, EnvelopeError> =
        Envelope::try_deserialize_with_policy(&bytes, chat, Policy::Lenient);
    assert!(matches!(lenient, Err(EnvelopeError::NotAnEnvelope(_))));
}
