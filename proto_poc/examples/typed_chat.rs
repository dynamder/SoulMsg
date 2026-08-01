//! Typed usage of SoulMsg with the `#[smsg]` macro: no runtime schema object needed.
//!
//! The macro reads the descriptor set at compile time and generates `EnvelopeMeta`
//! for every message in the matching package. `prost-build` (see `build.rs`)
//! generates the message types included below.

use smsg_macro::smsg;
use soul_msg::{Envelope, EnvelopeError, Policy};

#[smsg("verification/descriptors.pb")]
pub mod chat {
    include!(concat!(env!("OUT_DIR"), "/chat.rs"));
}

fn main() -> Result<(), EnvelopeError> {
    let msg = chat::ChatMessage {
        sender: "Alice".to_string(),
        content: "Hello, World!".to_string(),
        timestamp: 1_699_999_999,
    };

    // Send: type is the schema — no meta argument.
    let wire = Envelope::new_typed(&msg).to_bytes();

    // Receive: strict by default.
    let back: chat::ChatMessage = Envelope::try_deserialize_typed(&wire)?;
    assert_eq!(msg, back);

    // Receive: lenient.
    let back: chat::ChatMessage =
        Envelope::try_deserialize_with_policy_typed(&wire, Policy::Lenient)?;
    assert_eq!(msg, back);

    println!("typed roundtrip ok ({} bytes)", wire.len());
    Ok(())
}
