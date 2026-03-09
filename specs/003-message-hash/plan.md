# Implementation Plan: Message Hash Verification

**Branch**: `003-message-hash` | **Date**: 2026-03-08 | **Spec**: `spec.md`
**Input**: Feature specification from `/specs/003-message-hash/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Generate blake3 hash for each message definition in an smsg package. Hash is hardcoded in MessageMeta trait via procedural macro. SmsgEnvelope<T> wrapper includes version_hash and name_hash fields initialized at construction time via MessageMeta trait methods.

When sending: SmsgEnvelope serializes name_hash + version_hash + payload as headers
When receiving: try_deserialize extracts and verifies hashes before payload

## Technical Context

**Language/Version**: Rust 2024 (edition)  
**Primary Dependencies**: blake3 (for hash), syn, quote, winnow, toml, proc-macro2, zenoh-ext, zenoh  
**Storage**: N/A  
**Testing**: cargo test, cargo clippy  
**Target Platform**: Cross-platform (Windows/Linux)  
**Project Type**: Two-crate library (Rust):
- soul_msg: outer wrapper crate
- smsg_macro: inner proc-macro crate  
**Performance Goals**: Under 1 second for packages with up to 100 messages  
**Constraints**: Error messages within 500ms  
**Scale/Scope**: 100 messages per package

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Gate | Status | Notes |
|------|--------|-------|
| Technology Standards (clippy) | PASS | Will use cargo clippy |
| Technology Standards (winnow) | PASS | Already in use |
| Dependencies active maintenance | PASS | blake3 is well-maintained |
| Code Quality (<80 lines/function) | PASS | Design follows single responsibility |
| Testable Units | PASS | Unit tests required for all functions |

**Violations**: None

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

The project uses a two-crate structure:

```text
soul_msg/                    # Outer crate (library wrapper)
├── Cargo.toml
├── src/
│   └── lib.rs              # Re-exports smsg macro, provides MessageMeta trait and SmsgEnvelope<T>
└── tests/
    └── envelope_test.rs    # Tests for SmsgEnvelope

smsg_macro/                  # Inner crate (proc-macro)
├── Cargo.toml
├── src/
│   ├── lib.rs              # Main entry, smsg attribute macro
│   ├── codegen/
│   │   ├── mod.rs
│   │   ├── struct_gen.rs   # Struct generation
│   │   └── derive_gen.rs   # Generates impl ::soul_msg::MessageMeta for message structs
│   ├── parser/
│   │   ├── mod.rs
│   │   ├── package_parser.rs
│   │   └── import_resolver.rs
│   ├── ir.rs               # Intermediate representation
│   ├── hash.rs             # Hash computation
│   └── error.rs            # Error types
└── tests/
    ├── integration_test.rs  # Tests for code generation
    └── fixtures/
        ├── messages.smsg
        ├── messages_old.smsg
        └── packages/
```

### Crate Relationships

- **soul_msg** (outer): Re-exports `smsg` proc macro, provides `MessageMeta` trait, `SmsgEnvelope<T>`, and `EnvelopeError`
- **smsg_macro** (inner): Procedural macro crate that generates message structs and implements `MessageMeta` trait for them

## Architecture Design

### MessageMeta Trait

Defined in soul_msg, implemented by generated message structs via proc-macro:

```rust
pub trait MessageMeta {
    fn version_hash() -> [u8; 32];  // Hash of message schema (fields, types, order)
    fn name_hash() -> [u8; 32];      // Hash of message name (blake3 of message name string)
    fn message_name() -> &'static str;
}
```

### SmsgEnvelope<T>

Wrapper struct that bundles version metadata with payload:

```rust
pub struct SmsgEnvelope<T> {
    version_hash: [u8; 32],
    name_hash: [u8; 32],
    pub payload: T,
}

impl<T: MessageMeta + zenoh_ext::Deserialize> SmsgEnvelope<T> {
    pub fn new(payload: T) -> Self;
    pub fn into_parts(self) -> ([u8; 32], [u8; 32], T);
    pub fn into_payload(self) -> T;
    pub fn version_hash(&self) -> &[u8; 32];
    pub fn name_hash(&self) -> &[u8; 32];
    pub fn verify_version(&self, expected: &[u8; 32]) -> bool;
    pub fn verify_name(&self, expected: &[u8; 32]) -> bool;
    pub fn try_deserialize(data: &zenoh::bytes::ZBytes) -> Result<T, EnvelopeError>;
}

// zenoh_ext::Serialize implementation - serializes as name_hash + version_hash + payload
impl<T: MessageMeta + zenoh_ext::Serialize> zenoh_ext::Serialize for SmsgEnvelope<T> {
    fn serialize(&self, serializer: &mut zenoh_ext::ZSerializer) {
        self.name_hash.serialize(serializer);
        self.version_hash.serialize(serializer);
        self.payload.serialize(serializer);
    }
}

// zenoh_ext::Deserialize implementation - deserializes in same format
impl<T: MessageMeta + zenoh_ext::Deserialize> zenoh_ext::Deserialize for SmsgEnvelope<T> {
    fn deserialize(deserializer: &mut zenoh_ext::ZDeserializer) -> Result<Self, zenoh_ext::ZDeserializeError> {
        let name_hash: [u8; 32] = zenoh_ext::Deserialize::deserialize(deserializer)?;
        let version_hash: [u8; 32] = zenoh_ext::Deserialize::deserialize(deserializer)?;
        let payload: T = zenoh_ext::Deserialize::deserialize(deserializer)?;
        Ok(SmsgEnvelope { name_hash, version_hash, payload })
    }
}
```

### Serialization Format

When sending an SmsgEnvelope, the data is serialized as:
1. **name_hash** (32 bytes) - Hash of message type name
2. **version_hash** (32 bytes) - Hash of message schema
3. **payload** (variable) - The actual message data

### Deserialization Flow (try_deserialize)

When receiving data, the deserialization process is:

1. **Attempt to deserialize name_hash**: Try to extract first 32 bytes as name_hash
   - If deserialization fails → return `NotAnEnvelope` error (sender didn't send an SmsgEnvelope)
   
2. **Compare name_hash**: Compare extracted name_hash with T::name_hash()
   - If mismatch → return `TypeMismatch` error (sender sent different message type)
   
3. **Attempt to deserialize version_hash**: Try to extract next 32 bytes as version_hash
   - If deserialization fails → return `NotAnEnvelope` error
   
4. **Compare version_hash**: Compare extracted version_hash with T::version_hash()
   - If mismatch → return `VersionMismatch` error (sender used different schema version)
   
5. **Deserialize payload**: Deserialize remaining bytes as payload type T
   - If fails → return `DeserializeError`

### EnvelopeError Variants

```rust
pub enum EnvelopeError {
    NotAnEnvelope(String),           // Data is not a valid SmsgEnvelope
    TypeMismatch {                   // name_hash doesn't match expected type
        expected_name_hash: [u8; 32],
        actual_name_hash: [u8; 32],
    },
    VersionMismatch {                 // version_hash doesn't match expected version
        expected_version_hash: [u8; 32],
        actual_version_hash: [u8; 32],
    },
    DeserializeError(String),         // Payload deserialization failed
}
```

### Key Design Decisions

1. **Hash Computation**: Uses blake3 for both name_hash (fast, simple) and version_hash (includes schema details)
2. **Headers First**: name_hash and version_hash are always at the beginning of serialized data for efficient validation
3. **Fail Fast**: Validate name_hash before version_hash, version_hash before payload
4. **Self-Describing**: Message type and version are embedded in the envelope for runtime verification

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
