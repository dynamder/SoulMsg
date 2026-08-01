use crate::ir::{FieldType, MessageDef};
use blake3::Hasher;

const VERSION_HASH_DOMAIN: &[u8] = b"smsg:version_hash:v1\0";
const NAME_HASH_DOMAIN: &[u8] = b"smsg:name_hash:v1\0";

fn write_segment(hasher: &mut Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u32).to_be_bytes());
    hasher.update(bytes);
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum CompatibilityStatus {
    Match,
    Mismatch,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct MismatchDetail {
    pub message_name: String,
    pub hash1: [u8; 32],
    pub hash2: [u8; 32],
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct CompatibilityReport {
    pub status: CompatibilityStatus,
    pub details: Vec<MismatchDetail>,
}

pub fn compute_message_hash(message: &MessageDef) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(VERSION_HASH_DOMAIN);
    write_segment(&mut hasher, message.name.as_bytes());
    hasher.update(&(message.fields.len() as u32).to_be_bytes());
    for field in &message.fields {
        write_segment(&mut hasher, field.name.as_bytes());
        hash_field_type(&mut hasher, &field.field_type);
    }
    *hasher.finalize().as_bytes()
}

pub fn compute_name_hash(name: &str) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(NAME_HASH_DOMAIN);
    write_segment(&mut hasher, name.as_bytes());
    *hasher.finalize().as_bytes()
}

fn hash_field_type(hasher: &mut Hasher, field_type: &FieldType) {
    match field_type {
        FieldType::Primitive(p) => {
            hasher.update(b"primitive\0");
            write_segment(hasher, p.to_string().as_bytes());
        }
        FieldType::Array(inner, size) => {
            hasher.update(b"array\0");
            hash_field_type(hasher, inner);
            match size {
                Some(s) => {
                    hasher.update(&[1u8]);
                    hasher.update(&(*s as u32).to_be_bytes());
                }
                None => {
                    hasher.update(&[0u8]);
                }
            }
        }
        FieldType::Nested(name) => {
            hasher.update(b"nested\0");
            write_segment(hasher, name.as_bytes());
        }
    }
}

#[allow(dead_code)]
pub fn compare_hashes(hash1: &[u8; 32], hash2: &[u8; 32]) -> bool {
    hash1 == hash2
}

