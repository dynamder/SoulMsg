# Implementation Plan: Message Hash Verification

**Branch**: `003-message-hash` | **Date**: 2026-03-07 | **Spec**: `spec.md`
**Input**: Feature specification from `/specs/003-message-hash/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Generate blake3 hash for each message definition in an smsg package. Hash is hardcoded in MessageMeta trait via procedural macro. SmsgEnvelope<T> wrapper includes version_hash field initialized at construction time via MessageMeta::version_hash().

## Technical Context

**Language/Version**: Rust 2024 (edition)  
**Primary Dependencies**: blake3 (for hash), syn, quote, winnow, toml, proc-macro2  
**Storage**: N/A  
**Testing**: cargo test, cargo clippy  
**Target Platform**: Cross-platform (Windows/Linux)  
**Project Type**: Two-crate library (Rust):- soul_msg: outer wrapper crate- smsg_macro: inner proc-macro crate  
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
└── src/
    └── lib.rs              # Re-exports smsg macro, provides SmsgEnvelope<T>

smsg_macro/                  # Inner crate (proc-macro)
├── Cargo.toml
├── src/
│   ├── lib.rs              # Main entry, smsg attribute macro
│   ├── codegen/
│   │   ├── mod.rs
│   │   ├── struct_gen.rs   # Struct generation
│   │   └── derive_gen.rs   # Derive macro generation (MessageMeta, SmsgEnvelope)
│   ├── parser/
│   │   ├── mod.rs
│   │   ├── package_parser.rs
│   │   └── import_resolver.rs
│   ├── ir.rs               # Intermediate representation
│   ├── hash.rs             # Hash computation
│   └── error.rs            # Error types
└── tests/
    ├── integration_test.rs
    └── fixtures/
        ├── messages.smsg
        └── packages/
```

### Crate Relationships

- **soul_msg** (outer): Re-exports `smsg` proc macro, provides `MessageMeta` trait and `SmsgEnvelope<T>` wrapper type
- **smsg_macro** (inner): Procedural macro crate that generates message structs and implements `MessageMeta` trait for them (references `::soul_msg::MessageMeta`)

### Feature Implementation (003-message-hash additions)

Hash computation and MessageMeta implementation are in the smsg_macro crate:

```text
smsg_macro/src/
├── hash.rs                 # blake3 hash computation
├── codegen/
│   └── derive_gen.rs       # Generates impl ::soul_msg::MessageMeta for message structs
```

**Key Design**: `MessageMeta` trait and `SmsgEnvelope<T>` are defined in the outer crate (soul_msg). The proc-macro generates code that implements `MessageMeta` for the generated message structs, referencing the trait via `::soul_msg::MessageMeta`.

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
