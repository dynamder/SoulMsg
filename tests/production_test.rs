use soul_msg::smsg;
use soul_msg::{EnvelopeError, MessageMeta, SmsgEnvelope};
use zenoh_ext::z_serialize;

mod chat_module {
    use super::*;
    #[smsg(category = file, path = "smsg_macro/tests/fixtures/messages.smsg")]
    pub mod chat_msgs {}
}

mod robot_module {
    use super::*;
    #[smsg(category = file, path = "smsg_macro/tests/fixtures/messages_old.smsg")]
    pub mod old_msgs {}
}

#[test]
fn test_production_message_pipeline() {
    let msg = chat_module::chat_msgs::ChatMessage {
        sender: "Alice".to_string(),
        content: "Hello, World!".to_string(),
        timestamp: 1699999999,
    };

    let envelope = SmsgEnvelope::new(msg);
    let serialized = z_serialize(&envelope);

    let received: chat_module::chat_msgs::ChatMessage =
        SmsgEnvelope::try_deserialize(&serialized).unwrap();

    assert_eq!(received.sender, "Alice");
    assert_eq!(received.content, "Hello, World!");
}

#[test]
fn test_production_multiple_message_types() {
    let chat_msg = chat_module::chat_msgs::ChatMessage {
        sender: "Bob".to_string(),
        content: "Test message".to_string(),
        timestamp: 1000,
    };
    let chat_envelope = SmsgEnvelope::new(chat_msg);
    let chat_bytes = z_serialize(&chat_envelope).to_bytes().to_vec();

    let pos_msg = chat_module::chat_msgs::Position {
        x: 10.0,
        y: 20.0,
        z: 30.0,
    };
    let pos_envelope = SmsgEnvelope::new(pos_msg);
    let pos_bytes = z_serialize(&pos_envelope).to_bytes().to_vec();

    let chat_received: chat_module::chat_msgs::ChatMessage =
        SmsgEnvelope::try_deserialize(&zenoh::bytes::ZBytes::from(chat_bytes)).unwrap();
    let pos_received: chat_module::chat_msgs::Position =
        SmsgEnvelope::try_deserialize(&zenoh::bytes::ZBytes::from(pos_bytes)).unwrap();

    assert_eq!(chat_received.sender, "Bob");
    assert_eq!(pos_received.x, 10.0);
}

#[test]
fn test_production_message_broadcasting() {
    let messages = vec![
        chat_module::chat_msgs::ChatMessage {
            sender: "User1".to_string(),
            content: "First broadcast".to_string(),
            timestamp: 1000,
        },
        chat_module::chat_msgs::ChatMessage {
            sender: "User2".to_string(),
            content: "Second broadcast".to_string(),
            timestamp: 2000,
        },
        chat_module::chat_msgs::ChatMessage {
            sender: "User3".to_string(),
            content: "Third broadcast".to_string(),
            timestamp: 3000,
        },
    ];

    let envelopes: Vec<SmsgEnvelope<chat_module::chat_msgs::ChatMessage>> = messages
        .iter()
        .map(|m| SmsgEnvelope::new(m.clone()))
        .collect();

    for (original, envelope) in messages.iter().zip(envelopes.iter()) {
        let serialized = z_serialize(envelope);
        let deserialized: chat_module::chat_msgs::ChatMessage =
            SmsgEnvelope::try_deserialize(&serialized).unwrap();

        assert_eq!(deserialized.sender, original.sender);
        assert_eq!(deserialized.content, original.content);
    }
}

#[test]
fn test_production_robot_state_telemetry() {
    let robot_state = chat_module::chat_msgs::RobotState {
        name: "R2-D2".to_string(),
        position: chat_module::chat_msgs::Position {
            x: 123.45,
            y: 678.90,
            z: 111.11,
        },
        status: 5,
    };

    let envelope = SmsgEnvelope::new(robot_state);
    let serialized = z_serialize(&envelope);
    let transmitted = serialized.to_bytes().to_vec();

    let received: chat_module::chat_msgs::RobotState =
        SmsgEnvelope::try_deserialize(&zenoh::bytes::ZBytes::from(transmitted)).unwrap();

    assert_eq!(received.name, "R2-D2");
    assert_eq!(received.position.x, 123.45);
    assert_eq!(received.status, 5);
}

#[test]
fn test_production_version_migration_handling() {
    let old_msg = robot_module::old_msgs::ChatMessage {
        sender: "Legacy".to_string(),
        content: "Old format".to_string(),
        timestamp: 1000,
        version: 1,
    };
    let old_envelope = SmsgEnvelope::new(old_msg);
    let serialized = z_serialize(&old_envelope);
    let transmitted = serialized.to_bytes().to_vec();

    let received: Result<chat_module::chat_msgs::ChatMessage, EnvelopeError> =
        SmsgEnvelope::try_deserialize(&zenoh::bytes::ZBytes::from(transmitted));

    assert!(matches!(
        received,
        Err(EnvelopeError::VersionMismatch { .. })
    ));
}

