# Feature Specification: Message Hash Verification

**Feature Branch**: `003-message-hash`  
**Created**: 2026-03-07  
**Status**: Draft  
**Input**: User description: "as messages need to transfer and message packages can be updated, there should be a mechanism to check if the message definition between two endpoints matches. Therefore, we should hash every individual message using blake3. However, where should we add the hash is not clear, give me some suggestions."

## Clarifications

### Session 2026-03-07

- Q: Hash storage design → A: Hash hardcoded in MessageMeta trait; SmsgEnvelope<T> wrapper type with version_hash field initialized via MessageMeta::version_hash() during construction; no special __msg_version_hashes__ field in smsg struct

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Verify Message Compatibility Between Endpoints (Priority: P1)

When two endpoints need to communicate, the system MUST be able to verify that their message definitions are compatible before exchanging messages.

**Why this priority**: Without message compatibility verification, endpoints may fail to understand each other's messages, leading to runtime errors, data corruption, or silent failures that are difficult to debug.

**Independent Test**: Can be tested by creating two separate message packages with known definitions, generating their hashes, and verifying that the comparison correctly identifies matching and non-matching definitions.

**Acceptance Scenarios**:

1. **Given** two endpoints have identical message definitions, **When** their message hashes are compared, **Then** the system reports them as compatible
2. **Given** two endpoints have different message definitions, **When** their message hashes are compared, **Then** the system reports them as incompatible with details about the mismatch
3. **Given** a message package is updated with new or modified messages, **When** hashes are regenerated, **Then** the new hashes reflect the changes

---

### User Story 2 - Detect Message Package Changes (Priority: P2)

The system MUST detect when a message package has been modified so endpoints can determine if they need to re-sync or alert users about incompatibility.

**Why this priority**: Message packages can be updated over time. Endpoints need to know when definitions have changed to prevent communication failures or data inconsistencies.

**Independent Test**: Can be tested by modifying a message definition and verifying the hash changes accordingly.

**Acceptance Scenarios**:

1. **Given** a message definition is unchanged, **When** hash is recomputed, **Then** the hash remains identical
2. **Given** a message definition is modified (field added, removed, or changed), **When** hash is recomputed, **Then** the hash differs from the original

---

### User Story 3 - Access Hashes via MessageMeta Trait (Priority: P3)

The system MUST provide a `MessageMeta` trait with a hardcoded blake3 `version_hash()` function for each message type. When building `SmsgEnvelope`, the trait function is called to initialize the version_hash field.

**Why this priority**: Runtime access to message hashes enables endpoints to compare their message definitions dynamically.

**Independent Test**: Can be tested by calling `version_hash()` on message types and verifying correct hash values are returned.

**Acceptance Scenarios**:

1. **Given** a message type implements `MessageMeta`, **When** `version_hash()` is called, **Then** the blake3 hash of the message definition is returned
2. **Given** an `SmsgEnvelope` is created, **When** it is constructed, **Then** the version_hash field is initialized by calling the message's `MessageMeta::version_hash()`

---

### Edge Cases

- What happens when two endpoints have different numbers of messages but some share the same hash?
- How does the system handle message packages with circular references or self-referential types?
- What happens when the hash computation fails due to malformed message definitions?
- How does the system handle backwards compatibility when message packages are updated?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST generate a unique blake3 hash for each individual message definition in a package
- **FR-002**: System MUST compare message hashes between two endpoints and report compatibility status
- **FR-003**: System MUST regenerate hashes when message definitions are modified
- **FR-004**: System MUST provide clear error messages when hash computation or comparison fails
- **FR-005**: System MUST provide a `MessageMeta` trait with a hardcoded blake3 `version_hash()` function for each generated message type
- **FR-005b**: System MUST provide an `SmsgEnvelope<T>` wrapper type that includes version_hash field, initialized by calling `MessageMeta::version_hash()` during construction
- **FR-006**: System MUST handle the case where endpoints have different numbers of messages gracefully

### Key Entities

- **Message Definition**: The structural definition of a message including its name, fields, and types
- **Message Hash**: A blake3 hash computed from the message definition used for identification and comparison
- **MessageMeta Trait**: A Rust trait with hardcoded `version_hash()` function returning the blake3 hash for each message type
- **SmsgEnvelope<T>**: A wrapper type containing version_hash (32 bytes) and payload (T), where version_hash is initialized via `MessageMeta::version_hash()`
- **Compatibility Report**: The result of comparing hashes between two endpoints, indicating match/mismatch status

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Endpoints can verify message compatibility in under 1 second for packages with up to 100 messages
- **SC-002**: Hash comparison correctly identifies 100% of matching and non-matching message definitions in test cases
- **SC-003**: System detects message definition changes within 1 second of hash recomputation
- **SC-004**: Users receive actionable error messages within 500ms when hash operations fail
