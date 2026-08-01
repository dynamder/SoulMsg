# SoulMsg

[简体中文](./README_CN.md)

A schema-verified message layer for protobuf: payloads are encoded with
[protobuf](https://protobuf.dev/) and wrapped in an envelope carrying cryptographic
hashes (Blake3) for **message identity** and **definition version**.

## Overview

SoulMsg adds a hash envelope on top of protobuf payloads:

```
[name_hash:32][version_hash:32][payload_len:u32(LE)][protobuf payload]
```

- `name_hash` — identifies the message type by its short name (stable across schema edits).
- `version_hash` — captures the full definition (message name + fields), so any schema
  change is detected.
- Both hashes are computed **canonically from the protoc descriptor set**, so every
  language that hashes the same `.proto` produces **byte-identical** output. This is
  verified across Rust, Python, Go and Kotlin (`proto_poc/verification`).

Decoding is policy-driven: `Strict` (default) rejects hash mismatches;
`Lenient` skips the hash check and relies on protobuf's tolerant schema evolution.

## Features

- **Protobuf payloads**: use the entire protobuf ecosystem (tooling, codegen, conformance)
- **Schema identity**: cryptographic `name_hash` / `version_hash` on the wire
- **Strict by default, lenient on demand**: strict verification or tolerant evolution
- **Cross-language**: thin `soulmsg` bindings for Python / Go / Kotlin, all byte-identical
- **Byte-layer only**: the library produces/consumes bytes; transports (zenoh, etc.) carry them

## Installation

```toml
[dependencies]
soul_msg = "0.2"
prost = "0.14"
```

No build script, no `protoc`, no `OUT_DIR` — the `#[smsg]` macro parses `.proto`
at compile time.

## Usage (Rust)

### 1. Define messages and generate code

Write a `.proto` and attach the `#[smsg]` macro to a module. The macro parses the
file at compile time (with the pure-Rust `protox` compiler) and generates a prost
struct + `EnvelopeMeta` for every message:

```rust
#[smsg("proto/chat.proto")]
pub mod chat {}
```

### 2. Serialize / deserialize

```rust
use soul_msg::{Envelope, EnvelopeError, Policy};

// Send: type is the schema — no runtime metadata needed.
let msg = chat::ChatMessage {
    sender: "Alice".to_string(),
    content: "Hello, World!".to_string(),
    timestamp: 1_699_999_999,
};
let wire: Vec<u8> = Envelope::new_typed(&msg).to_bytes();

// Receive: strict (default).
let received: chat::ChatMessage = Envelope::try_deserialize_typed(&wire)?;

// Receive: lenient (protobuf's tolerant evolution).
let received: chat::ChatMessage =
    Envelope::try_deserialize_with_policy_typed(&wire, Policy::Lenient)?;
```

### 3. Runtime schema (dynamic / dispatch)

When the message type isn't known ahead of time (e.g. a subscriber receiving many
types), load the descriptor set at runtime and peek the hashes to dispatch:

```rust
use soul_msg::Schema;

let schema = Schema::from_descriptor_set(include_bytes!("descriptors.pb"))?;
let (name_hash, version_hash) = Envelope::peek(&wire)?;
let meta = schema.by_name_hash(&name_hash); // -> MessageRef
```

## Cross-language bindings

Each language uses its own protobuf runtime; the `soulmsg` binding adds only the
~100 lines of hashing + framing + policy. All are verified byte-identical to Rust:

| Language | Binding | Payload codec |
|----------|---------|---------------|
| Rust | `soul_msg` + `smsg_macro` | prost |
| Python | `bindings/python/soulmsg` | protobuf (`pip install protobuf blake3`) |
| Go | `bindings/go/soulmsg` | protobuf-go + blake3 |
| Kotlin/JVM | `bindings/kotlin/soulmsg` | protobuf-java + BouncyCastle |

See `proto_poc/verification/README.md` for the byte-for-byte verification harness.

## Error handling

- `NotAnEnvelope` — too short / payload length framing violation
- `TypeMismatch` — `name_hash` doesn't match the expected message
- `VersionMismatch` — `version_hash` doesn't match the expected definition
- `DeserializeError` — protobuf payload decode failure

## License

MIT
