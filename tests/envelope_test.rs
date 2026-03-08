use soul_msg::{MessageMeta, SmsgEnvelope};
use zenoh_ext::{z_deserialize, z_serialize};

mod test_messages {
    use soul_msg::smsg;
    
    #[smsg(category = file, path = "smsg_macro/tests/fixtures/messages.smsg")]
    pub mod chat_msgs {}
}

mod envelope_tests {
    use super::*;

    #[test]
    fn test_envelope_new_with_chat_message() {
        let msg = test_messages::chat_msgs::ChatMessage {
            sender: "Alice".to_string(),
            content: "Hello World".to_string(),
            timestamp: 1234567890,
        };

        let envelope = SmsgEnvelope::new(msg);

        assert_eq!(
            *envelope.version_hash(),
            test_messages::chat_msgs::ChatMessage::version_hash()
        );
        assert_eq!(
            *envelope.name_hash(),
            test_messages::chat_msgs::ChatMessage::name_hash()
        );
    }

    #[test]
    fn test_envelope_new_with_position() {
        let pos = test_messages::chat_msgs::Position {
            x: 1.5,
            y: 2.5,
            z: 3.5,
        };

        let envelope = SmsgEnvelope::new(pos);

        assert_eq!(
            *envelope.version_hash(),
            test_messages::chat_msgs::Position::version_hash()
        );
    }

    #[test]
    fn test_envelope_into_parts() {
        let msg = test_messages::chat_msgs::ChatMessage {
            sender: "Bob".to_string(),
            content: "Test".to_string(),
            timestamp: 999,
        };

        let envelope = SmsgEnvelope::new(msg);
        let (version_hash, name_hash, payload) = envelope.into_parts();

        assert_eq!(version_hash, test_messages::chat_msgs::ChatMessage::version_hash());
        assert_eq!(name_hash, test_messages::chat_msgs::ChatMessage::name_hash());
        assert_eq!(payload.sender, "Bob");
    }

    #[test]
    fn test_envelope_into_payload() {
        let msg = test_messages::chat_msgs::ChatMessage {
            sender: "Charlie".to_string(),
            content: "Payload test".to_string(),
            timestamp: 777,
        };

        let envelope = SmsgEnvelope::new(msg);
        let payload = envelope.into_payload();

        assert_eq!(payload.sender, "Charlie");
    }

    #[test]
    fn test_envelope_clone() {
        let msg = test_messages::chat_msgs::ChatMessage {
            sender: "Clone".to_string(),
            content: "Test".to_string(),
            timestamp: 111,
        };

        let envelope1 = SmsgEnvelope::new(msg);
        let envelope2 = envelope1.clone();

        assert_eq!(*envelope1.version_hash(), *envelope2.version_hash());
        assert_eq!(*envelope1.name_hash(), *envelope2.name_hash());
        assert_eq!(envelope1.payload.sender, envelope2.payload.sender);
    }

    #[test]
    fn test_envelope_partial_eq() {
        let msg1 = test_messages::chat_msgs::ChatMessage {
            sender: "Same".to_string(),
            content: "Content".to_string(),
            timestamp: 100,
        };
        let msg2 = test_messages::chat_msgs::ChatMessage {
            sender: "Same".to_string(),
            content: "Content".to_string(),
            timestamp: 100,
        };

        let envelope1 = SmsgEnvelope::new(msg1);
        let envelope2 = SmsgEnvelope::new(msg2);

        assert_eq!(envelope1, envelope2);
    }

    #[test]
    fn test_envelope_partial_eq_different() {
        let msg1 = test_messages::chat_msgs::ChatMessage {
            sender: "First".to_string(),
            content: "Content".to_string(),
            timestamp: 100,
        };
        let msg2 = test_messages::chat_msgs::ChatMessage {
            sender: "Second".to_string(),
            content: "Content".to_string(),
            timestamp: 100,
        };

        let envelope1 = SmsgEnvelope::new(msg1);
        let envelope2 = SmsgEnvelope::new(msg2);

        assert_ne!(envelope1, envelope2);
    }

    #[test]
    fn test_envelope_debug_format() {
        let msg = test_messages::chat_msgs::ChatMessage {
            sender: "Debug".to_string(),
            content: "Test".to_string(),
            timestamp: 123,
        };

        let envelope = SmsgEnvelope::new(msg);
        let debug_str = format!("{:?}", envelope);

        assert!(debug_str.contains("SmsgEnvelope"));
    }

