#![no_main]
use libfuzzer_sys::fuzz_target;
use prost::Message;

fuzz_target!(|data: &[u8]| {
    // Decode arbitrary bytes as a descriptor set and hash every message it
    // contains; the canonical encoding must never panic or over-read.
    if let Ok(set) = prost_types::FileDescriptorSet::decode(data) {
        for file in &set.file {
            for msg in &file.message_type {
                let full = smsg_core::full_message_name(file.package(), msg.name());
                let _ = smsg_core::compute_name_hash(msg.name());
                let _ = smsg_core::compute_message_version_hash(msg, &full);
            }
        }
    }
});
