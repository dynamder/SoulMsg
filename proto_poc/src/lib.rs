//! Small-scale prototype: protobuf (prost) as the envelope payload instead of
//! the zenoh-ext encoding used by `soul_msg`.
//!
//! Demonstrates:
//! - payload encoded with protobuf (via prost), produced from `.proto` definitions;
//! - `name_hash` (stable across schema edits) and `version_hash` (changes on schema
//!   edits) computed canonically from the protoc file descriptor set, so that any
//!   language hashing the same `.proto` yields the same hashes;
//! - a fixed-layout envelope `[name_hash:32][version_hash:32][protobuf payload]`.

use prost::Message;
use prost_types::FileDescriptorSet;
use std::fmt;

pub mod chat {
    include!(concat!(env!("OUT_DIR"), "/chat.rs"));
}
pub mod chat_old {
    include!(concat!(env!("OUT_DIR"), "/chat_old.rs"));
}

const NAME_HASH_DOMAIN: &[u8] = b"smsg_proto:name:v1\0";
const VERSION_HASH_DOMAIN: &[u8] = b"smsg_proto:version:v1\0";

fn write_seg(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u32).to_le_bytes());
    hasher.update(bytes);
}

pub fn compute_name_hash(full_name: &str) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(NAME_HASH_DOMAIN);
    write_seg(&mut hasher, full_name.as_bytes());
    *hasher.finalize().as_bytes()
}

/// Canonical version hash of a message definition: full name + each field's
/// (name, number, protobuf type, type_name) in declaration order.
pub fn compute_message_version_hash(
    msg: &prost_types::DescriptorProto,
    full_name: &str,
) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
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

/// The fixed-layout envelope header:
/// `name_hash:32 + version_hash:32 + payload_len:u32(LE)`.
pub const HEADER_LEN: usize = 68;

#[derive(Debug, Clone, PartialEq)]
pub enum EnvelopeError {
    NotAnEnvelope(String),
    TypeMismatch {
        expected_name_hash: [u8; 32],
        actual_name_hash: [u8; 32],
    },
    VersionMismatch {
        expected_version_hash: [u8; 32],
        actual_version_hash: [u8; 32],
    },
    DeserializeError(String),
}

impl fmt::Display for EnvelopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnvelopeError::NotAnEnvelope(m) => write!(f, "Not an envelope: {}", m),
            EnvelopeError::TypeMismatch {
                expected_name_hash,
                actual_name_hash,
            } => write!(
                f,
                "Type mismatch: expected {:02x?}, got {:02x?}",
                expected_name_hash, actual_name_hash
            ),
            EnvelopeError::VersionMismatch {
                expected_version_hash,
                actual_version_hash,
            } => write!(
                f,
                "Version mismatch: expected {:02x?}, got {:02x?}",
                expected_version_hash, actual_version_hash
            ),
            EnvelopeError::DeserializeError(m) => write!(f, "Deserialize error: {}", m),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MessageRef {
    pub full_name: String,
    pub name_hash: [u8; 32],
    pub version_hash: [u8; 32],
}

pub struct ProtoSchema {
    pub messages: Vec<MessageRef>,
}

impl ProtoSchema {
    pub fn from_descriptor_bytes(bytes: &[u8]) -> Result<Self, String> {
        let set = FileDescriptorSet::decode(bytes).map_err(|e| e.to_string())?;
        let mut messages = Vec::new();
        for file in &set.file {
            let package = file.package();
            for msg in &file.message_type {
                let short_name = msg.name();
                let full_name = if package.is_empty() {
                    short_name.to_string()
                } else {
                    format!("{}.{}", package, short_name)
                };
                messages.push(MessageRef {
                    full_name: full_name.clone(),
                    // Mirror soul_msg semantics: the name hash identifies the message
                    // by its short name, so a same-named message in a different package
                    // still gets a TypeMismatch only via the version hash.
                    name_hash: compute_name_hash(short_name),
                    version_hash: compute_message_version_hash(msg, &full_name),
                });
            }
        }
        Ok(ProtoSchema { messages })
    }

