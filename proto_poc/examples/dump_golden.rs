//! Dumps golden envelope/payload/hash bytes (hex) for fixed messages so other
//! languages can verify byte-for-byte consistency against the Rust implementation.
//!
//! Usage: cargo run -p proto_poc --example dump_golden > verification/golden.json

use prost::Message;
use proto_poc::{chat, default_schema, ProtoEnvelope};
use serde_json::json;

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

fn emit<T: Message>(
    schema: &proto_poc::ProtoSchema,
    full_name: &str,
    msg: &T,
) -> serde_json::Value {
    let meta = schema.by_name(full_name).expect(full_name);
    let envelope = ProtoEnvelope::new(msg, meta).to_bytes();
    let payload = msg.encode_to_vec();
    json!({
        "name_hash": hex(&meta.name_hash),
        "version_hash": hex(&meta.version_hash),
        "payload": hex(&payload),
        "envelope": hex(&envelope),
    })
}

fn main() {
    let schema = default_schema();

    let chat_msg = chat::ChatMessage {
        sender: "Alice".to_string(),
        content: "Hello, World!".to_string(),
        timestamp: 1_699_999_999,
    };
    let robot = chat::RobotState {
        name: "R2-D2".to_string(),
        position: Some(chat::Position {
            x: 100.5,
            y: 200.5,
            z: 300.5,
        }),
        status: 42,
    };

    let doc = json!({
        "spec": "smsg_proto envelope: [name_hash:32][version_hash:32][payload_len:u32le][payload]",
        "messages": {
            "chat.ChatMessage": emit(&schema, "chat.ChatMessage", &chat_msg),
            "chat.RobotState": emit(&schema, "chat.RobotState", &robot),
        }
    });

    println!("{}", serde_json::to_string_pretty(&doc).unwrap());
}
