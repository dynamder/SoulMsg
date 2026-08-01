//! Shared core for SoulMsg: deterministic message hashing from protobuf
//! descriptors and the envelope wire framing.
//!
//! This crate is free of proc-macro, network and zenoh code so it can be shared
//! by the runtime library (`soul_msg`), the proc-macro (`smsg_macro`) and the
//! cross-language verification harness.

pub mod frame;
pub mod hash;

pub use frame::{
    envelope_bytes, peek, read_header, EnvelopeError, EnvelopeHeader, Policy, HEADER_LEN,
};
pub use hash::{
    compute_message_version_hash, compute_name_hash, full_message_name, name_preimage,
    version_preimage,
};
