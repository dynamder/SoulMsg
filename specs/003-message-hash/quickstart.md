# Quickstart: Message Hash Verification

## Add blake3 Dependency

```toml
# Cargo.toml
[dependencies]
blake3 = "1.5"
```

## Generate Message with Hash

```rust
use soul_msg::smsg;

// Define message in .smsg file
#[smsg("messages/my_message.smsg")]
mod my_message {}
```

## Access Hash at Runtime

```rust
use my_message::{MyMessage, MessageMeta};

// Get the version hash (computed at compile-time)
let hash = MyMessage::version_hash();
println!("Hash: {:x?}", hash);

// Create envelope with hash
use my_message::SmsgEnvelope;

let envelope = SmsgEnvelope::new(MyMessage { 
    field: "hello".to_string() 
});

// Hash is automatically included
assert_eq!(envelope.version_hash, MyMessage::version_hash());
```

## Verify Compatibility

```rust
fn check_compatibility<T: MessageMeta, U: MessageMeta>() -> bool {
    T::version_hash() == U::version_hash()
}
```

## Run Tests

```bash
cargo test
cargo clippy
```
