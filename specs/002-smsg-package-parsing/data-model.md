# Data Model: Smsg Macro Attribute Parsing

## Entities

### SmsgAttribute
Represents the parsed attributes from `#[smsg(...)]`

| Field | Type | Validation | Description |
|-------|------|------------|-------------|
| `category` | enum | `file` or `package` | Specifies whether to parse a single file or a package |
| `path` | String | Required, valid path | Path to .smsg file or package directory |

### SmsgPackage (for package category)
Represents a parsed SoulSmsg Package

| Field | Type | Validation | Description |
|-------|------|------------|-------------|
| `name` | String | Required, valid package name | Package identifier |
| `version` | String | Required, semver | Package version |
| `edition` | String | Fixed to "2026" | Package edition |
| `dependencies` | Vec<Dependency> | Optional | List of package dependencies |

### Dependency
Represents a package dependency from package.toml

| Field | Type | Validation | Description |
|-------|------|------------|-------------|
| `name` | String | Required | Dependency name (lib name) |
| `path` | String | Required, relative path | Path to dependency package |

### ModuleStructure
Represents the generated Rust module hierarchy mirroring package folder structure (FR-013)

| Field | Type | Validation | Description |
|-------|------|------------|-------------|
| `root_module` | Module | Required | Root module for the package |
| `submodules` | Vec<Module> | Optional | Nested modules for subdirectories |

### Module
Represents a single Rust module (generated from folder or file)

| Field | Type | Validation | Description |
|-------|------|------------|-------------|
| `name` | String | Required, valid Rust identifier | Module name (folder/file name) |
| `path` | String | Required | Relative path in package |
| `messages` | Vec<Message> | Optional | Message types in this module |
| `children` | Vec<Module> | Optional | Nested submodules |

## State Transitions

```
User writes #[smsg(...)]
        ↓
Parse attributes (string or named)
        ↓
Determine source_type (default: "file")
        ↓
Load source (file or package directory)
        ↓
Parse .smsg content(s)
        ↓
Build ModuleStructure (mirror folder structure) ← NEW: FR-013
        ↓
Generate Rust structs + modules
```

## Validation Rules

1. **Attribute Syntax**:
   - String: `#[smsg("path")]` → `category=file`, `path="path"`
   - Named: `#[smsg(category = file, path = "path")]` → parsed accordingly
   
2. **Category Validation**:
   - `category` must be identifier `file` or `package` (no quotes)
   - If `category` is omitted, default to `file` for backward compatibility
   
3. **Path Validation**:
   - Path must be non-empty string literal
   - For `file` category: path must point to a .smsg file
   - For `package` category: path must point to a directory containing package.toml

4. **Module Structure Validation (FR-013)**:
   - Each subdirectory in the package becomes a nested `mod` in Rust
   - Module names must be valid Rust identifiers (alphanumeric + underscore)
   - Root module contains messages from .smsg files at package root
   - Nested modules contain messages from .smsg files in corresponding subdirectories
