//! Cross-language verification crate for SoulMsg's protobuf payload.
//!
//! The envelope/hash logic now lives in `soul_msg`; this crate keeps the shared
//! `.proto` fixtures, the prost code generation, the golden baseline and the
//! cross-language verifiers, aliasing the `soul_msg` API for the Rust checks.

pub use soul_msg::{
    Envelope as ProtoEnvelope, EnvelopeError, MessageRef, Policy, Schema as ProtoSchema, HEADER_LEN,
};

pub mod chat {
    include!(concat!(env!("OUT_DIR"), "/chat.rs"));
}
pub mod chat_old {
    include!(concat!(env!("OUT_DIR"), "/chat_old.rs"));
}

/// Loads the schema compiled into this crate (all `.proto` files in `proto/`).
pub fn default_schema() -> ProtoSchema {
    ProtoSchema::from_descriptor_set(include_bytes!(concat!(env!("OUT_DIR"), "/descriptors.pb")))
        .expect("built-in descriptor set must be valid")
}
