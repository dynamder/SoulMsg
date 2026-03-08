use soul_msg::{MessageMeta, SmsgEnvelope, EnvelopeError};
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

mod envelope_basic_tests {
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

mod network_transmission_simulation_tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_simulate_network_transmission_with_payload_integrity() {
        let original_msg = test_messages::chat_msgs::ChatMessage {
            sender: "NetworkTest".to_string(),
            content: "Testing payload integrity over simulated network".to_string(),
            timestamp: 1699999999,
        };

        let envelope = SmsgEnvelope::new(original_msg.clone());

        let serialized = z_serialize(&envelope);
        let bytes = serialized.to_bytes();
        
        let cursor = Cursor::new(bytes);
        let received_bytes: Vec<u8> = cursor.get_ref().to_vec();
        
        let received_zbytes = zenoh::bytes::ZBytes::from(received_bytes);
        let deserialized: SmsgEnvelope<test_messages::chat_msgs::ChatMessage> = z_deserialize(&received_zbytes).unwrap();

        assert_eq!(deserialized.payload.sender, original_msg.sender);
        assert_eq!(deserialized.payload.content, original_msg.content);
        assert_eq!(deserialized.payload.timestamp, original_msg.timestamp);
    }

    #[test]
    fn test_simulate_packet_loss_recovery() {
        let msg = test_messages::chat_msgs::ChatMessage {
            sender: "PacketTest".to_string(),
            content: "Test message".to_string(),
            timestamp: 123,
        };

        let envelope = SmsgEnvelope::new(msg.clone());
        
        let (vhash, nhash, payload) = envelope.into_parts();
        let serialized_payload = z_serialize(&payload);
        let payload_bytes = serialized_payload.to_bytes();
        
        let mut tx_data = Vec::new();
        tx_data.extend_from_slice(&nhash);
        tx_data.extend_from_slice(&vhash);
        tx_data.extend_from_slice(&payload_bytes);

        let result = SmsgEnvelope::<test_messages::chat_msgs::ChatMessage>::try_deserialize(tx_data.clone());
        assert!(result.is_ok());

        let corrupted_data = {
            let mut corrupted = tx_data.clone();
            corrupted[0] = corrupted[0].wrapping_add(1);
            corrupted
        };
        
        let corrupted_result = SmsgEnvelope::<test_messages::chat_msgs::ChatMessage>::try_deserialize(corrupted_data);
        match corrupted_result {
            Ok(_) => panic!("Corrupted data should not deserialize successfully"),
            Err(EnvelopeError::TypeMismatch { .. }) => {}
            Err(_) => {}
        }
    }

    #[test]
    fn test_sequential_message_handling() {
        let messages = vec![
            test_messages::chat_msgs::ChatMessage {
                sender: "User1".to_string(),
                content: "First message".to_string(),
                timestamp: 1000,
            },
            test_messages::chat_msgs::ChatMessage {
                sender: "User2".to_string(),
                content: "Second message".to_string(),
                timestamp: 2000,
            },
            test_messages::chat_msgs::ChatMessage {
                sender: "User3".to_string(),
                content: "Third message".to_string(),
                timestamp: 3000,
            },
        ];

        for (i, original) in messages.iter().enumerate() {
            let envelope = SmsgEnvelope::new(original.clone());
            let serialized = z_serialize(&envelope);
            let deserialized: SmsgEnvelope<test_messages::chat_msgs::ChatMessage> = z_deserialize(&serialized).unwrap();
            
            assert_eq!(deserialized.payload.sender, original.sender, "Message {} sender mismatch", i);
            assert_eq!(deserialized.payload.content, original.content, "Message {} content mismatch", i);
            assert_eq!(deserialized.payload.timestamp, original.timestamp, "Message {} timestamp mismatch", i);
        }
    }

    #[test]
    fn test_large_payload_handling() {
        let large_content = "x".repeat(1024 * 100);
        let msg = test_messages::chat_msgs::ChatMessage {
            sender: "LargePayload".to_string(),
            content: large_content,
            timestamp: 999999,
        };

        let envelope = SmsgEnvelope::new(msg.clone());
        let serialized = z_serialize(&envelope);
        
        assert!(serialized.to_bytes().len() > 1024 * 50);
        
        let deserialized: SmsgEnvelope<test_messages::chat_msgs::ChatMessage> = z_deserialize(&serialized).unwrap();
        
        assert_eq!(deserialized.payload.sender, msg.sender);
        assert_eq!(deserialized.payload.content.len(), msg.content.len());
        assert_eq!(deserialized.payload.timestamp, msg.timestamp);
    }

