# Quick Start: Using #[smsg] with Package Parsing

## Installation

Add to your `Cargo.toml`:
```toml
[dependencies]
soul_msg = "0.1.0"
```

## Basic Usage

### Option 1: Single .smsg File (Original Syntax)

```rust
use soul_msg::smsg;

#[smsg("messages/my_message.smsg")]
mod msgs { }
```

### Option 2: Single .smsg File (New Syntax)

```rust
use soul_msg::smsg;

#[smsg(category = file, path = "messages/my_message.smsg")]
mod msgs { }
```

### Option 3: Package (New)

Create a package structure:
```
my_package/
├── package.toml
└── messages/
    └── my_message.smsg
```

`package.toml`:
```toml
[package]
name = "my_package"
version = "1.0.0"
edition = "2026"

[dependencies]
# optional: other_package = "path/to/other_package"
```

Use in your code:
```rust
use soul_msg::smsg;

#[smsg(category = package, path = "packages/my_package")]
mod pkg { }
```

## Attribute Syntax

- `category` value is an **identifier** (no quotes): `category = file`, `category = package`
- `path` value is a **string literal** (with quotes): `path = "path/to/file"`

## Package Structure

A SoulSmsg Package is a directory containing:
- `package.toml` - Package metadata
- `*.smsg` files - Message definitions

### package.toml Format
```toml
[package]
name = "package_name"
version = "1.0.0"
edition = "2026"

[dependencies]
dependency_name = "path/to/dependency"
```

## Importing from Packages

In your .smsg file, import message types from dependencies:
```smsg
import package_name.module_path.MessageType

MyMessage {
    field: module_path.MessageType
}
```

## Generated Module Structure (FR-013)

The generated Rust code mirrors your package folder structure:

```
my_package/
├── package.toml
├── messages/
│   └── my_message.smsg
└── nested/
    └── other.smsg
```

This generates:
```rust
pub mod my_package {
    pub mod messages {
        // Structs from messages/my_message.smsg
    }
    
    pub mod nested {
        // Structs from nested/other.smsg
    }
}
```

Each subdirectory becomes a nested `mod` block, making the generated code intuitive and easy to navigate.