    #[test]
    fn test_message_meta_version_hash_length() {
        let hash = test_messages::chat_msgs::ChatMessage::version_hash();
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_message_meta_message_name() {
        assert_eq!(test_messages::chat_msgs::ChatMessage::message_name(), "ChatMessage");
    }

    #[test]
    fn test_multiple_envelopes_same_type() {
        let msg1 = test_messages::chat_msgs::ChatMessage {
            sender: "First".to_string(),
            content: "Message 1".to_string(),
            timestamp: 1,
        };
        let msg2 = test_messages::chat_msgs::ChatMessage {
            sender: "Second".to_string(),
            content: "Message 2".to_string(),
            timestamp: 2,
        };

        let envelope1 = SmsgEnvelope::new(msg1);
        let envelope2 = SmsgEnvelope::new(msg2);

        assert_eq!(*envelope1.version_hash(), *envelope2.version_hash());
        assert_ne!(envelope1.payload.sender, envelope2.payload.sender);
    }

    #[test]
    fn test_multiple_messages_same_type_same_hash() {
        let hash1 = test_messages::chat_msgs::ChatMessage::version_hash();
        let hash2 = test_messages::chat_msgs::ChatMessage::version_hash();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_messages_have_different_hashes() {
        let chat_hash = test_messages::chat_msgs::ChatMessage::version_hash();
        let pos_hash = test_messages::chat_msgs::Position::version_hash();
        assert_ne!(chat_hash, pos_hash);
    }

    #[test]
    fn test_version_hash_matches_trait_method() {
        let msg = test_messages::chat_msgs::RobotState {
            name: "TestBot".to_string(),
            position: test_messages::chat_msgs::Position {
                x: 10.0,
                y: 20.0,
                z: 30.0,
            },
            status: 5,
        };

        let envelope = SmsgEnvelope::new(msg);
        let expected_hash = test_messages::chat_msgs::RobotState::version_hash();
        let envelope_hash = *envelope.version_hash();

        assert_eq!(envelope_hash, expected_hash);
    }

    #[test]
    fn test_verify_version_method() {
        let msg = test_messages::chat_msgs::ChatMessage {
            sender: "Verify".to_string(),
            content: "Test".to_string(),
            timestamp: 123,
        };

        let envelope = SmsgEnvelope::new(msg);
        let expected_hash = test_messages::chat_msgs::ChatMessage::version_hash();

        assert!(envelope.verify_version(&expected_hash));
    }

    #[test]
    fn test_verify_name_method() {
        let msg = test_messages::chat_msgs::ChatMessage {
            sender: "Verify".to_string(),
            content: "Test".to_string(),
            timestamp: 123,
        };

        let envelope = SmsgEnvelope::new(msg);
        let expected_name_hash = test_messages::chat_msgs::ChatMessage::name_hash();

        assert!(envelope.verify_name(&expected_name_hash));
    }
}

mod envelope_serialize_tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize_trait_roundtrip() {
        let original = test_messages::chat_msgs::ChatMessage {
            sender: "Alice".to_string(),
            content: "Hello, World!".to_string(),
            timestamp: 1234567890,
        };

        let envelope = SmsgEnvelope::new(original.clone());

        let serialized = z_serialize(&envelope);
        let deserialized: SmsgEnvelope<test_messages::chat_msgs::ChatMessage> = z_deserialize(&serialized).unwrap();

        assert_eq!(deserialized.payload.sender, original.sender);
        assert_eq!(deserialized.payload.content, original.content);
        assert_eq!(deserialized.payload.timestamp, original.timestamp);
        assert_eq!(*deserialized.version_hash(), *envelope.version_hash());
        assert_eq!(*deserialized.name_hash(), *envelope.name_hash());
    }

    #[test]
    fn test_serialize_deserialize_preserves_hashes() {
        let msg = test_messages::chat_msgs::Position {
            x: 1.5,
            y: 2.5,
            z: 3.5,
        };

        let envelope = SmsgEnvelope::new(msg);
        let original_version_hash = *envelope.version_hash();
        let original_name_hash = *envelope.name_hash();

        let serialized = z_serialize(&envelope);
        let deserialized: SmsgEnvelope<test_messages::chat_msgs::Position> = z_deserialize(&serialized).unwrap();

        assert_eq!(*deserialized.version_hash(), original_version_hash);
        assert_eq!(*deserialized.name_hash(), original_name_hash);
        assert_eq!(deserialized.payload.x, 1.5);
        assert_eq!(deserialized.payload.y, 2.5);
        assert_eq!(deserialized.payload.z, 3.5);
    }

    #[test]
    fn test_serialize_deserialize_different_message_types() {
        let chat_msg = test_messages::chat_msgs::ChatMessage {
            sender: "Bob".to_string(),
            content: "Test message".to_string(),
            timestamp: 999,
        };
        let chat_envelope = SmsgEnvelope::new(chat_msg);

        let pos_msg = test_messages::chat_msgs::Position {
            x: 10.0,
            y: 20.0,
            z: 30.0,
        };
        let pos_envelope = SmsgEnvelope::new(pos_msg);

        let chat_serialized = z_serialize(&chat_envelope);
        let pos_serialized = z_serialize(&pos_envelope);

        let chat_deserialized: SmsgEnvelope<test_messages::chat_msgs::ChatMessage> = z_deserialize(&chat_serialized).unwrap();
        let pos_deserialized: SmsgEnvelope<test_messages::chat_msgs::Position> = z_deserialize(&pos_serialized).unwrap();

        assert_eq!(chat_deserialized.payload.sender, "Bob");
        assert_eq!(pos_deserialized.payload.x, 10.0);
        assert_ne!(*chat_deserialized.name_hash(), *pos_deserialized.name_hash());
    }
}

mod try_deserialize_tests {
    use soul_msg::EnvelopeError;

