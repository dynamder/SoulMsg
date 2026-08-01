#![no_main]
use libfuzzer_sys::fuzz_target;
use prost::Message;
use prost_types::field_descriptor_proto::Type;
use prost_types::{DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet};
use soul_msg::{Envelope, Policy, Schema};

#[derive(Clone, PartialEq, Message)]
struct ChatMessage {
    #[prost(string, tag = "1")]
    sender: String,
    #[prost(string, tag = "2")]
    content: String,
    #[prost(int64, tag = "3")]
    timestamp: i64,
}

fn meta() -> soul_msg::MessageRef {
    fn field(name: &str, number: i32, r#type: Type) -> FieldDescriptorProto {
        FieldDescriptorProto {
            name: Some(name.to_string()),
            number: Some(number),
            r#type: Some(r#type as i32),
            ..Default::default()
        }
    }
    let chat_msg = DescriptorProto {
        name: Some("ChatMessage".to_string()),
        field: vec![
            field("sender", 1, Type::String),
            field("content", 2, Type::String),
            field("timestamp", 3, Type::Int64),
        ],
        ..Default::default()
    };
    let file = FileDescriptorProto {
        name: Some("chat.proto".to_string()),
        package: Some("chat".to_string()),
        syntax: Some("proto3".to_string()),
        message_type: vec![chat_msg],
        ..Default::default()
    };
    let set = FileDescriptorSet { file: vec![file] };
    Schema::from_descriptor_set(&set.encode_to_vec())
        .unwrap()
        .by_name("chat.ChatMessage")
        .unwrap()
        .clone()
}

fuzz_target!(|data: &[u8]| {
    // Envelope decoding must never panic on arbitrary bytes, in strict or lenient
    // mode, with hashes intact or not.
    let m = meta();
    let _ = Envelope::try_deserialize::<ChatMessage>(data, &m);
    let _ = Envelope::try_deserialize_with_policy::<ChatMessage>(data, &m, Policy::Strict);
    let _ = Envelope::try_deserialize_with_policy::<ChatMessage>(data, &m, Policy::Lenient);
    let _ = Envelope::peek(data);
});
