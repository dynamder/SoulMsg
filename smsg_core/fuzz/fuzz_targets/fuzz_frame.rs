#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Envelope parsing must never panic on arbitrary bytes.
    let _ = smsg_core::read_header(data);
    let _ = smsg_core::peek(data);
});
