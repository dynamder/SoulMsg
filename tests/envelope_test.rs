use soul_msg::{EnvelopeError, MessageMeta, SmsgEnvelope};
use zenoh_ext::{z_deserialize, z_serialize};

mod test_messages {
    use soul_msg::smsg;

    #[smsg(category = file, path = "smsg_macro/tests/fixtures/messages.smsg")]
    pub mod chat_msgs {}
}

mod test_messages_old {
    use soul_msg::smsg;

    #[smsg(category = file, path = "smsg_macro/tests/fixtures/messages_old.smsg")]
    pub mod old_chat_msgs {}
}

mod unit_tests {
    use super::*;

    #[test]
    fn test_version_hash_is_deterministic() {
        let hash1 = test_messages::chat_msgs::ChatMessage::version_hash();
        let hash2 = test_messages::chat_msgs::ChatMessage::version_hash();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_message_types_have_different_hashes() {
        let chat_hash = test_messages::chat_msgs::ChatMessage::version_hash();
        let pos_hash = test_messages::chat_msgs::Position::version_hash();
        assert_ne!(chat_hash, pos_hash);
    }

    #[test]
    fn test_name_hash_identifies_message_type() {
        let chat_name = test_messages::chat_msgs::ChatMessage::name_hash();
        let pos_name = test_messages::chat_msgs::Position::name_hash();
        assert_ne!(chat_name, pos_name);
    }

    #[test]
    fn test_version_hash_changes_when_schema_changes() {
        let old_hash = test_messages_old::old_chat_msgs::ChatMessage::version_hash();
        let new_hash = test_messages::chat_msgs::ChatMessage::version_hash();
        assert_ne!(
            old_hash, new_hash,
            "Version hash should change when schema changes"
        );
    }

    #[test]
    fn test_name_hash_stable_across_versions() {
        let old_name = test_messages_old::old_chat_msgs::ChatMessage::name_hash();
        let new_name = test_messages::chat_msgs::ChatMessage::name_hash();
        assert_eq!(
            old_name, new_name,
            "Name hash should be stable for same message type"
        );
    }

    #[test]
    fn test_verify_version_returns_true_for_matching() {
        let msg = test_messages::chat_msgs::ChatMessage {
            sender: "Alice".to_string(),
            content: "Hello".to_string(),
            timestamp: 123,
        };
        let envelope = SmsgEnvelope::new(msg);
        let expected = test_messages::chat_msgs::ChatMessage::version_hash();
        assert!(envelope.verify_version(&expected));
    }

    #[test]
    fn test_verify_version_returns_false_for_mismatch() {
        let msg = test_messages::chat_msgs::ChatMessage {
            sender: "Alice".to_string(),
            content: "Hello".to_string(),
            timestamp: 123,
        };
        let envelope = SmsgEnvelope::new(msg);
        let wrong = test_messages::chat_msgs::Position::version_hash();
        assert!(!envelope.verify_version(&wrong));
    }

    #[test]
    fn test_verify_name_returns_true_for_matching() {
        let msg = test_messages::chat_msgs::ChatMessage {
            sender: "Bob".to_string(),
            content: "Test".to_string(),
            timestamp: 456,
        };
        let envelope = SmsgEnvelope::new(msg);
        let expected = test_messages::chat_msgs::ChatMessage::name_hash();
        assert!(envelope.verify_name(&expected));
    }

    #[test]
    fn test_verify_name_returns_false_for_mismatch() {
        let msg = test_messages::chat_msgs::ChatMessage {
            sender: "Bob".to_string(),
            content: "Test".to_string(),
            timestamp: 456,
        };
        let envelope = SmsgEnvelope::new(msg);
        let wrong = test_messages::chat_msgs::Position::name_hash();
        assert!(!envelope.verify_name(&wrong));
    }

    #[test]
    fn test_serialize_deserialize_roundtrip_preserves_data() {
        let original = test_messages::chat_msgs::ChatMessage {
            sender: "Alice".to_string(),
            content: "Hello World!".to_string(),
            timestamp: 1234567890,
        };

        let envelope = SmsgEnvelope::new(original.clone());
        let serialized = z_serialize(&envelope);
        let deserialized: SmsgEnvelope<test_messages::chat_msgs::ChatMessage> =
            z_deserialize(&serialized).unwrap();

        assert_eq!(deserialized.payload, original);
        assert_eq!(*deserialized.version_hash(), *envelope.version_hash());
        assert_eq!(*deserialized.name_hash(), *envelope.name_hash());
    }

    #[test]
    fn test_into_parts_extracts_all_components() {
        let msg = test_messages::chat_msgs::ChatMessage {
            sender: "Test".to_string(),
            content: "Content".to_string(),
            timestamp: 999,
        };

        let envelope = SmsgEnvelope::new(msg);
        let (vhash, nhash, payload) = envelope.into_parts();

        assert_eq!(vhash, test_messages::chat_msgs::ChatMessage::version_hash());
        assert_eq!(nhash, test_messages::chat_msgs::ChatMessage::name_hash());
        assert_eq!(payload.sender, "Test");
    }

    #[test]
    fn test_into_payload_returns_inner_message() {
        let msg = test_messages::chat_msgs::Position {
            x: 1.5,
            y: 2.5,
            z: 3.5,
        };

        let envelope = SmsgEnvelope::new(msg);
        let payload = envelope.into_payload();

        assert_eq!(payload.x, 1.5);
        assert_eq!(payload.y, 2.5);
        assert_eq!(payload.z, 3.5);
    }
}

mod error_handling_tests {
    use super::*;

