# Contract: #[smsg] Macro Attribute Syntax

## Overview
The `#[smsg]` proc macro attribute accepts two forms of input for specifying the source of message definitions.

## Syntax Variants

### Variant 1: String Path (Legacy/Backward Compatible)
```rust
#[smsg("path/to/messages.smsg")]
```

**Equivalent to**: `#[smsg(category = file, path = "path/to/messages.smsg")]`

### Variant 2: Named Attributes (New Syntax)
```rust
#[smsg(category = file, path = "path/to/messages.smsg")]
#[smsg(category = package, path = "path/to/package")]
```

## Attribute Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `category` | identifier | No | `file` | Source category: `file` or `package` (no quotes) |
| `path` | string literal | Yes | - | Path to .smsg file or package directory (quoted) |

## Error Cases

### Invalid Category
```rust
#[smsg(category = invalid, path = "test.smsg")]
// Error: category must be `file` or `package`
```

### Missing Path
```rust
#[smsg(category = file)]
// Error: path is required
```

### Malformed Syntax
```rust
#[smsg(not_an_attribute)]
// Error: expected `category = identifier` or path string
```

## Usage Examples

### Single File (Legacy)
```rust
#[smsg("messages/mymsg.smsg")]
mod msgs { }
// Generates structs from mymsg.smsg
```

### Single File (New Syntax)
```rust
#[smsg(category = file, path = "messages/mymsg.smsg")]
mod msgs { }
```

### Package
```rust
#[smsg(category = package, path = "packages/mypackage")]
mod pkg { }
// Loads package.toml and parses all .smsg files in the package
```

## Module Structure Contract (FR-013)

When using `category = package`, the generated Rust module structure MUST mirror the package folder structure:

```
packages/mypackage/
├── package.toml
├── messages/
│   └── mymsg.smsg
└── nested/
    └── other.smsg
```

MUST generate:
```rust
pub mod mypackage {
    // Root module (from package.toml metadata)
    
    pub mod messages {
        // Structs from messages/mymsg.smsg
    }
    
    pub mod nested {
        // Structs from nested/other.smsg
    }
}
```

### Rules:
1. Each subdirectory becomes a nested `mod` block
2. Module names are derived from folder names (must be valid Rust identifiers)
3. .smsg files at package root generate structs in the root module
4. .smsg files in subdirectories generate structs in the corresponding nested module
