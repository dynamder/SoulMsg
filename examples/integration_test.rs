use soul_msg::smsg;
use soul_msg::SmsgEnvelope;
use zenoh_ext::z_serialize;

mod chat_module {
    use super::*;
    #[smsg(category = file, path = "smsg_macro/tests/fixtures/messages.smsg")]
    pub mod chat_msgs {}
}

fn main() {
    use chat_module::chat_msgs;

    let msg = chat_msgs::ChatMessage {
        sender: "Alice".to_string(),
        content: "Hello".to_string(),
        timestamp: 123,
    };

    let envelope = SmsgEnvelope::new(msg);

    let (vhash, nhash, payload) = envelope.into_parts();
    let serialized = z_serialize(&payload);
    let payload_bytes = serialized.to_bytes();
    
    let mut tx_data = Vec::new();
    tx_data.extend_from_slice(&nhash);
    tx_data.extend_from_slice(&vhash);
    tx_data.extend_from_slice(&payload_bytes);

    let received: chat_msgs::ChatMessage = SmsgEnvelope::try_deserialize(tx_data).unwrap();
    assert_eq!(received.sender, "Alice");
}