    #[test]
    fn test_empty_payload_handling() {
        let msg = test_messages::chat_msgs::ChatMessage {
            sender: String::new(),
            content: String::new(),
            timestamp: 0,
        };

        let envelope = SmsgEnvelope::new(msg.clone());
        let serialized = z_serialize(&envelope);
        let deserialized: SmsgEnvelope<test_messages::chat_msgs::ChatMessage> = z_deserialize(&serialized).unwrap();

        assert_eq!(deserialized.payload.sender, msg.sender);
        assert_eq!(deserialized.payload.content, msg.content);
    }

    #[test]
    fn test_unicode_payload_handling() {
        let msg = test_messages::chat_msgs::ChatMessage {
            sender: "Unicode用户".to_string(),
            content: "Hello 🌍 你好 🔥".to_string(),
            timestamp: 123456,
        };

        let envelope = SmsgEnvelope::new(msg.clone());
        let serialized = z_serialize(&envelope);
        let deserialized: SmsgEnvelope<test_messages::chat_msgs::ChatMessage> = z_deserialize(&serialized).unwrap();

        assert_eq!(deserialized.payload.sender, msg.sender);
        assert_eq!(deserialized.payload.content, msg.content);
    }

    #[test]
    fn test_special_characters_in_payload() {
        let msg = test_messages::chat_msgs::ChatMessage {
            sender: "Test\"with\nnewlines\tand\ttabs".to_string(),
            content: "Content with\r\nCRLF and 'quotes' and <xml>".to_string(),
            timestamp: 111,
        };

        let envelope = SmsgEnvelope::new(msg.clone());
        let serialized = z_serialize(&envelope);
        let deserialized: SmsgEnvelope<test_messages::chat_msgs::ChatMessage> = z_deserialize(&serialized).unwrap();

        assert_eq!(deserialized.payload.sender, msg.sender);
        assert_eq!(deserialized.payload.content, msg.content);
    }
}

mod version_compatibility_tests {
    use super::*;

    #[test]
    fn test_version_hash_differs_when_schema_changes() {
        let old_msg = test_messages_old::old_chat_msgs::ChatMessage {
            sender: "Test".to_string(),
            content: "Hello".to_string(),
            timestamp: 123,
            version: 1,
        };
        let old_envelope = SmsgEnvelope::new(old_msg);

        let new_msg = test_messages::chat_msgs::ChatMessage {
            sender: "Test".to_string(),
            content: "Hello".to_string(),
            timestamp: 123,
        };
        let new_envelope = SmsgEnvelope::new(new_msg);

        assert_ne!(*old_envelope.version_hash(), *new_envelope.version_hash(), "Version hashes should differ for different schemas");
    }

    #[test]
    fn test_name_hash_same_for_compatible_messages() {
        let old_msg = test_messages_old::old_chat_msgs::ChatMessage {
            sender: "Test".to_string(),
            content: "Hello".to_string(),
            timestamp: 123,
            version: 1,
        };
        let old_envelope = SmsgEnvelope::new(old_msg);

        let new_msg = test_messages::chat_msgs::ChatMessage {
            sender: "Test".to_string(),
            content: "Hello".to_string(),
            timestamp: 123,
        };
        let new_envelope = SmsgEnvelope::new(new_msg);

        assert_eq!(*old_envelope.name_hash(), *new_envelope.name_hash(), "Name hashes should be same for same message type");
    }

    #[test]
    fn test_try_deserialize_version_mismatch() {
        let old_msg = test_messages_old::old_chat_msgs::ChatMessage {
            sender: "Test".to_string(),
            content: "Content".to_string(),
            timestamp: 123,
            version: 1,
        };
        let old_envelope = SmsgEnvelope::new(old_msg);

        let (vhash, nhash, payload) = old_envelope.into_parts();
        let serialized_payload = z_serialize(&payload);
        let payload_bytes = serialized_payload.to_bytes();
        
        let mut tx_data = Vec::new();
        tx_data.extend_from_slice(&nhash);
        tx_data.extend_from_slice(&vhash);
        tx_data.extend_from_slice(&payload_bytes);

        let result = SmsgEnvelope::<test_messages::chat_msgs::ChatMessage>::try_deserialize(tx_data);
        
        match result {
            Err(EnvelopeError::VersionMismatch { .. }) => {}
            _ => panic!("Expected VersionMismatch error"),
        }
    }