#[test]
fn test_production_graceful_error_recovery() {
    let chat_msg = chat_module::chat_msgs::ChatMessage {
        sender: "Test".to_string(),
        content: "Content".to_string(),
        timestamp: 123,
    };
    let envelope = SmsgEnvelope::new(chat_msg);
    let mut corrupted = z_serialize(&envelope).to_bytes().to_vec();

    corrupted.push(0xFF);
    corrupted.push(0xFF);

    let result: Result<chat_module::chat_msgs::ChatMessage, EnvelopeError> =
        SmsgEnvelope::try_deserialize(&zenoh::bytes::ZBytes::from(corrupted));

    assert!(result.is_err());
}

#[test]
fn test_production_empty_and_minimal_messages() {
    let empty_msg = chat_module::chat_msgs::ChatMessage {
        sender: String::new(),
        content: String::new(),
        timestamp: 0,
    };

    let envelope = SmsgEnvelope::new(empty_msg);
    let serialized = z_serialize(&envelope);
    let deserialized: chat_module::chat_msgs::ChatMessage =
        SmsgEnvelope::try_deserialize(&serialized).unwrap();

    assert_eq!(deserialized.sender, "");
    assert_eq!(deserialized.timestamp, 0);
}

#[test]
fn test_production_unicode_and_special_chars() {
    let msg = chat_module::chat_msgs::ChatMessage {
        sender: "Unicode用户🔧".to_string(),
        content: "Hello 🌍 你好 🔥 && \"quotes\"".to_string(),
        timestamp: 123456,
    };

    let envelope = SmsgEnvelope::new(msg.clone());
    let serialized = z_serialize(&envelope);
    let deserialized: chat_module::chat_msgs::ChatMessage =
        SmsgEnvelope::try_deserialize(&serialized).unwrap();

    assert_eq!(deserialized.sender, msg.sender);
    assert_eq!(deserialized.content, msg.content);
}

#[test]
fn test_production_large_payload() {
    let large_content = "x".repeat(1024 * 100);
    let msg = chat_module::chat_msgs::ChatMessage {
        sender: "LargePayload".to_string(),
        content: large_content,
        timestamp: 999999,
    };

    let envelope = SmsgEnvelope::new(msg.clone());
    let serialized = z_serialize(&envelope);

    assert!(serialized.to_bytes().len() > 1024 * 50);

    let deserialized: chat_module::chat_msgs::ChatMessage =
        SmsgEnvelope::try_deserialize(&serialized).unwrap();

    assert_eq!(deserialized.content.len(), msg.content.len());
}

#[test]
fn test_production_type_validation_at_receiver() {
    let pos_msg = chat_module::chat_msgs::Position {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    let envelope = SmsgEnvelope::new(pos_msg);
    let serialized = z_serialize(&envelope);

    let result: Result<chat_module::chat_msgs::ChatMessage, EnvelopeError> =
        SmsgEnvelope::try_deserialize(&serialized);

    assert!(matches!(result, Err(EnvelopeError::TypeMismatch { .. })));
}

#[test]
fn test_production_empty_payload_zst() {
    let empty_msg = chat_module::chat_msgs::EmptyMessage {};

    let envelope = SmsgEnvelope::new(empty_msg);
    let serialized = z_serialize(&envelope);
    let bytes = serialized.to_bytes().to_vec();

    assert!(
        !bytes.is_empty(),
        "Empty message should still produce serialized bytes due to hash headers"
    );

    let received: chat_module::chat_msgs::EmptyMessage =
        SmsgEnvelope::try_deserialize(&zenoh::bytes::ZBytes::from(bytes)).unwrap();

    let _ = received;
}

#[test]
fn test_production_empty_payload_roundtrip() {
    let empty_msg = chat_module::chat_msgs::EmptyMessage {};

    let envelope = SmsgEnvelope::new(empty_msg);
    let serialized = z_serialize(&envelope);
    let transmitted = serialized.to_bytes().to_vec();

    let _received: chat_module::chat_msgs::EmptyMessage =
        SmsgEnvelope::try_deserialize(&zenoh::bytes::ZBytes::from(transmitted)).unwrap();

    assert!(envelope.verify_version(&chat_module::chat_msgs::EmptyMessage::version_hash()));
    assert!(envelope.verify_name(&chat_module::chat_msgs::EmptyMessage::name_hash()));
}