#[allow(dead_code)]
pub fn compare_messages(
    messages1: &[(&str, [u8; 32])],
    messages2: &[(&str, [u8; 32])],
) -> CompatibilityReport {
    let mut details = Vec::new();

    let map1: std::collections::HashMap<&str, [u8; 32]> =
        messages1.iter().map(|(k, v)| (*k, *v)).collect();
    let map2: std::collections::HashMap<&str, [u8; 32]> =
        messages2.iter().map(|(k, v)| (*k, *v)).collect();

    for (name, hash1) in messages1 {
        if let Some(hash2) = map2.get(name) {
            if hash1 != hash2 {
                details.push(MismatchDetail {
                    message_name: name.to_string(),
                    hash1: *hash1,
                    hash2: *hash2,
                });
            }
        } else {
            details.push(MismatchDetail {
                message_name: name.to_string(),
                hash1: *hash1,
                hash2: [0u8; 32],
            });
        }
    }

    for (name, hash2) in messages2 {
        if !map1.contains_key(name) {
            details.push(MismatchDetail {
                message_name: name.to_string(),
                hash1: [0u8; 32],
                hash2: *hash2,
            });
        }
    }

    let status = if details.is_empty() {
        CompatibilityStatus::Match
    } else {
        CompatibilityStatus::Mismatch
    };

    CompatibilityReport { status, details }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{Field, FieldType, PrimitiveType};

    #[test]
    fn test_compute_message_hash_returns_32_bytes() {
        let msg = MessageDef {
            name: "TestMessage".to_string(),
            fields: vec![Field {
                name: "field1".to_string(),
                field_type: FieldType::Primitive(PrimitiveType::String),
                line: 1,
                col: 1,
            }],
            line: 1,
            col: 1,
        };
        let hash = compute_message_hash(&msg);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_different_messages_produce_different_hashes() {
        let msg1 = MessageDef {
            name: "MessageA".to_string(),
            fields: vec![Field {
                name: "field1".to_string(),
                field_type: FieldType::Primitive(PrimitiveType::String),
                line: 1,
                col: 1,
            }],
            line: 1,
            col: 1,
        };
        let msg2 = MessageDef {
            name: "MessageB".to_string(),
            fields: vec![Field {
                name: "field1".to_string(),
                field_type: FieldType::Primitive(PrimitiveType::String),
                line: 1,
                col: 1,
            }],
            line: 1,
            col: 1,
        };
        let hash1 = compute_message_hash(&msg1);
        let hash2 = compute_message_hash(&msg2);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_field_addition_changes_hash() {
        let msg1 = MessageDef {
            name: "TestMessage".to_string(),
            fields: vec![Field {
                name: "field1".to_string(),
                field_type: FieldType::Primitive(PrimitiveType::String),
                line: 1,
                col: 1,
            }],
            line: 1,
            col: 1,
        };
        let msg2 = MessageDef {
            name: "TestMessage".to_string(),
            fields: vec![
                Field {
                    name: "field1".to_string(),
                    field_type: FieldType::Primitive(PrimitiveType::String),
                    line: 1,
                    col: 1,
                },
                Field {
                    name: "field2".to_string(),
                    field_type: FieldType::Primitive(PrimitiveType::Int32),
                    line: 2,
                    col: 1,
                },
            ],
            line: 1,
            col: 1,
        };
        let hash1 = compute_message_hash(&msg1);
        let hash2 = compute_message_hash(&msg2);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_field_removal_changes_hash() {
        let msg1 = MessageDef {
            name: "TestMessage".to_string(),
            fields: vec![
                Field {
                    name: "field1".to_string(),
                    field_type: FieldType::Primitive(PrimitiveType::String),
                    line: 1,
                    col: 1,
                },
                Field {
                    name: "field2".to_string(),
                    field_type: FieldType::Primitive(PrimitiveType::Int32),
                    line: 2,
                    col: 1,
                },
            ],
            line: 1,
            col: 1,
        };
        let msg2 = MessageDef {
            name: "TestMessage".to_string(),
            fields: vec![Field {
                name: "field1".to_string(),
                field_type: FieldType::Primitive(PrimitiveType::String),
                line: 1,
                col: 1,
            }],
            line: 1,
            col: 1,
        };
        let hash1 = compute_message_hash(&msg1);
        let hash2 = compute_message_hash(&msg2);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_field_type_change_changes_hash() {
        let msg1 = MessageDef {
            name: "TestMessage".to_string(),
            fields: vec![Field {
                name: "field1".to_string(),
                field_type: FieldType::Primitive(PrimitiveType::String),
                line: 1,
                col: 1,
            }],
            line: 1,
            col: 1,
        };
        let msg2 = MessageDef {
            name: "TestMessage".to_string(),
            fields: vec![Field {
                name: "field1".to_string(),
                field_type: FieldType::Primitive(PrimitiveType::Int32),
                line: 1,
                col: 1,
            }],
            line: 1,
            col: 1,
        };
        let hash1 = compute_message_hash(&msg1);
        let hash2 = compute_message_hash(&msg2);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_field_name_change_changes_hash() {
        let msg1 = MessageDef {
            name: "TestMessage".to_string(),
            fields: vec![Field {
                name: "fieldA".to_string(),
                field_type: FieldType::Primitive(PrimitiveType::String),
                line: 1,
                col: 1,
            }],
            line: 1,
            col: 1,
        };
        let msg2 = MessageDef {
            name: "TestMessage".to_string(),
            fields: vec![Field {
                name: "fieldB".to_string(),
                field_type: FieldType::Primitive(PrimitiveType::String),
                line: 1,
                col: 1,
            }],
            line: 1,
            col: 1,
        };
        let hash1 = compute_message_hash(&msg1);
        let hash2 = compute_message_hash(&msg2);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_array_type_changes_hash() {
        let msg1 = MessageDef {
            name: "TestMessage".to_string(),
            fields: vec![Field {
                name: "field1".to_string(),
                field_type: FieldType::Primitive(PrimitiveType::String),
                line: 1,
                col: 1,
            }],
            line: 1,
            col: 1,
        };
        let msg2 = MessageDef {
            name: "TestMessage".to_string(),
            fields: vec![Field {
                name: "field1".to_string(),
                field_type: FieldType::Array(
                    Box::new(FieldType::Primitive(PrimitiveType::String)),
                    None,
                ),
                line: 1,
                col: 1,
            }],
            line: 1,
            col: 1,
        };
        let hash1 = compute_message_hash(&msg1);
        let hash2 = compute_message_hash(&msg2);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_nested_type_changes_hash() {
        let msg1 = MessageDef {
            name: "TestMessage".to_string(),
            fields: vec![Field {
                name: "field1".to_string(),
                field_type: FieldType::Primitive(PrimitiveType::String),
                line: 1,
                col: 1,
            }],
            line: 1,
            col: 1,
        };
        let msg2 = MessageDef {
            name: "TestMessage".to_string(),
            fields: vec![Field {
                name: "field1".to_string(),
                field_type: FieldType::Nested("CustomType".to_string()),
                line: 1,
                col: 1,
            }],
            line: 1,
            col: 1,
        };
        let hash1 = compute_message_hash(&msg1);
        let hash2 = compute_message_hash(&msg2);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_name_hash_is_deterministic() {
        assert_eq!(
            compute_name_hash("ChatMessage"),
            compute_name_hash("ChatMessage")
        );
    }

    #[test]
    fn test_name_hash_ignores_structure() {
        let msg_a = MessageDef {
            name: "ChatMessage".to_string(),
            fields: vec![Field {
                name: "sender".to_string(),
                field_type: FieldType::Primitive(PrimitiveType::String),
                line: 1,
                col: 1,
            }],
            line: 1,
            col: 1,
        };
        let msg_b = MessageDef {
            name: "ChatMessage".to_string(),
            fields: vec![Field {
                name: "other".to_string(),
                field_type: FieldType::Primitive(PrimitiveType::Int32),
                line: 1,
                col: 1,
            }],
            line: 1,
            col: 1,
        };
        assert_eq!(
            compute_name_hash(&msg_a.name),
            compute_name_hash(&msg_b.name)
        );
        assert_ne!(compute_message_hash(&msg_a), compute_message_hash(&msg_b));
    }

    #[test]
    fn test_name_hash_domain_separated_from_version_hash() {
        let msg = MessageDef {
            name: "ChatMessage".to_string(),
            fields: vec![Field {
                name: "sender".to_string(),
                field_type: FieldType::Primitive(PrimitiveType::String),
                line: 1,
                col: 1,
            }],
            line: 1,
            col: 1,
        };
        assert_ne!(compute_name_hash(&msg.name), compute_message_hash(&msg));
    }

    #[test]
    fn test_hash_no_boundary_ambiguity() {
        let msg1 = MessageDef {
            name: "M".to_string(),
            fields: vec![Field {
                name: "ab".to_string(),
                field_type: FieldType::Primitive(PrimitiveType::String),
                line: 1,
                col: 1,
            }],
            line: 1,
            col: 1,
        };
        let msg2 = MessageDef {
            name: "Ma".to_string(),
            fields: vec![Field {
                name: "b".to_string(),
                field_type: FieldType::Primitive(PrimitiveType::String),
                line: 1,
                col: 1,
            }],
            line: 1,
            col: 1,
        };
        assert_ne!(compute_message_hash(&msg1), compute_message_hash(&msg2));
    }

    #[test]
    fn test_hash_field_count_distinguishes_empty_from_absent() {
        let msg1 = MessageDef {
            name: "Msg".to_string(),
            fields: vec![],
            line: 1,
            col: 1,
        };
        let msg2 = MessageDef {
            name: "Msg".to_string(),
            fields: vec![Field {
                name: "x".to_string(),
                field_type: FieldType::Primitive(PrimitiveType::String),
                line: 1,
                col: 1,
            }],
            line: 1,
            col: 1,
        };
        assert_ne!(compute_message_hash(&msg1), compute_message_hash(&msg2));
    }

    #[test]
    fn test_array_size_changes_hash() {
        let msg1 = MessageDef {
            name: "Msg".to_string(),
            fields: vec![Field {
                name: "v".to_string(),
                field_type: FieldType::Array(
                    Box::new(FieldType::Primitive(PrimitiveType::Int32)),
                    Some(3),
                ),
                line: 1,
                col: 1,
            }],
            line: 1,
            col: 1,
        };
        let msg2 = MessageDef {
            name: "Msg".to_string(),
            fields: vec![Field {
                name: "v".to_string(),
                field_type: FieldType::Array(
                    Box::new(FieldType::Primitive(PrimitiveType::Int32)),
                    Some(4),
                ),
                line: 1,
                col: 1,
            }],
            line: 1,
            col: 1,
        };
        let msg3 = MessageDef {
            name: "Msg".to_string(),
            fields: vec![Field {
                name: "v".to_string(),
                field_type: FieldType::Array(
                    Box::new(FieldType::Primitive(PrimitiveType::Int32)),
                    None,
                ),
                line: 1,
                col: 1,
            }],
            line: 1,
            col: 1,
        };
        assert_ne!(compute_message_hash(&msg1), compute_message_hash(&msg2));
        assert_ne!(compute_message_hash(&msg1), compute_message_hash(&msg3));
    }

    #[test]
    fn test_compare_hashes_identical() {
        let hash1: [u8; 32] = [0u8; 32];
        let hash2: [u8; 32] = [0u8; 32];
        assert!(compare_hashes(&hash1, &hash2));
    }

    #[test]
    fn test_compare_hashes_different() {
        let hash1: [u8; 32] = [0u8; 32];
        let hash2: [u8; 32] = [1u8; 32];
        assert!(!compare_hashes(&hash1, &hash2));
    }

    #[test]
    fn test_compare_messages_all_match() {
        let messages1 = vec![("Msg1", [0u8; 32]), ("Msg2", [1u8; 32])];
        let messages2 = vec![("Msg1", [0u8; 32]), ("Msg2", [1u8; 32])];
        let report = compare_messages(&messages1, &messages2);
        assert_eq!(report.status, CompatibilityStatus::Match);
        assert!(report.details.is_empty());
    }

    #[test]
    fn test_compare_messages_mismatch() {
        let messages1 = vec![("Msg1", [0u8; 32])];
        let messages2 = vec![("Msg1", [1u8; 32])];
        let report = compare_messages(&messages1, &messages2);
        assert_eq!(report.status, CompatibilityStatus::Mismatch);
        assert_eq!(report.details.len(), 1);
        assert_eq!(report.details[0].message_name, "Msg1");
    }

    #[test]
    fn test_compare_messages_different_count() {
        let messages1 = vec![("Msg1", [0u8; 32]), ("Msg2", [1u8; 32])];
        let messages2 = vec![("Msg1", [0u8; 32])];
        let report = compare_messages(&messages1, &messages2);
        assert_eq!(report.status, CompatibilityStatus::Mismatch);
        assert_eq!(report.details.len(), 1);
    }

    #[test]
    fn test_compare_messages_missing_in_both() {
        let messages1 = vec![("Msg1", [0u8; 32])];
        let messages2 = vec![("Msg2", [1u8; 32])];
        let report = compare_messages(&messages1, &messages2);
        assert_eq!(report.status, CompatibilityStatus::Mismatch);
        assert_eq!(report.details.len(), 2);
    }
}