    use super::*;

    #[test]
    fn test_try_deserialize_valid() {
        let original = test_messages::chat_msgs::ChatMessage {
            sender: "Test".to_string(),
            content: "Try deserialize".to_string(),
            timestamp: 555,
        };

        let envelope = SmsgEnvelope::new(original);

        let (vhash, nhash, payload) = envelope.into_parts();
        let serialized_payload = z_serialize(&payload);
        let payload_bytes = serialized_payload.to_bytes();
        
        let mut tx_data = Vec::new();
        tx_data.extend_from_slice(&nhash);
        tx_data.extend_from_slice(&vhash);
        tx_data.extend_from_slice(&payload_bytes);

        let received: test_messages::chat_msgs::ChatMessage = SmsgEnvelope::try_deserialize(tx_data).unwrap();
        assert_eq!(received.sender, "Test");
    }

    #[test]
    fn test_try_deserialize_type_mismatch() {
        let chat_msg = test_messages::chat_msgs::ChatMessage {
            sender: "Test".to_string(),
            content: "Content".to_string(),
            timestamp: 123,
        };
        let envelope = SmsgEnvelope::new(chat_msg);

        let (vhash, nhash, payload) = envelope.into_parts();
        let serialized_payload = z_serialize(&payload);
        let payload_bytes = serialized_payload.to_bytes();
        
        let mut tx_data = Vec::new();
        tx_data.extend_from_slice(&nhash);
        tx_data.extend_from_slice(&vhash);
        tx_data.extend_from_slice(&payload_bytes);

        let wrong_msg = test_messages::chat_msgs::Position {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let wrong_envelope = SmsgEnvelope::new(wrong_msg);
        let wrong_nhash = *wrong_envelope.name_hash();

        let mut wrong_data = Vec::new();
        wrong_data.extend_from_slice(&wrong_nhash);
        wrong_data.extend_from_slice(&vhash);
        wrong_data.extend_from_slice(&payload_bytes);

        let result: Result<test_messages::chat_msgs::ChatMessage, EnvelopeError> = SmsgEnvelope::try_deserialize(wrong_data);
        assert!(matches!(result, Err(EnvelopeError::TypeMismatch { .. })));
    }

    #[test]
    fn test_try_deserialize_data_too_short() {
        let result: Result<test_messages::chat_msgs::ChatMessage, EnvelopeError> = SmsgEnvelope::try_deserialize(vec![0u8; 32]);
        assert!(matches!(result, Err(EnvelopeError::NotAnEnvelope(_))));
    }
}