    pub fn by_name(&self, full_name: &str) -> Option<&MessageRef> {
        self.messages.iter().find(|m| m.full_name == full_name)
    }
}

/// Loads the schema compiled into this crate (all `.proto` files in `proto/`).
pub fn default_schema() -> ProtoSchema {
    ProtoSchema::from_descriptor_bytes(include_bytes!(concat!(env!("OUT_DIR"), "/descriptors.pb")))
        .expect("built-in descriptor set must be valid")
}

/// Decode-time schema compatibility policy.
///
/// - `Strict` (default): name/version hashes must match the expected message; a
///   mismatch is rejected with `TypeMismatch` / `VersionMismatch`.
/// - `Lenient`: hashes are ignored; the payload is decoded with protobuf's
///   tolerant evolution semantics (unknown/extra fields skipped, missing fields
///   defaulted). Structural framing (header size, payload length prefix) is
///   still enforced in both modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    Strict,
    Lenient,
}

impl Default for Policy {
    fn default() -> Self {
        Policy::Strict
    }
}

/// A message wrapped with its schema-derived hashes. The payload is protobuf-encoded.
pub struct ProtoEnvelope {
    name_hash: [u8; 32],
    version_hash: [u8; 32],
    payload: Vec<u8>,
}

impl ProtoEnvelope {
    pub fn new<T: prost::Message>(msg: &T, meta: &MessageRef) -> Self {
        Self {
            name_hash: meta.name_hash,
            version_hash: meta.version_hash,
            payload: msg.encode_to_vec(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(HEADER_LEN + self.payload.len());
        out.extend_from_slice(&self.name_hash);
        out.extend_from_slice(&self.version_hash);
        out.extend_from_slice(&(self.payload.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.payload);
        out
    }

    /// Verifies the name/version hashes against the expected message, validates
    /// the payload length framing, then decodes the protobuf payload.
    ///
    /// Equivalent to [`ProtoEnvelope::try_deserialize_with_policy`] with
    /// [`Policy::Strict`].
    pub fn try_deserialize<T: prost::Message + Default>(
        bytes: &[u8],
        meta: &MessageRef,
    ) -> Result<T, EnvelopeError> {
        Self::try_deserialize_with_policy(bytes, meta, Policy::Strict)
    }

    /// Decodes an envelope under the given [`Policy`].
    ///
    /// Structural framing (header size, payload length prefix, trailing bytes) is
    /// always validated; only the hash verification is conditional on the policy.
    pub fn try_deserialize_with_policy<T: prost::Message + Default>(
        bytes: &[u8],
        meta: &MessageRef,
        policy: Policy,
    ) -> Result<T, EnvelopeError> {
        if bytes.len() < HEADER_LEN {
            return Err(EnvelopeError::NotAnEnvelope(
                "Data too short for the envelope header.".to_string(),
            ));
        }

        if policy == Policy::Strict {
            let mut actual_name = [0u8; 32];
            actual_name.copy_from_slice(&bytes[0..32]);
            if actual_name != meta.name_hash {
                return Err(EnvelopeError::TypeMismatch {
                    expected_name_hash: meta.name_hash,
                    actual_name_hash: actual_name,
                });
            }

            let mut actual_version = [0u8; 32];
            actual_version.copy_from_slice(&bytes[32..64]);
            if actual_version != meta.version_hash {
                return Err(EnvelopeError::VersionMismatch {
                    expected_version_hash: meta.version_hash,
                    actual_version_hash: actual_version,
                });
            }
        }

        let plen = u32::from_le_bytes(bytes[64..68].try_into().unwrap()) as usize;
        if bytes.len() != HEADER_LEN + plen {
            return Err(EnvelopeError::NotAnEnvelope(format!(
                "Payload length mismatch: header says {} bytes but {} remain",
                plen,
                bytes.len() - HEADER_LEN
            )));
        }

        T::decode(&bytes[HEADER_LEN..]).map_err(|e| EnvelopeError::DeserializeError(e.to_string()))
    }
}
