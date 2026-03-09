# Quickstart: Message Hash Verification

## Install Dependencies

```toml
# Cargo.toml
[dependencies]
soul_msg = "0.1"
zenoh-ext = "1.7"
zenoh = "1.7"
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
use my_message::{MyMessage, MessageMeta, SmsgEnvelope};

// Get the version hash (computed at compile-time)
let version_hash = MyMessage::version_hash();
let name_hash = MyMessage::name_hash();
println!("Version: {:x?}", version_hash);

// Create envelope with hash
let envelope = SmsgEnvelope::new(MyMessage { 
    field: "hello".to_string() 
});

// Access hashes from envelope
assert_eq!(*envelope.version_hash(), MyMessage::version_hash());
assert_eq!(*envelope.name_hash(), MyMessage::name_hash());
```

## Serialize and Deserialize

### Option 1: try_deserialize (with validation)

```rust
use soul_msg::{SmsgEnvelope, EnvelopeError};
use zenoh_ext::{z_serialize, z_deserialize};

// Sender: create envelope and serialize manually
let envelope = SmsgEnvelope::new(MyMessage { field: "hello".to_string() });
let (vhash, nhash, payload) = envelope.into_parts();
let payload_bytes = z_serialize(&payload).to_bytes();

let mut tx_data = Vec::new();
tx_data.extend_from_slice(&nhash);  // 32 bytes
tx_data.extend_from_slice(&vhash);  // 32 bytes
tx_data.extend_from_slice(&payload_bytes);

// Receiver: validate and deserialize
match SmsgEnvelope::<MyMessage>::try_deserialize(&zenoh::bytes::ZBytes::from(tx_data)) {
    Ok(payload) => println!("Received: {:?}", payload),
    Err(EnvelopeError::TypeMismatch { .. }) => println!("Wrong message type"),
    Err(EnvelopeError::VersionMismatch { .. }) => println!("Schema version changed"),
    Err(e) => println!("Error: {:?}", e),
}
```

### Option 2: Serialize/Deserialize roundtrip

```rust
use zenoh_ext::{z_serialize, z_deserialize};

// Serialize (preserves name_hash and version_hash)
let envelope = SmsgEnvelope::new(MyMessage { field: "hello".to_string() });
let serialized = z_serialize(&envelope);

// Deserialize (no validation, preserves hashes)
let deserialized: SmsgEnvelope<MyMessage> = z_deserialize(&serialized).unwrap();
assert_eq!(*deserialized.version_hash(), *envelope.version_hash());
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