    #[test]
    fn test_try_deserialize_data_too_short() {
        let result: Result<test_messages::chat_msgs::ChatMessage, EnvelopeError> =
            SmsgEnvelope::try_deserialize(vec![0u8; 32]);
        assert!(matches!(result, Err(EnvelopeError::NotAnEnvelope(_))));
    }

    #[test]
    fn test_try_deserialize_type_mismatch_error() {
        let pos_msg = test_messages::chat_msgs::Position {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let pos_envelope = SmsgEnvelope::new(pos_msg);
        let serialized = z_serialize(&pos_envelope);

        let result: Result<test_messages::chat_msgs::ChatMessage, EnvelopeError> =
            SmsgEnvelope::try_deserialize(serialized);

        assert!(matches!(result, Err(EnvelopeError::TypeMismatch { .. })));
    }

    #[test]
    fn test_try_deserialize_version_mismatch_error() {
        let old_msg = test_messages_old::old_chat_msgs::ChatMessage {
            sender: "Test".to_string(),
            content: "Hello".to_string(),
            timestamp: 123,
            version: 1,
        };
        let old_envelope = SmsgEnvelope::new(old_msg);
        let serialized = z_serialize(&old_envelope);

        let result: Result<test_messages::chat_msgs::ChatMessage, EnvelopeError> =
            SmsgEnvelope::try_deserialize(serialized);

        assert!(matches!(result, Err(EnvelopeError::VersionMismatch { .. })));
    }

    #[test]
    fn test_error_display_includes_details() {
        let err = EnvelopeError::TypeMismatch {
            expected_name_hash: [1u8; 32],
            actual_name_hash: [2u8; 32],
        };
        let err_str = err.to_string();
        assert!(err_str.contains("Type mismatch"));
    }
}

mod nested_message_tests {
    use super::*;

    #[test]
    fn test_nested_struct_serialization() {
        let msg = test_messages::chat_msgs::RobotState {
            name: "R2-D2".to_string(),
            position: test_messages::chat_msgs::Position {
                x: 100.5,
                y: 200.5,
                z: 300.5,
            },
            status: 42,
        };

        let envelope = SmsgEnvelope::new(msg.clone());
        let serialized = z_serialize(&envelope);
        let deserialized: SmsgEnvelope<test_messages::chat_msgs::RobotState> =
            z_deserialize(&serialized).unwrap();

        assert_eq!(deserialized.payload.name, msg.name);
        assert_eq!(deserialized.payload.position.x, msg.position.x);
        assert_eq!(deserialized.payload.status, msg.status);
    }
}