    #[test]
    fn test_try_deserialize_with_old_version_stores_hash() {
        let original_msg = test_messages_old::old_chat_msgs::ChatMessage {
            sender: "Old".to_string(),
            content: "Old message".to_string(),
            timestamp: 1000,
            version: 1,
        };

        let envelope = SmsgEnvelope::new(original_msg);
        let stored_version_hash = *envelope.version_hash();
        let stored_name_hash = *envelope.name_hash();

        let new_msg = test_messages::chat_msgs::ChatMessage {
            sender: "Alice".to_string(),
            content: "Old message".to_string(),
            timestamp: 1000,
        };
        let new_envelope = SmsgEnvelope::new(new_msg);

        assert_ne!(stored_version_hash, *new_envelope.version_hash());
        assert_eq!(stored_name_hash, *new_envelope.name_hash());

        assert!(!new_envelope.verify_version(&stored_version_hash));
        assert!(new_envelope.verify_name(&stored_name_hash));
    }
}

mod nested_message_tests {
    use super::*;

    #[test]
    fn test_robot_state_with_nested_position() {
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
        let deserialized: SmsgEnvelope<test_messages::chat_msgs::RobotState> = z_deserialize(&serialized).unwrap();

        assert_eq!(deserialized.payload.name, msg.name);
        assert_eq!(deserialized.payload.position.x, msg.position.x);
        assert_eq!(deserialized.payload.position.y, msg.position.y);
        assert_eq!(deserialized.payload.position.z, msg.position.z);
        assert_eq!(deserialized.payload.status, msg.status);
    }

    #[test]
    fn test_nested_message_hash_uniqueness() {
        let robot_hash = test_messages::chat_msgs::RobotState::version_hash();
        let position_hash = test_messages::chat_msgs::Position::version_hash();
        
        assert_ne!(robot_hash, position_hash);
    }
}

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_verify_with_wrong_hash() {
        let msg = test_messages::chat_msgs::ChatMessage {
            sender: "Test".to_string(),
            content: "Content".to_string(),
            timestamp: 123,
        };
        let envelope = SmsgEnvelope::new(msg);

        let wrong_hash = [0u8; 32];
        assert!(!envelope.verify_version(&wrong_hash));
        assert!(!envelope.verify_name(&wrong_hash));
    }

    #[test]
    fn test_multiple_envelopes_same_hash_different_payloads() {
        let msg1 = test_messages::chat_msgs::ChatMessage {
            sender: "A".to_string(),
            content: "Message A".to_string(),
            timestamp: 100,
        };
        let msg2 = test_messages::chat_msgs::ChatMessage {
            sender: "B".to_string(),
            content: "Message B".to_string(),
            timestamp: 200,
        };

        let envelope1 = SmsgEnvelope::new(msg1);
        let envelope2 = SmsgEnvelope::new(msg2);

        assert_eq!(*envelope1.version_hash(), *envelope2.version_hash());
        assert_eq!(*envelope1.name_hash(), *envelope2.name_hash());
        
        assert_ne!(envelope1.payload.sender, envelope2.payload.sender);
    }

    #[test]
    fn test_all_zero_payload() {
        let msg = test_messages::chat_msgs::ChatMessage {
            sender: String::new(),
            content: String::new(),
            timestamp: 0,
        };

        let envelope = SmsgEnvelope::new(msg.clone());
        
        let (vhash, nhash, payload) = envelope.into_parts();
        let serialized_payload = z_serialize(&payload);
        let payload_bytes = serialized_payload.to_bytes();
        
        let mut tx_data = Vec::new();
        tx_data.extend_from_slice(&nhash);
        tx_data.extend_from_slice(&vhash);
        tx_data.extend_from_slice(&payload_bytes);

        let received = SmsgEnvelope::<test_messages::chat_msgs::ChatMessage>::try_deserialize(tx_data).unwrap();
        
        assert_eq!(received.sender, "");
        assert_eq!(received.content, "");
        assert_eq!(received.timestamp, 0);
    }

    #[test]
    fn test_max_values_payload() {
        let msg = test_messages::chat_msgs::ChatMessage {
            sender: "x".repeat(10000),
            content: "y".repeat(10000),
            timestamp: i64::MAX,
        };

        let envelope = SmsgEnvelope::new(msg.clone());
        let serialized = z_serialize(&envelope);
        let deserialized: SmsgEnvelope<test_messages::chat_msgs::ChatMessage> = z_deserialize(&serialized).unwrap();

        assert_eq!(deserialized.payload.sender.len(), msg.sender.len());
        assert_eq!(deserialized.payload.content.len(), msg.content.len());
        assert_eq!(deserialized.payload.timestamp, i64::MAX);
    }

    #[test]
    fn test_negative_timestamp() {
        let msg = test_messages::chat_msgs::ChatMessage {
            sender: "Test".to_string(),
            content: "Negative timestamp test".to_string(),
            timestamp: -1000000,
        };

        let envelope = SmsgEnvelope::new(msg.clone());
        let serialized = z_serialize(&envelope);
        let deserialized: SmsgEnvelope<test_messages::chat_msgs::ChatMessage> = z_deserialize(&serialized).unwrap();

        assert_eq!(deserialized.payload.timestamp, -1000000);
    }

    #[test]
    fn test_mixed_positive_negative_floats() {
        let pos = test_messages::chat_msgs::Position {
            x: -123.456,
            y: 0.0,
            z: 789.012,
        };

        let envelope = SmsgEnvelope::new(pos.clone());
        let serialized = z_serialize(&envelope);
        let deserialized: SmsgEnvelope<test_messages::chat_msgs::Position> = z_deserialize(&serialized).unwrap();

        assert_eq!(deserialized.payload.x, -123.456);
        assert_eq!(deserialized.payload.y, 0.0);
        assert_eq!(deserialized.payload.z, 789.012);
    }
}

