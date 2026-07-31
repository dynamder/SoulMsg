use smsg_macro::smsg;
use soul_msg::{MessageMeta, SmsgEnvelope};
use zenoh_ext::{z_deserialize, z_serialize};

mod file_type_tests {
    use super::*;

    #[smsg(category = file, path = "tests/fixtures/messages.smsg")]
    pub mod test_messages {}

    #[test]
    fn test_message_type_has_version_hash() {
        let hash = test_messages::ChatMessage::version_hash();
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_message_type_has_name_hash() {
        let hash = test_messages::ChatMessage::name_hash();
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_message_has_correct_name() {
        assert_eq!(test_messages::ChatMessage::message_name(), "ChatMessage");
    }

    #[test]
    fn test_serialization_roundtrip() {
        let msg = test_messages::ChatMessage {
            sender: "Alice".to_string(),
            content: "Hello".to_string(),
            timestamp: 12345,
        };

        let serialized = z_serialize(&msg);
        let deserialized: test_messages::ChatMessage = z_deserialize(&serialized).unwrap();
        assert_eq!(deserialized.sender, "Alice");
    }
}

mod package_type_tests {
    use super::*;

    #[smsg(category = package, path = "tests/fixtures/packages/testpkg")]
    pub mod testpkg {}

    #[test]
    fn test_package_root_message_serialization_roundtrip() {
        let person = testpkg::Person {
            name: "Alice".to_string(),
            age: 25,
            email: "alice@test.com".to_string(),
        };

        let serialized = z_serialize(&person);
        let deserialized: testpkg::Person = z_deserialize(&serialized).unwrap();
        assert_eq!(deserialized.name, "Alice");
        assert_eq!(deserialized.age, 25);
    }

    #[test]
    fn test_package_nested_serialization_roundtrip() {
        let product = testpkg::inventory::Product {
            id: "PROD-001".to_string(),
            name: "Widget".to_string(),
            price: 19.99,
            stock: 100,
        };

        let serialized = z_serialize(&product);
        let deserialized: testpkg::inventory::Product = z_deserialize(&serialized).unwrap();
        assert_eq!(deserialized.id, "PROD-001");
        assert_eq!(deserialized.price, 19.99);
    }
}

mod version_compatibility_tests {
    use super::*;

    #[smsg(category = file, path = "tests/fixtures/messages_old.smsg")]
    pub mod old_messages {}

    #[smsg(category = file, path = "tests/fixtures/messages.smsg")]
    pub mod new_messages {}

    #[test]
    fn test_version_hash_differs_for_different_schemas() {
        let old_hash = old_messages::ChatMessage::version_hash();
        let new_hash = new_messages::ChatMessage::version_hash();
        assert_ne!(old_hash, new_hash);
    }

    #[test]
    fn test_name_hash_same_for_same_message_type() {
        let old_name = old_messages::ChatMessage::name_hash();
        let new_name = new_messages::ChatMessage::name_hash();
        assert_eq!(old_name, new_name);
    }

    #[test]
    fn test_version_verify_rejects_old_version() {
        let msg = new_messages::ChatMessage {
            sender: "Test".to_string(),
            content: "Hello".to_string(),
            timestamp: 123,
        };
        let envelope = SmsgEnvelope::new(msg);

        let old_version = old_messages::ChatMessage::version_hash();
        assert!(!envelope.verify_version(&old_version));
    }

    #[test]
    fn test_name_verify_accepts_compatible_type() {
        let old_msg = old_messages::ChatMessage {
            sender: "Test".to_string(),
            content: "Hello".to_string(),
            timestamp: 123,
            version: 1,
        };
        let envelope = SmsgEnvelope::new(old_msg);

        let new_name = new_messages::ChatMessage::name_hash();
        assert!(envelope.verify_name(&new_name));
    }

    #[test]
    fn test_into_parts_includes_both_hashes() {
        let msg = new_messages::RobotState {
            name: "Bot".to_string(),
            position: new_messages::Position {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            status: 5,
        };
        let envelope = SmsgEnvelope::new(msg);

        let (version_hash, name_hash, payload) = envelope.into_parts();

        assert_eq!(version_hash, new_messages::RobotState::version_hash());
        assert_eq!(name_hash, new_messages::RobotState::name_hash());
        assert_eq!(payload.name, "Bot");
    }
}
