---

description: "Task list template for feature implementation"
---

# Tasks: SoulSmsg Package Parsing

**Input**: Design documents from `/specs/002-smsg-package-parsing/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are REQUIRED per spec.md - each user story has explicit "Independent Test" criteria

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- Paths shown below assume single project - adjust based on plan.md structure

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [X] T001 [P] Review existing project structure in src/ and Cargo.toml
- [X] T002 Add test dependencies to Cargo.toml for TOML parsing (toml crate)
- [X] T003 Create test fixtures directory tests/fixtures/packages/

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T004 Update src/ir.rs - Add SmsgPackage struct with name, version, edition, dependencies fields per data-model.md
- [X] T005 Update src/ir.rs - Add Dependency struct with name and path fields per data-model.md
- [X] T006 [P] Update src/error.rs - Add PackageError enum variants for TOML parsing errors
- [X] T007 [P] Update src/error.rs - Add ImportError enum variants for import resolution errors

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Define Package Metadata and Dependencies (Priority: P1) 🎯 MVP

**Goal**: Parse package.toml files to extract package metadata (name, version, edition) and dependencies

**Independent Test**: Create a package with package.toml containing package metadata and dependencies, verify parsing extracts correct values

### Tests for User Story 1 (REQUIRED per spec.md)

- [X] T008 [P] [US1] Unit test: parse valid package.toml with all fields in tests/
- [X] T009 [P] [US1] Unit test: parse package.toml with multiple dependencies in tests/
- [X] T010 [P] [US1] Unit test: parse package.toml with no dependencies in tests/
- [X] T011 [US1] Unit test: error handling for missing [package] section in tests/
- [X] T012 [US1] Unit test: error handling for missing required fields in tests/

### Implementation for User Story 1

- [X] T013 [P] [US1] Create src/parser/package_parser.rs - Implement TOML parser for package.toml using toml crate
- [X] T014 [P] [US1] Add ModuleStructure and Module structs to src/ir.rs per data-model.md
- [X] T015 [US1] Implement parse_package_toml function in src/parser/package_parser.rs (FR-001, FR-002)
- [X] T016 [US1] Implement parse_dependencies function in src/parser/package_parser.rs (FR-003, FR-004, FR-005)
- [X] T017 [US1] Validate edition field is "2026" per FR-002
- [X] T018 [US1] Run cargo test to verify US1 tests pass

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - Import Message Types from Dependencies (Priority: P1)

**Goal**: Parse import statements in .smsg files and resolve to correct package/message type

**Independent Test**: Create an .smsg file with import statements and verify parser resolves package and module path correctly

### Tests for User Story 2 (REQUIRED per spec.md)

- [X] T019 [P] [US2] Unit test: parse valid import statement "import package.module_path.msg_type" in tests/
- [X] T020 [P] [US2] Unit test: error for invalid package name in import in tests/
- [X] T021 [US2] Unit test: error for malformed import syntax in tests/

### Implementation for User Story 2

- [X] T022 [P] [US2] Extend src/parser/mod.rs - Add import statement parser using winnow
- [X] T023 [P] [US2] Add ImportStatement struct to src/ir.rs (package, module_path, message_type fields)
- [X] T024 [US2] Implement import resolution logic in src/parser/import_resolver.rs (FR-006, FR-007)
- [X] T025 [US2] Implement dependency path resolution relative to package.toml per FR-010
- [X] T026 [US2] Implement error handling for unresolvable imports per FR-009
- [X] T027 [US2] Run cargo test to verify US2 tests pass

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 - Resolve Module Paths from Package Structure (Priority: P2)

**Goal**: Generate Rust modules that mirror the package folder structure

**Independent Test**: Create packages in nested folders and verify module paths match folder structure

### Tests for User Story 3 (REQUIRED per spec.md)

- [X] T028 [P] [US3] Unit test: generate module structure for nested directories in tests/
- [X] T029 [P] [US3] Unit test: validate module names are valid Rust identifiers in tests/
- [X] T030 [US3] Integration test: generate complete Rust code for sample package in tests/

### Implementation for User Story 3

- [X] T031 [P] [US3] Implement package directory walker in src/parser/package_parser.rs (FR-013)
- [X] T032 [P] [US3] Update src/codegen/struct_gen.rs - Add module structure generation logic
- [X] T033 [US3] Implement nested mod block generation in src/codegen/struct_gen.rs (FR-013)
- [X] T034 [US3] Implement module sandboxing (enforce FR-011 - packages cannot access files outside root)
- [X] T035 [US3] Run cargo test to verify US3 tests pass

**Checkpoint**: All user stories should now be independently functional

---

## Phase 6: Integrate Macro Attribute Parsing (Category = package)

**Purpose**: Update the main #[smsg] macro to support the new category = package attribute

- [X] T036 Update src/lib.rs - Extend attribute parsing to support category = package syntax per contracts/macro-attributes.md
- [X] T037 Implement package loading flow in src/lib.rs (load package.toml, parse all .smsg files, generate module structure)
- [X] T038 Ensure backward compatibility with existing string syntax per contract
- [X] T039 Run cargo test to verify full integration works

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [X] T040 [P] Update quickstart.md with package parsing examples if needed
- [X] T041 Run cargo clippy and fix any warnings
- [ ] T042 [P] Add integration test in tests/integration_test.rs covering full package workflow
- [X] T043 Run quickstart.md validation scenarios from quickstart.md
- [X] T044 Run cargo test (full test suite)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3)
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P1)**: Can start after Foundational (Phase 2) - May integrate with US1 but should be independently testable
- **User Story 3 (P2)**: Can start after Foundational (Phase 2) - Uses module structure generation, independently testable

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- IR structures before parsers
- Parsers before code generation
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- US1 test tasks T008-T010 can run in parallel
- US1 implementation tasks T013-T014 can run in parallel
- Once Foundational phase completes, all user stories can start in parallel (if team capacity allows)
- US2 test tasks T019-T020 can run in parallel
- US2 implementation tasks T022-T023 can run in parallel
- US3 test tasks T028-T029 can run in parallel
- US3 implementation tasks T031-T032 can run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task: "Unit test: parse valid package.toml with all fields in tests/"
Task: "Unit test: parse package.toml with multiple dependencies in tests/"
Task: "Unit test: parse package.toml with no dependencies in tests/"

# Launch all implementation for User Story 1 together:
Task: "Create src/parser/package_parser.rs - Implement TOML parser for package.toml"
Task: "Add ModuleStructure and Module structs to src/ir.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo
4. Add User Story 3 → Test independently → Deploy/Demo
5. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1
   - Developer B: User Story 2
   - Developer C: User Story 3
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
