# SoulMsg

[简体中文](./README_CN.md)

A message serialization framework for Rust with type-safe versioning and schema evolution support.

## Overview

SoulMsg is a message serialization library that combines a custom DSL (`.smsg` files) for defining message types with automatic Rust struct generation. Messages are wrapped in envelopes containing cryptographic hashes (Blake3) for version and type verification, enabling safe schema evolution in distributed systems.

## Features

- **Custom DSL**: Define messages using a simple `.smsg` file format
- **Proc-macro Generation**: Automatically generate Rust structs from `.smsg` definitions
- **Type Safety**: Cryptographic name hashing prevents deserializing messages into wrong types
- **Version Tracking**: Version hashing enables detection of schema changes
- **Zenoh Integration**: Built on Zenoh for efficient serialization/deserialization (Zenoh-only, serde coming soon)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
soul_msg = "0.1"
zenoh = "1.7"
zenoh-ext = "1.7"
```

## Usage

### Define Messages (.smsg files)

Create a `.smsg` file to define your message types:

```smsg
message ChatMessage {
    string sender
    string content
    int64 timestamp
}

message Position {
    float64 x
    float64 y
    float64 z
}
```

### Generate Rust Code

Use the `#[smsg]` attribute macro to generate Rust code from your `.smsg` file:

```rust
use soul_msg::{smsg, MessageMeta, SmsgEnvelope};
use zenoh_ext::z_serialize;

#[smsg(category = file, path = "messages.smsg")]
pub mod chat_msgs {}
```

### Serialize and Deserialize

```rust
use soul_msg::SmsgEnvelope;
use zenoh_ext::z_serialize;

// Create a message
let msg = chat_msgs::ChatMessage {
    sender: "Alice".to_string(),
    content: "Hello, World!".to_string(),
    timestamp: 1699999999,
};

// Wrap in envelope with version/name hashes
let envelope = SmsgEnvelope::new(msg);

// Serialize
let serialized = z_serialize(&envelope);
let bytes = serialized.to_bytes();

// Deserialize (with type and version verification)
let received: chat_msgs::ChatMessage =
    SmsgEnvelope::try_deserialize(bytes).unwrap();
```

### Zenoh-Only Support (Serde Coming Soon)

Currently, SoulMsg only supports serialization via **Zenoh** with `zenoh-ext`. This is the default and recommended backend for distributed systems and pub/sub messaging.

**Serde support is on the roadmap** and will be added in a future release for use cases that don't require Zenoh.

## Package Support

SoulMsg supports organizing messages into **packages** for larger projects. A package is a directory containing:

1. A `package.toml` file defining package metadata
2. Multiple `.smsg` files organized in subdirectories

### Creating a Package

Create a directory structure like this:

```
mypackage/
├── package.toml
├── person.smsg
└── orders/
    └── order.smsg
```

The `package.toml` should contain:

```toml
[package]
name = "mypackage"
version = "1.0.0"
edition = "2026"
```

Define messages in `.smsg` files as usual. Subdirectories become Rust modules.

### Using Packages

Use the `category = package` attribute:

```rust
#[smsg(category = package, path = "path/to/mypackage")]
pub mod mypackage {}
```

This generates a module hierarchy matching your directory structure:

```rust
use mypackage::person::Person;
use mypackage::orders::Order;
```

Packages enable:
- **Modular organization**: Group related messages together
- **Namespacing**: Avoid name collisions between message types
- **Selective importing**: Import only the messages you need

## Supported Types

| .smsg Type | Rust Type |
|------------|-----------|
| `string`   | `String`  |
| `int32`    | `i32`     |
| `int64`    | `i64`     |
| `float32`  | `f32`     |
| `float64`  | `f64`     |
| `bool`     | `bool`    |
| `bytes`    | `Vec<u8>` |

Nested messages are also supported.

## Error Handling

`SmsgEnvelope::try_deserialize` returns `EnvelopeError` for various failure conditions:

- `NotAnEnvelope`: Data is too short or has invalid length prefixes
- `TypeMismatch`: Message name hash doesn't match expected type
- `VersionMismatch`: Message version hash doesn't match expected version
- `DeserializeError`: Failed to deserialize the payload

## License

MIT
