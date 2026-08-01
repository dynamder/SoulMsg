//! Cross-language verification crate for SoulMsg's protobuf payload.
//!
//! The envelope/hash logic lives in `soul_msg`; the message types are generated at
//! compile time by the `#[smsg]` macro (no protoc, no build script). This crate
//! keeps the shared `.proto` fixtures, the golden baseline and the cross-language
//! verifiers.

pub use soul_msg::{
    Envelope as ProtoEnvelope, EnvelopeError, MessageRef, Policy, Schema as ProtoSchema, HEADER_LEN,
};

use smsg_macro::smsg;

#[smsg("proto/chat.proto")]
pub mod chat {}

#[smsg("proto/chat_old.proto")]
pub mod chat_old {}

/// Loads the schema from the committed descriptor set used by the cross-language
/// verifiers (`verification/descriptors.pb`).
pub fn default_schema() -> ProtoSchema {
    ProtoSchema::from_descriptor_set(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/verification/descriptors.pb"
    )))
    .expect("committed descriptor set must be valid")
}
