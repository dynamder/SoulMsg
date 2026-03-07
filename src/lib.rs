pub use smsg_macro::smsg;

pub trait MessageMeta {
    fn version_hash() -> [u8; 32];
    fn message_name() -> &'static str;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SmsgEnvelope<T> {
    version_hash: [u8; 32],
    pub payload: T,
}

impl<T: MessageMeta> SmsgEnvelope<T> {
    pub fn new(payload: T) -> Self {
        Self {
            version_hash: T::version_hash(),
            payload,
        }
    }

    pub fn into_parts(self) -> ([u8; 32], T) {
        (self.version_hash, self.payload)
    }

    pub fn into_payload(self) -> T {
        self.payload
    }
    pub fn version_hash(&self) -> &[u8; 32] {
        &self.version_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct TestMessage {
        id: u32,
        name: String,
    }

    impl TestMessage {
        fn new() -> Self {
            Self {
                id: 0,
                name: String::new(),
            }
        }
    }

    impl MessageMeta for TestMessage {
        fn version_hash() -> [u8; 32] {
            [
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
                0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
                0x1d, 0x1e, 0x1f, 0x20,
            ]
        }

        fn message_name() -> &'static str {
            "TestMessage"
        }
    }

    #[test]
    fn test_smsg_envelope_new() {
        let msg = TestMessage::new();
        let envelope = SmsgEnvelope::new(msg);

        assert_eq!(*envelope.version_hash(), TestMessage::version_hash());
        assert_eq!(envelope.payload.id, 0);
        assert_eq!(envelope.payload.name, "");
    }

    #[test]
    fn test_smsg_envelope_new_with_data() {
        let msg = TestMessage {
            id: 42,
            name: "Test".to_string(),
        };
        let envelope = SmsgEnvelope::new(msg);

        assert_eq!(*envelope.version_hash(), TestMessage::version_hash());
        assert_eq!(envelope.payload.id, 42);
        assert_eq!(envelope.payload.name, "Test");
    }

    #[test]
    fn test_smsg_envelope_into_parts() {
        let msg = TestMessage::new();
        let envelope = SmsgEnvelope::new(msg);
        let (hash, payload) = envelope.into_parts();

        assert_eq!(hash, TestMessage::version_hash());
        assert_eq!(payload.id, 0);
    }

    #[test]
    fn test_smsg_envelope_into_payload() {
        let msg = TestMessage {
            id: 100,
            name: "Payload".to_string(),
        };
        let envelope = SmsgEnvelope::new(msg);
        let payload = envelope.into_payload();

        assert_eq!(payload.id, 100);
        assert_eq!(payload.name, "Payload");
    }

    #[test]
    fn test_smsg_envelope_clone() {
        let msg = TestMessage::new();
        let envelope = SmsgEnvelope::new(msg);
        let cloned = envelope.clone();

        assert_eq!(*cloned.version_hash(), *envelope.version_hash());
        assert_eq!(cloned.payload.id, envelope.payload.id);
    }

    #[test]
    fn test_smsg_envelope_debug() {
        let msg = TestMessage::new();
        let envelope = SmsgEnvelope::new(msg);
        let debug_str = format!("{:?}", envelope);

        assert!(debug_str.contains("SmsgEnvelope"));
        assert!(debug_str.contains("version_hash"));
        assert!(debug_str.contains("payload"));
    }

    #[test]
    fn test_message_meta_traits() {
        let hash = TestMessage::version_hash();
        assert_eq!(hash.len(), 32);

        let name = TestMessage::message_name();
        assert_eq!(name, "TestMessage");
    }

    #[test]
    fn test_smsg_envelope_partial_eq() {
        let msg1 = TestMessage {
            id: 1,
            name: "A".to_string(),
        };
        let msg2 = TestMessage {
            id: 1,
            name: "A".to_string(),
        };
        let envelope1 = SmsgEnvelope::new(msg1);
        let envelope2 = SmsgEnvelope::new(msg2);

        assert_eq!(envelope1, envelope2);
    }

    #[test]
    fn test_smsg_envelope_partial_eq_different_payload() {
        let msg1 = TestMessage {
            id: 1,
            name: "A".to_string(),
        };
        let msg2 = TestMessage {
            id: 2,
            name: "B".to_string(),
        };
        let envelope1 = SmsgEnvelope::new(msg1);
        let envelope2 = SmsgEnvelope::new(msg2);

        assert_ne!(envelope1, envelope2);
    }
}
