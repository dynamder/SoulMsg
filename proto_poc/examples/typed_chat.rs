//! Typed usage of SoulMsg with the `#[smsg]` macro: no build script, no protoc,
//! no `OUT_DIR` — the macro parses the `.proto` at compile time and generates the
//! message types and their `EnvelopeMeta`.

use smsg_macro::smsg;
use soul_msg::{Envelope, EnvelopeError, Policy};

#[smsg("proto/chat.proto")]
pub mod chat {}

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
