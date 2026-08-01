//! Deterministic message hashing derived from the protobuf file descriptor set.
//!
//! The hashing input is a canonical encoding of the `DescriptorProto` so that any
//! language hashing the same `.proto` (via its descriptor) produces identical
//! hashes — this is the cross-language consistency contract of SoulMsg.

use blake3::Hasher;
use prost_types::DescriptorProto;

pub const NAME_HASH_DOMAIN: &[u8] = b"smsg_proto:name:v1\0";
pub const VERSION_HASH_DOMAIN: &[u8] = b"smsg_proto:version:v1\0";

fn write_seg(hasher: &mut Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u32).to_le_bytes());
    hasher.update(bytes);
}

/// `name_hash` identifies a message type by its short name. It is stable across
/// schema edits, so a same-named message in another package gets a type mismatch
/// only through the version hash.
pub fn compute_name_hash(short_name: &str) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(NAME_HASH_DOMAIN);
    write_seg(&mut hasher, short_name.as_bytes());
    *hasher.finalize().as_bytes()
}

/// `version_hash` captures the full definition: the message's full name plus each
/// field's (name, number, protobuf type, type_name) in declaration order.
pub fn compute_message_version_hash(msg: &DescriptorProto, full_name: &str) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(VERSION_HASH_DOMAIN);
    write_seg(&mut hasher, full_name.as_bytes());
    hasher.update(&(msg.field.len() as u32).to_le_bytes());
    for f in &msg.field {
        write_seg(&mut hasher, f.name().as_bytes());
        hasher.update(&f.number().to_le_bytes());
        let ty = f.r#type() as i32;
        hasher.update(&ty.to_le_bytes());
        if !f.type_name().is_empty() {
            write_seg(&mut hasher, f.type_name().as_bytes());
        }
    }
    *hasher.finalize().as_bytes()
}

/// The full name of a top-level message, given its package and short name.
pub fn full_message_name(package: &str, short_name: &str) -> String {
    if package.is_empty() {
        short_name.to_string()
    } else {
        format!("{}.{}", package, short_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost_types::field_descriptor_proto::Type;
    use prost_types::FieldDescriptorProto;

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

    fn msg(name: &str, fields: Vec<FieldDescriptorProto>) -> DescriptorProto {
        DescriptorProto {
            name: Some(name.to_string()),
            field: fields,
            ..Default::default()
        }
    }

    #[test]
    fn test_name_hash_is_deterministic() {
        assert_eq!(
            compute_name_hash("ChatMessage"),
            compute_name_hash("ChatMessage")
        );
    }

    #[test]
    fn test_name_hash_domain_separated_from_version_hash() {
        let m = msg("ChatMessage", vec![field("sender", 1, Type::String, "")]);
        assert_ne!(
            compute_name_hash("ChatMessage"),
            compute_message_version_hash(&m, "chat.ChatMessage")
        );
    }

    #[test]
    fn test_hash_no_boundary_ambiguity() {
        // "M" + field "ab" must differ from "Ma" + field "b".
        let m1 = msg("M", vec![field("ab", 1, Type::String, "")]);
        let m2 = msg("Ma", vec![field("b", 1, Type::String, "")]);
        assert_ne!(
            compute_message_version_hash(&m1, "M"),
            compute_message_version_hash(&m2, "Ma")
        );
    }

    #[test]
    fn test_added_field_changes_version_hash() {
        let m1 = msg("M", vec![field("a", 1, Type::String, "")]);
        let m2 = msg(
            "M",
            vec![
                field("a", 1, Type::String, ""),
                field("b", 2, Type::Int32, ""),
            ],
        );
        assert_ne!(
            compute_message_version_hash(&m1, "M"),
            compute_message_version_hash(&m2, "M")
        );
    }

    #[test]
    fn test_field_number_change_changes_hash() {
        let m1 = msg("M", vec![field("a", 1, Type::String, "")]);
        let m2 = msg("M", vec![field("a", 2, Type::String, "")]);
        assert_ne!(
            compute_message_version_hash(&m1, "M"),
            compute_message_version_hash(&m2, "M")
        );
    }

    #[test]
    fn test_full_name_in_package() {
        assert_eq!(full_message_name("chat", "ChatMessage"), "chat.ChatMessage");
        assert_eq!(full_message_name("", "ChatMessage"), "ChatMessage");
    }
}
