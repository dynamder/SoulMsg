# SoulMsg Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-03-07

## Active Technologies
- Rust 2024 (edition) + blake3 (for hash), syn, quote, winnow, toml, proc-macro2 (003-message-hash)

- Rust 1.75+ (proc macros) + winnow (parsing), quote, syn (002-smsg-package-parsing)
- File-based (.smsg files, package.toml) (002-smsg-package-parsing)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test; cargo clippy

## Code Style

Rust 1.75+ (proc macros): Follow standard conventions

## Recent Changes
- 003-message-hash: Added Rust 2024 (edition) + blake3 (for hash), syn, quote, winnow, toml, proc-macro2
- 002-smsg-package-parsing: Added Rust 1.75+ (proc macros) + winnow (parsing), quote, syn

- 002-smsg-package-parsing: Added Rust 1.75+ (proc macros) + winnow (parsing), quote, syn

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
