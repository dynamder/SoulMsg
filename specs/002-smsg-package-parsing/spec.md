# Feature Specification: SoulSmsg Package Dependency Parsing

**Feature Branch**: `002-smsg-package-parsing`  
**Created**: 2026-03-07  
**Status**: Draft  
**Input**: User description: "we now have an mvp project. I want to add package parsing for the soul_smsg. Each SoulSmsg Package will contain a package.toml. under the dependencies section, use example_lib = "path/to/package_folder" to specify a dependency. The module path for a package should reflect the actual package folder. In the .smsg file, use "import package.module_path.msg_type" to use a dependency."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Define Package Metadata and Dependencies (Priority: P1)

A package author creates a SoulSmsg Package with a package.toml file that specifies package metadata (name, version, edition) and dependencies on other packages using relative paths.

**Why this priority**: This is the foundational capability that enables code reuse across packages.

**Independent Test**: Can be tested by creating a package with package.toml containing package metadata and dependencies and verifying the file is parsed correctly.

**Acceptance Scenarios**:

1. **Given** a new package folder with package.toml containing [package] section with name, version (e.g., "1.0.0"), and edition (fixed to "2026"), **When** parsed, **Then** the parser extracts package name, version, and edition correctly.
2. **Given** a new package folder with package.toml, **When** the file contains a dependencies section with "example_lib = path/to/package_folder", **Then** the parser extracts the dependency name and path correctly.
3. **Given** a package.toml with multiple dependencies, **When** parsed, **Then** all dependencies are extracted as a list with their names and paths.
4. **Given** a package.toml without dependencies section, **When** parsed, **Then** dependencies are treated as empty.
5. **Given** a package.toml with missing or invalid [package] section, **When** parsed, **Then** an appropriate error is generated.

---

### User Story 2 - Import Message Types from Dependencies (Priority: P1)

A message author uses the import syntax in an .smsg file to reference message types defined in dependency packages.

**Why this priority**: This is the core functionality that enables message type reuse across packages.

**Independent Test**: Can be tested by creating an .smsg file with import statements and verifying the parser resolves the correct package and module path.

**Acceptance Scenarios**:

1. **Given** an .smsg file with "import package.module_path.msg_type", **When** parsed, **Then** the system identifies the package name, module path, and message type.
2. **Given** an import statement referencing a valid dependency, **When** resolved, **Then** the referenced message type from that package is available as a field type.
3. **Given** an import statement with invalid package name, **When** resolved, **Then** an appropriate error is generated.
4. **Given** a message field with type "module.msg_type", **When** resolved, **Then** the system looks up the type from the imported package's exports.

---

### User Story 3 - Resolve Module Paths from Package Structure (Priority: P2)

The system maps package folder structure to module paths, allowing the import syntax to reflect actual folder organization. The generated Rust code should mirror this structure.

**Why this priority**: This provides intuitive, discoverable module paths based on how packages are organized on disk.

**Independent Test**: Can be tested by creating packages in nested folders and verifying module paths match the folder structure.

**Acceptance Scenarios**:

1. **Given** a package in folder "mypackage/nested/", **When** module path is determined, **Then** it resolves to "mypackage.nested".
2. **Given** a package with subdirectories "messages/" and "nested/", **When** Rust code is generated, **Then** the Rust modules are generated as `mod messages { ... }` and `mod nested { ... }` mirroring the folder structure.

---

### Edge Cases

- What happens when package.toml has malformed TOML syntax?
- How does the system handle circular dependencies between packages?
- What happens when the dependency path points to a non-existent folder?
- How are conflicting message type names handled when imported from multiple packages?
- What happens when the import statement has incorrect syntax?
- What happens when an import references a message type that doesn't exist in the target package?
- Can a package reference files outside its root directory? (No - packages are sandboxed to their root)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST parse package.toml files to extract the [package] section with name, version, and edition fields
- **FR-002**: System MUST validate that edition field is set to "2026" (fixed value for initial version)
- **FR-003**: System MUST parse package.toml files to extract the dependencies section
- **FR-004**: System MUST support dependency declarations in format "lib_name = path/to/package_folder"
- **FR-005**: System MUST extract multiple dependencies from a single package.toml
- **FR-006**: System MUST parse import statements in .smsg files with format "import package.module_path.msg_type"
- **FR-007**: System MUST resolve import statements to locate the correct package and message type, making the type available for use as a field type
- **FR-008**: System MUST derive module paths from the actual package folder structure
- **FR-009**: System MUST provide clear error messages when dependencies or imports cannot be resolved
- **FR-010**: System MUST resolve dependency paths relative to the package.toml file's directory
- **FR-011**: System MUST enforce package sandboxing (packages cannot access files outside their root directory)
- **FR-012**: System MUST handle packages without dependencies gracefully
- **FR-013**: System MUST generate Rust modules that mirror the smsg package folder structure (each subdirectory becomes a nested module)

### Key Entities

- **Package**: A folder containing a package.toml file that defines the package metadata (name, version, edition) and dependencies
- **Package Metadata**: The [package] section containing name, version, and edition (fixed to "2026")
- **Dependency**: A reference in package.toml to another package using name = path format
- **Import Statement**: A line in .smsg file that references a message type from another package
- **Module Path**: The dotted path derived from package folder structure (e.g., "my_package.nested.module")
- **Message Type**: A defined message type within a package that can be imported and used
- **Generated Module Structure**: The Rust module hierarchy generated from the package folder structure (subdirectories → nested modules)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Package authors can define dependencies in package.toml and have them parsed correctly 100% of the time
- **SC-002**: Import statements in .smsg files are correctly parsed and resolved to the appropriate package and message type
- **SC-003**: Module paths accurately reflect the folder structure of packages
- **SC-004**: All resolution failures produce user-friendly error messages within 1 second of attempting resolution

## Clarifications

### Session 2026-03-07

- Q: How are imported message types consumed in the .smsg file? → A: Imported types become available as field types; use "import package.module_path.msg_type" then reference as "module.msg_type" in field definitions
- Q: Dependency paths in package.toml are relative to what? → A: Paths are relative to the package.toml file's directory; package root is where package.toml exists; packages cannot access files outside their root
- Q: How should the generated Rust module structure map to the smsg package folder structure? → A: The Rust module structure should mirror the package folder structure; each subdirectory becomes a nested module matching the folder name
