# Data Model: Message Hash Verification

## Entities

### Message Definition
Represents a message structure in an smsg package.

| Field | Type | Description |
|-------|------|-------------|
| name | String | Message name |
| fields | Vec<Field> | Message fields |
| source_hash | [u8; 32] | blake3 hash of definition |

**Relationships**: 
- Part of a Message Package
- Has one MessageMeta implementation
- Used by SmsgEnvelope as payload

---

### MessageHash (blake3::Hash)
A 32-byte (256-bit) blake3 hash of a message definition.

| Field | Type | Description |
|-------|------|-------------|
| bytes | [u8; 32] | Raw hash bytes |

**Validation**: Must be exactly 32 bytes

---

### MessageMeta Trait
Trait providing compile-time access to message hash.

```rust
pub trait MessageMeta {
    fn version_hash() -> [u8; 32];
    fn message_name() -> &'static str;
}
```

| Method | Return | Description |
|--------|--------|-------------|
| version_hash | [u8; 32] | blake3 hash of message definition |
| message_name | &'static str | Name of the message type |

**Relationships**: Implemented by each generated message struct

---

### SmsgEnvelope<T>
Wrapper type containing message hash and payload.

```rust
pub struct SmsgEnvelope<T> {
    pub version_hash: [u8; 32],
    pub payload: T,
}
```

| Field | Type | Description |
|-------|------|-------------|
| version_hash | [u8; 32] | Hash from MessageMeta::version_hash() |
| payload | T | The wrapped message |

**Construction**: 
- `SmsgEnvelope::new(payload: T) -> Self` calls `MessageMeta::version_hash()` to initialize version_hash

**Relationships**:
- Generic over any message type T that implements MessageMeta

---

### CompatibilityReport
Result of comparing hashes between two endpoints.

| Field | Type | Description |
|-------|------|-------------|
| status | CompatibilityStatus | Match or Mismatch |
| details | Vec<MismatchDetail> | Details of any mismatches |

**Validation**: 
- status must be valid enum variant

---

## State Transitions

Not applicable - this is a stateless verification system.

## Validation Rules

1. FR-001: Each message definition MUST generate unique blake3 hash
2. FR-005: MessageMeta::version_hash() returns [u8; 32] (32 bytes)
3. FR-005b: SmsgEnvelope::new() initializes version_hash from MessageMeta::version_hash()