mod error_message_tests {
    use super::*;

    #[test]
    fn test_not_an_envelope_error_message() {
        let result: Result<test_messages::chat_msgs::ChatMessage, EnvelopeError> = 
            SmsgEnvelope::try_deserialize(vec![0u8; 10]);
        
        match result {
            Err(EnvelopeError::NotAnEnvelope(msg)) => {
                assert!(!msg.is_empty());
            }
            _ => panic!("Expected NotAnEnvelope error"),
        }
    }

    #[test]
    fn test_type_mismatch_error_contains_hashes() {
        let chat_msg = test_messages::chat_msgs::ChatMessage {
            sender: "Test".to_string(),
            content: "Content".to_string(),
            timestamp: 123,
        };
        let chat_envelope = SmsgEnvelope::new(chat_msg);

        let pos_msg = test_messages::chat_msgs::Position {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let pos_envelope = SmsgEnvelope::new(pos_msg);

        let chat_name_hash = *chat_envelope.name_hash();
        let pos_name_hash = *pos_envelope.name_hash();
        
        let (vhash, _nhash, payload) = chat_envelope.into_parts();
        let serialized_payload = z_serialize(&payload);
        let payload_bytes = serialized_payload.to_bytes();
        
        let mut tx_data = Vec::new();
        tx_data.extend_from_slice(&pos_name_hash);
        tx_data.extend_from_slice(&vhash);
        tx_data.extend_from_slice(&payload_bytes);

        let result = SmsgEnvelope::<test_messages::chat_msgs::ChatMessage>::try_deserialize(tx_data);
        
        match result {
            Err(EnvelopeError::TypeMismatch { expected_name_hash, actual_name_hash }) => {
                assert_eq!(expected_name_hash, chat_name_hash);
                assert_eq!(actual_name_hash, pos_name_hash);
            }
            _ => panic!("Expected TypeMismatch error"),
        }
    }

    #[test]
    fn test_version_mismatch_error_contains_hashes() {
        let old_msg = test_messages_old::old_chat_msgs::ChatMessage {
            sender: "Test".to_string(),
            content: "Content".to_string(),
            timestamp: 123,
            version: 1,
        };
        let old_envelope = SmsgEnvelope::new(old_msg);

        let new_msg = test_messages::chat_msgs::ChatMessage {
            sender: "Test".to_string(),
            content: "Content".to_string(),
            timestamp: 123,
        };
        let new_envelope = SmsgEnvelope::new(new_msg);

        let old_version_hash = *old_envelope.version_hash();
        let old_name_hash = *old_envelope.name_hash();
        let new_version_hash = *new_envelope.version_hash();
        
        let (vhash, nhash, payload) = old_envelope.into_parts();
        let serialized_payload = z_serialize(&payload);
        let payload_bytes = serialized_payload.to_bytes();
        
        let mut tx_data = Vec::new();
        tx_data.extend_from_slice(&nhash);
        tx_data.extend_from_slice(&vhash);
        tx_data.extend_from_slice(&payload_bytes);
        
        let result = SmsgEnvelope::<test_messages::chat_msgs::ChatMessage>::try_deserialize(tx_data);
        
        match result {
            Err(EnvelopeError::VersionMismatch { expected_version_hash, actual_version_hash }) => {
                assert_eq!(expected_version_hash, new_version_hash);
                assert_eq!(actual_version_hash, old_version_hash);
            }
            _ => panic!("Expected VersionMismatch error"),
        }
    }
}
