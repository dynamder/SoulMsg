//! Cross-language FFI layer for SoulMsg (WIP — under evaluation).
//!
//! Only the dynamic schema/value layer is present so far; the C ABI surface is
//! deliberately not wired up pending the protobuf-vs-custom direction decision.

pub mod dynamic;
