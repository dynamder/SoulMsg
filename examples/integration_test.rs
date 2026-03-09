use soul_msg::SmsgEnvelope;
use soul_msg::smsg;
use zenoh_ext::z_serialize;

mod chat_module {
    use super::*;
    #[smsg(category = file, path = "smsg_macro/tests/fixtures/messages.smsg")]
    pub mod chat_msgs {}
}

fn main() {
    let msg = chat_module::chat_msgs::ChatMessage {
        sender: "Alice".to_string(),
        content: "Hello".to_string(),
        timestamp: 123,
    };

    let envelope = SmsgEnvelope::new(msg);
    let serialized = z_serialize(&envelope);

    let received: chat_module::chat_msgs::ChatMessage =
        SmsgEnvelope::try_deserialize(&serialized).unwrap();

    assert_eq!(received.sender, "Alice");

    println!("Message sent and received successfully!");
    println!("Sender: {}", received.sender);
    println!("Content: {}", received.content);
}
