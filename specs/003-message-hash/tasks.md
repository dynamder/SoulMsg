# Tasks: Message Hash Verification

**Feature**: 003-message-hash  
**Branch**: `003-message-hash`  
**Generated**: 2026-03-07

## Dependencies Graph

```
Phase 1 (Setup)
    └── Phase 2 (Foundational)
            └── Phase 3 (US3 - Access Hashes + SmsgEnvelope)
                    └── Phase 4 (US1 - Verify Compatibility + Error Handling)
                            └── Phase 5 (US2 - Detect Changes + Edge Cases)
                                    └── Phase 6 (Polish)
```

## Phase 1: Setup

- [X] T001 Add blake3 dependency to Cargo.toml in dependencies section
- [X] T002 Run cargo fetch to verify blake3 dependency resolves

## Phase 2: Foundational

- [X] T003 Create hash module src/hash.rs with blake3 hash computation function
- [X] T004 Add hash module to lib.rs: `mod hash;`
- [X] T005 Add derive module for MessageMeta generation in codegen/derive_gen.rs

## Phase 3: User Story 3 - Access Hashes via MessageMeta Trait (P3)

**Goal**: Provide MessageMeta trait with hardcoded version_hash() for each message type

**Independent Test**: Call version_hash() on generated message types, verify correct blake3 hash values

**Implementation**:

- [X] T006 [P] [US3] Define MessageMeta trait in src/codegen/derive_gen.rs
- [X] T007 [P] [US3] Implement MessageMeta trait generation in derive_gen.rs for each struct
- [X] T008 [US3] Generate version_hash() returning [u8; 32] - hardcoded hash from struct definition
- [X] T009 [US3] Generate message_name() returning &'static str
- [X] T010 [US3] Integrate MessageMeta generation into lib.rs smsg macro flow
- [X] T019 Add SmsgEnvelope<T> wrapper type generation to derive_gen.rs
- [X] T020 Implement SmsgEnvelope::new() calling MessageMeta::version_hash()
- [X] T021 Implement SmsgEnvelope::into_parts() method

## Phase 4: User Story 1 - Verify Message Compatibility Between Endpoints (P1)

**Goal**: Compare message hashes between two endpoints and report compatibility

**Independent Test**: Create two packages, generate hashes, verify comparison identifies matching/non-matching

**Implementation**:

- [X] T011 [P] [US1] Define CompatibilityStatus enum (Match, Mismatch)
- [X] T012 [P] [US1] Define MismatchDetail struct for error reporting
- [X] T013 [US1] Implement hash comparison function in src/hash.rs
- [X] T014 [US1] Generate CompatibilityReport from comparison result
- [X] T022 [P] [US1] Define HashError enum in src/error.rs for hash computation failures
- [X] T023 [US1] Implement error messages for hash computation failures (FR-004)
- [X] T024 [US1] Implement error messages for hash comparison failures (FR-004)

## Phase 5: User Story 2 - Detect Message Package Changes (P2)

**Goal**: Detect when message package is modified by regenerating hashes

**Independent Test**: Modify message definition, verify hash changes

**Implementation**:

- [X] T015 [US2] Hash generation must include all struct fields in canonical order
- [X] T016 [US2] Test that field addition/removal changes hash
- [X] T017 [US2] Test that field type change changes hash
- [X] T018 [US2] Test that field name change changes hash
- [X] T025 [US2] Handle endpoints with different message counts gracefully (FR-006)

## Phase 6: Polish & Cross-Cutting Concerns

- [X] T026 Run cargo clippy and fix warnings
- [X] T027 Run cargo test and ensure all tests pass
- [X] T028 [P] Verify test coverage meets 80% threshold for core business logic (Constitution III)

## Parallel Execution Opportunities

| Tasks | Can Run In Parallel Because |
|-------|----------------------------|
| T006 + T007 | Different trait methods |
| T011 + T012 | Different type definitions |
| T015 + T016 + T017 + T018 | Different test scenarios |
| T022 + T023 | Different error types |
| T026 + T028 | Different quality gates |

## Implementation Strategy

**MVP Scope**: User Story 3 (Phase 3) - MessageMeta trait is the foundation for all other stories

**Incremental Delivery**:
1. Phase 1-2: Setup + foundational hash module
2. Phase 3: MessageMeta trait + SmsgEnvelope (core feature)
3. Phase 4: Hash comparison + error handling
4. Phase 5: Detection of changes + edge cases
5. Phase 6: Polish + test coverage

## Test Summary

| User Story | Test Criteria |
|------------|----------------|
| US1 | Hash comparison returns correct match/mismatch status |
| US2 | Modified message definitions produce different hashes |
| US3 | version_hash() returns valid 32-byte blake3 hash |

## Requirements Coverage

| Requirement | Task IDs |
|-------------|----------|
| FR-001 (unique blake3 hash) | T003, T006-T009 |
| FR-002 (compare hashes) | T011-T014 |
| FR-003 (regenerate on modification) | T015-T018 |
| FR-004 (error messages) | T022-T024 |
| FR-005 (MessageMeta trait) | T006-T010 |
| FR-005b (SmsgEnvelope) | T019-T021 |
| FR-006 (different message counts) | T025 |
