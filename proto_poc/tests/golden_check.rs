//! Guards against drift between the committed cross-language golden fixture and
//! the current Rust implementation. Regenerates the fixed messages and compares
//! name_hash / version_hash / payload / envelope against `verification/golden.json`.

use prost::Message;
use proto_poc::{chat, default_schema, ProtoEnvelope};
use serde_json::Value;

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

fn check<T: Message>(schema: &proto_poc::ProtoSchema, golden: &Value, full_name: &str, msg: &T) {
    let meta = schema.by_name(full_name).unwrap();
    let entry = golden["messages"][full_name].as_object().unwrap();
    assert_eq!(
        hex(&meta.name_hash),
        entry["name_hash"].as_str().unwrap(),
        "{} name_hash",
        full_name
    );
    assert_eq!(
        hex(&meta.version_hash),
        entry["version_hash"].as_str().unwrap(),
        "{} version_hash",
        full_name
    );
    assert_eq!(
        hex(&msg.encode_to_vec()),
        entry["payload"].as_str().unwrap(),
        "{} payload",
        full_name
    );
    assert_eq!(
        hex(&ProtoEnvelope::new(msg, meta).to_bytes()),
        entry["envelope"].as_str().unwrap(),
        "{} envelope",
        full_name
    );
}

#[test]
fn golden_matches_committed_fixture() {
    let schema = default_schema();
    let raw = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/verification/golden.json"
    ))
    .expect("golden.json fixture");
    let golden: Value = serde_json::from_str(&raw).unwrap();

    let chat_msg = chat::ChatMessage {
        sender: "Alice".to_string(),
        content: "Hello, World!".to_string(),
        timestamp: 1_699_999_999,
    };
    check(&schema, &golden, "chat.ChatMessage", &chat_msg);

    let robot = chat::RobotState {
        name: "R2-D2".to_string(),
        position: Some(chat::Position {
            x: 100.5,
            y: 200.5,
            z: 300.5,
        }),
        status: 42,
    };
    check(&schema, &golden, "chat.RobotState", &robot);
}
