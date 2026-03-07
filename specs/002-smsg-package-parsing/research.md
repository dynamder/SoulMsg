# Research: Adding Named Attributes to #[smsg] Macro

## Decision: Use syn crate's MetaList parsing with winnow

**Rationale**: The syn crate provides robust parsing for proc macro attributes. Using `syn::parse_macro_input!` with syn's MetaList allows parsing both:
1. Legacy string path: `#[smsg("path/to/file.smsg")]`
2. New named attributes: `#[smsg(category = file, path = "path/to/file.smsg")]`

The attribute value `category = file` uses an identifier (not a string literal), so we parse it as `syn::Ident`.

## Implementation Approach

### Option 1: Parse as MetaList (Recommended)
- Use `syn::parse_macro_input!(attr as syn::MetaList)` to parse named attributes
- Check if it's a list of name-value pairs (e.g., `category = file`)
- Fall back to string parsing if not a MetaList

### Option 2: Parse as LitStr handle simple string path
- Only: `#[smsg("path")]`
- Would require breaking change

**Chosen**: Option 1 - maintains backward compatibility

## Syntax Details

The new attribute syntax uses:
- **Identifier** (not string literal) for `category` value: `category = file` (NOT `category = "file"`)
- **String literal** for `path` value: `path = "path/to/file"` (quoted)

This is the Rust convention - identifiers don't need quotes, string paths do.

## Alternatives Considered

1. **Using custom parser combinators** - Overkill for simple attribute parsing
2. **Using clap for argument parsing** - Not suitable for proc macros
3. **Manual string parsing** - Error-prone, doesn't leverage syn

## Key Implementation Details

1. **Backward Compatibility**: Check if attribute is a simple string literal first
2. **Error Handling**: Provide clear compile errors for invalid attribute syntax
3. **Validation**: Validate `category` is either `file` or `package` (as identifiers)
4. **Parser**: Use winnow (already in project) for any complex parsing needs

## Module Structure Generation (FR-013)

**Decision**: Generate Rust `mod` declarations that mirror the package folder structure

**Rationale**: This provides intuitive, discoverable module paths based on how packages are organized on disk. Users can predict exactly what code will be generated.

### Example:
```
packages/mypackage/
├── package.toml
├── messages/
│   └── mymsg.smsg
└── nested/
    └── other.smsg
```

Generates:
```rust
pub mod mypackage {
    pub mod messages {
        // structs from mymsg.smsg
    }
    pub mod nested {
        // structs from other.smsg
    }
}
```

### Implementation Approach
- Walk the package directory recursively
- For each subdirectory, generate a nested `mod` block
- For each .smsg file, generate structs in the current module scope
- Validate module names are valid Rust identifiers

## References
- syn crate documentation: https://docs.rs/syn/latest/syn/
- winnow crate: https://github.com/winnow-rs/winnow
- proc_macro_attribute tutorial: https://doc.rust-lang.org/reference/procedural-macros.html
