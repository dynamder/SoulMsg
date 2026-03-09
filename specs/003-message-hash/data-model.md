# Data Model: Message Hash Verification

## Entities

### Message Definition
Represents a message structure in an smsg package.

| Field | Type | Description |
|-------|------|-------------|
| name | String | Message name |
| fields | Vec<Field> | Message fields |
| version_hash | [u8; 32] | blake3 hash of message schema (fields, types, order) |
| name_hash | [u8; 32] | blake3 hash of message name |

**Relationships**: 
- Part of a Message Package
- Has one MessageMeta implementation
- Used by SmsgEnvelope as payload

---

### MessageHash (blake3::Hash)
A 32-byte (256-bit) blake3 hash used for version and type identification.

| Field | Type | Description |
|-------|------|-------------|
| bytes | [u8; 32] | Raw hash bytes |

**Validation**: Must be exactly 32 bytes

---

### MessageMeta Trait
Trait providing compile-time access to message hashes.

```rust
pub trait MessageMeta {
    fn version_hash() -> [u8; 32];
    fn name_hash() -> [u8; 32];
    fn message_name() -> &'static str;
}
```

| Method | Return | Description |
|--------|--------|-------------|
| version_hash | [u8; 32] | blake3 hash of message schema (fields, types, order) |
| name_hash | [u8; 32] | blake3 hash of message name string |
| message_name | &'static str | Name of the message type |

**Relationships**: Implemented by each generated message struct via proc-macro

---

### SmsgEnvelope<T>
Wrapper type containing version metadata and payload.

```rust
pub struct SmsgEnvelope<T> {
    version_hash: [u8; 32],
    name_hash: [u8; 32],
    pub payload: T,
}

impl<T: MessageMeta + zenoh_ext::Deserialize> SmsgEnvelope<T> {
    pub fn new(payload: T) -> Self;
    pub fn try_deserialize(data: &zenoh::bytes::ZBytes) -> Result<T, EnvelopeError>;
}

impl<T: MessageMeta + zenoh_ext::Serialize> zenoh_ext::Serialize for SmsgEnvelope<T> {
    fn serialize(&self, serializer: &mut zenoh_ext::ZSerializer);
}

impl<T: MessageMeta + zenoh_ext::Deserialize> zenoh_ext::Deserialize for SmsgEnvelope<T> {
    fn deserialize(deserializer: &mut zenoh_ext::ZDeserializer) -> Result<Self, zenoh_ext::ZDeserializeError>;
}
```

| Field | Type | Description |
|-------|------|-------------|
| version_hash | [u8; 32] | Hash from MessageMeta::version_hash() |
| name_hash | [u8; 32] | Hash from MessageMeta::name_hash() |
| payload | T | The wrapped message |

**Construction**: 
- `SmsgEnvelope::new(payload: T) -> Self` calls `MessageMeta::version_hash()` and `MessageMeta::name_hash()` to initialize fields

**Serialization**:
- Implements `zenoh_ext::Serialize` - serializes as `name_hash (32) + version_hash (32) + payload`

**Deserialization**:
- Implements `zenoh_ext::Deserialize` - deserializes in same format
- `try_deserialize()` provides progressive validation (name_hash first, then version_hash)

**Relationships**:
- Generic over any message type T that implements MessageMeta

---

### EnvelopeError
Error types for envelope deserialization.

```rust
pub enum EnvelopeError {
    NotAnEnvelope(String),
    TypeMismatch {
        expected_name_hash: [u8; 32],
        actual_name_hash: [u8; 32],
    },
    VersionMismatch {
        expected_version_hash: [u8; 32],
        actual_version_hash: [u8; 32],
    },
    DeserializeError(String),
}
```

| Variant | Description |
|---------|-------------|
| NotAnEnvelope | Data is not a valid SmsgEnvelope (failed to deserialize headers) |
| TypeMismatch | name_hash doesn't match expected message type |
| VersionMismatch | version_hash doesn't match expected schema version |
| DeserializeError | Payload deserialization failed |

---

## Serialization Format

When sending an SmsgEnvelope, data is serialized as:

```
[name_hash: 32 bytes][version_hash: 32 bytes][payload: variable bytes]
```

## Deserialization Flow

1. **Extract name_hash**: Try to deserialize first 32 bytes as name_hash
   - Failure → `NotAnEnvelope`
   
2. **Compare name_hash**: Compare with T::name_hash()
   - Mismatch → `TypeMismatch`
   
3. **Extract version_hash**: Try to deserialize next 32 bytes as version_hash
   - Failure → `NotAnEnvelope`
   
4. **Compare version_hash**: Compare with T::version_hash()
   - Mismatch → `VersionMismatch`
   
5. **Deserialize payload**: Deserialize remaining bytes as T
   - Failure → `DeserializeError`

---

## State Transitions

Not applicable - this is a stateless verification system.

## Validation Rules

1. FR-001: Each message definition MUST generate unique blake3 hash
2. FR-005: MessageMeta::version_hash() returns [u8; 32] (32 bytes)
3. FR-005b: MessageMeta::name_hash() returns [u8; 32] (32 bytes)
4. FR-005c: SmsgEnvelope::new() initializes version_hash and name_hash from MessageMeta
5. FR-006: SmsgEnvelope::try_deserialize() validates name_hash before version_hash
