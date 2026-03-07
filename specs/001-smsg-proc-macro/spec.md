# Feature Specification: SMSG Proc Macro - Generate Rust Structs from Message Definitions

**Feature Branch**: `001-smsg-proc-macro`  
**Created**: 2026-03-02  
**Status**: Draft  
**Input**: User description: "this is a rust proc macro crate. it analyze the input .smsg file, parsing it, and generate rust structs accordingly. the usage is like below:

#[smsg(\"path/to/file.smsg\")]
pub mod example_msg {}

the .smsg file looks like below(a ROS-style like message definition).

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

message RobotState {
    string name
    Position position
    int32 status
}"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Parse .smsg Files and Generate Rust Structs (Priority: P1)

As a Rust developer, I want to define message structures in .smsg files using a ROS-style syntax, so that I can automatically generate Rust struct definitions in my codebase.

**Why this priority**: This is the core functionality of the entire crate. Without this, there is no value to users.

**Independent Test**: Can be tested by creating a .smsg file with message definitions, applying the proc macro, and verifying that correct Rust structs are generated with proper field names and types.

**Acceptance Scenarios**:

1. **Given** a valid .smsg file containing a message definition with primitive types, **When** the proc macro is applied to a module, **Then** Rust structs are generated with matching field names and appropriate Rust types (string → String, int64 → i64, float64 → f64)

2. **Given** a valid .smsg file containing nested message definitions, **When** the proc macro is applied, **Then** Rust structs are generated for all messages with nested types referencing the other generated structs

3. **Given** a .smsg file with multiple message definitions, **When** the proc macro is applied, **Then** all message structs are generated and accessible within the module

---

### User Story 2 - Support All Primitive Types (Priority: P2)

As a Rust developer, I want all common primitive types from ROS message syntax to be supported, so that I can define messages with various data types.

**Why this priority**: Users need a complete type system to define meaningful messages for robotics and messaging applications.

**Independent Test**: Can be tested by creating .smsg files with each primitive type and verifying the correct Rust type is generated.

**Acceptance Scenarios**:

1. **Given** .smsg file with string type, **When** parsed, **Then** generates String type in Rust
2. **Given** .smsg file with int8, int16, int32, int64 types, **When** parsed, **Then** generates i8, i16, i32, i64 respectively
3. **Given** .smsg file with uint8, uint16, uint32, uint64 types, **When** parsed, **Then** generates u8, u16, u32, u64 respectively
4. **Given** .smsg file with float32, float64 types, **When** parsed, **Then** generates f32, f64 respectively
5. **Given** .smsg file with bool type, **When** parsed, **Then** generates bool type in Rust

---

### User Story 3 - Handle File Path Resolution (Priority: P3)

As a Rust developer, I want the proc macro to locate .smsg files relative to my source code, so that I can organize message definitions alongside my code.

**Why this priority**: Proper file path resolution is essential for practical usage in real projects.

**Independent Test**: Can be tested by placing .smsg files in various locations relative to the Rust source files and verifying the macro resolves paths correctly.

**Acceptance Scenarios**:

1. **Given** a .smsg file in the same directory as the Rust source file, **When** macro uses relative path, **Then** the file is found and parsed correctly
2. **Given** a .smsg file in a subdirectory, **When** macro uses relative path from source file location, **Then** the file is found and parsed correctly

---

### Edge Cases

- What happens when the .smsg file does not exist or path is invalid?
  - Report an compile error of file not found

- How does the system handle malformed .smsg syntax (missing field types, invalid type names)?
  - Report the wrong syntax, with its line-col location and description.

- How does the system handle duplicate message names within the same file?
  - Report redefine error

- What happens when a referenced nested message type is not defined in the file?
  - Report a message type not defined error

- How does the system handle empty message definitions?
  - pass with nothing generated

- What happens when field names conflict with Rust keywords?
  - forbid it and report an error


## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The proc macro MUST parse .smsg files containing ROS-style message definitions using the nom parser crate
- **FR-002**: The proc macro MUST generate Rust struct definitions matching the message structure
- **FR-003**: The generated structs MUST have public fields matching the field names in the .smsg file
- **FR-004**: The proc macro MUST support primitive types: string, bool, int8, int16, int32, int64, uint8, uint16, uint32, uint64, float32, float64
- **FR-005**: The proc macro MUST support nested message types (messages defined within the same file)
- **FR-006**: The proc macro MUST resolve file paths relative to the Rust source file location
- **FR-007**: The proc macro MUST generate multiple structs when the .smsg file contains multiple message definitions
- **FR-008**: The system MUST provide clear error messages when .smsg file parsing fails
- **FR-009**: The system MUST validate .smsg syntax and report specific errors with line numbers

### Key Entities

- **Message Definition**: A named structure in .smsg containing field definitions
- **Field**: A named element within a message with a type
- **Primitive Type**: Basic data types (string, integer, float, bool)
- **Nested Type**: A message type referenced by another message defined in the same file

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Developers can define a complete message in .smsg syntax and generate Rust structs in under 5 minutes from first use
- **SC-002**: All supported primitive types are correctly mapped from .smsg to Rust types with 100% accuracy
- **SC-003**: Nested message definitions are correctly resolved and generate valid Rust code with proper type references
- **SC-004**: Parse errors are reported with clear error messages indicating the problematic line and nature of the error
- **SC-005**: Users can successfully generate structs for at least 10 message definitions in a single .smsg file

## Assumptions

- The primary use case is for robotics applications following ROS message conventions
- Users have basic familiarity with Rust and proc macros
- The crate will be published to crates.io for public use
- The .smsg file format follows standard ROS message definition syntax
- The implementation uses the nom parser crate for parsing .smsg files

## Clarifications

### Session 2026-03-02

- Q: Should the spec explicitly require using nom for parsing? → A: Yes - Record nom as a requirement in the specification
