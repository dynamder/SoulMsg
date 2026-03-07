# Research: Message Hash Verification

## Decision: Use blake3 crate for hash generation

**Rationale**: User explicitly requested blake3 for hash generation. blake3 is:
- Actively maintained (recent releases in 2024)
- Extremely fast (faster than SHA-256, MD5)
- Produces 32-byte (256-bit) hashes suitable for version identification
- Has simple API: `blake3::hash(data)` returns `Hash`
- No runtime dependencies beyond standard library

**Alternatives considered**:
- sha2: Slower than blake3, same output size
- md5: Not cryptographically secure, but faster - rejected for "verification" use case
- xxhash3: Very fast but not cryptographic - acceptable for non-security but blake3 is standard

## Hash Storage Design

**Decision**: Hash hardcoded in MessageMeta trait; SmsgEnvelope<T> wrapper type

**Rationale**: 
- MessageMeta trait with `version_hash()` function provides compile-time hash
- Hash is computed at code generation time, hardcoded into the generated trait impl
- SmsgEnvelope<T> wraps payload with version_hash field, initialized via MessageMeta::version_hash() during construction

**Implementation approach**:
1. In proc-macro, compute blake3 hash of the message struct definition (source code or IR)
2. Generate MessageMeta trait impl with hardcoded hash
3. Generate SmsgEnvelope<T> struct with version_hash: [u8; 32] field
4. In envelope constructor, call MessageMeta::version_hash() to initialize the field

## References

- blake3 crate: https://crates.io/crates/blake3
- Specification: `specs/003-message-hash/spec.md`
