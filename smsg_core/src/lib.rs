//! Shared core for SoulMsg: `.smsg` intermediate representation, parser, and
//! deterministic message hashing.
//!
//! This crate is intentionally free of proc-macro and network code so that it can
//! be reused by both the Rust proc-macro code generator (`smsg_macro`) and the
//! cross-language FFI layer (`smsg_ffi`).

pub mod error;
pub mod hash;
pub mod ir;
pub mod parser;

pub use error::SmsgParseError;
pub use ir::SmsgFile;
pub use parser::parse_smsg;
