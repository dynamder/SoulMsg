use crate::error::PackageError;
use crate::ir::{Dependency, SmsgPackage};
use toml::Value;

pub fn parse_package_toml(content: &str, _base_path: &str) -> Result<SmsgPackage, PackageError> {
    let value: Value = content
        .parse()
        .map_err(|e: toml::de::Error| PackageError::TomlParse(e.to_string()))?;

    let package_table = value
        .get("package")
        .and_then(|v: &Value| v.as_table())
        .ok_or(PackageError::MissingPackageSection)?;

    let name = package_table
        .get("name")
        .and_then(|v: &Value| v.as_str())
        .ok_or(PackageError::MissingField("name".to_string()))?
        .to_string();

    let version = package_table
        .get("version")
        .and_then(|v: &Value| v.as_str())
        .ok_or(PackageError::MissingField("version".to_string()))?
        .to_string();

    let edition = package_table
        .get("edition")
        .and_then(|v: &Value| v.as_str())
        .ok_or(PackageError::MissingField("edition".to_string()))?
        .to_string();

    if edition != "2026" {
        return Err(PackageError::InvalidEdition(edition));
    }

    let dependencies =
        if let Some(deps_table) = value.get("dependencies").and_then(|v: &Value| v.as_table()) {
            parse_deps_from_table(deps_table)
        } else {
            Vec::new()
        };

    Ok(SmsgPackage {
        name,
        version,
        edition,
        dependencies,
    })
}

#[allow(dead_code)]
pub fn parse_dependencies(deps_toml: &str) -> Vec<Dependency> {
    if deps_toml.trim().is_empty() {
        return Vec::new();
    }

    let value: Value = match deps_toml.parse() {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let deps_table = match value.as_table() {
        Some(t) => t,
        None => return Vec::new(),
    };

    parse_deps_from_table(deps_table)
}

fn parse_deps_from_table(table: &toml::map::Map<String, Value>) -> Vec<Dependency> {
    table
        .iter()
        .map(|(name, value)| {
            let path = if let Some(table) = value.as_table() {
                table
                    .get("path")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_default()
            } else if let Some(version) = value.as_str() {
                version.to_string()
            } else {
                String::new()
            };
            Dependency {
                name: name.clone(),
                path,
            }
        })
        .collect()
}

pub fn walk_package_directory(
    dir_path: &std::path::Path,
) -> std::io::Result<Vec<std::path::PathBuf>> {
    let mut smsg_files = Vec::new();

    if !dir_path.is_dir() {
        return Ok(smsg_files);
    }

    fn visit_dir(
        dir: &std::path::Path,
        files: &mut Vec<std::path::PathBuf>,
    ) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dir(&path, files)?;
            } else if path.extension().is_some_and(|ext| ext == "smsg") {
                files.push(path);
            }
        }
        Ok(())
    }

    visit_dir(dir_path, &mut smsg_files)?;
    Ok(smsg_files)
}

pub fn build_module_structure(
    root_path: &std::path::Path,
    smsg_files: &[std::path::PathBuf],
) -> crate::ir::ModuleStructure {
    use crate::ir::Module;
    use crate::parser::parse_smsg;

    let root_name = root_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("root")
        .to_string();

    let root_path_str = root_path.to_string_lossy().to_string();
    let mut root_module = Module::new(root_name.clone(), root_path_str.clone());

    for smsg_file in smsg_files {
        let relative = smsg_file.strip_prefix(root_path).unwrap_or(smsg_file);
        let parent = relative.parent();

        if let Ok(content) = std::fs::read_to_string(smsg_file) {
            if let Ok(smsg_file_parsed) = parse_smsg(&content) {
                match parent {
                    Some(parent_dir)
                        if parent_dir == std::path::Path::new("")
                            || parent_dir.as_os_str().is_empty() =>
                    {
                        root_module.messages.extend(smsg_file_parsed.messages);
                    }
                    Some(parent_dir) => {
                        let parts: Vec<&str> =
                            parent_dir.iter().filter_map(|p| p.to_str()).collect();
                        add_to_nested_module(&mut root_module, &parts, &smsg_file_parsed.messages);
                    }
                    None => {
                        root_module.messages.extend(smsg_file_parsed.messages);
                    }
                }
            }
        }
    }

    crate::ir::ModuleStructure { root_module }
}

fn add_to_nested_module(
    parent: &mut crate::ir::Module,
    path_parts: &[&str],
    messages: &[crate::ir::MessageDef],
) {
    if path_parts.is_empty() {
        parent.messages.extend(messages.to_vec());
        return;
    }

    let (first, rest) = path_parts.split_first().unwrap();

    if let Some(child) = parent.children.iter_mut().find(|m| m.name == *first) {
        add_to_nested_module(child, rest, messages);
    } else {
        let child_path = format!("{}/{}", parent.path, first);
        let mut new_module = crate::ir::Module::new(first.to_string(), child_path);
        add_to_nested_module(&mut new_module, rest, messages);
        parent.children.push(new_module);
    }
}

#[cfg(test)]
mod package_tests {
    use super::*;

    #[test]
    fn test_parse_valid_package_toml() {
        let toml_content = r#"
[package]
name = "mypackage"
version = "1.0.0"
edition = "2026"
"#;
        let result = parse_package_toml(toml_content, "tests/fixtures/packages/test_pkg");
        assert!(result.is_ok());
        let pkg = result.unwrap();
        assert_eq!(pkg.name, "mypackage");
        assert_eq!(pkg.version, "1.0.0");
        assert_eq!(pkg.edition, "2026");
    }

    #[test]
    fn test_parse_package_toml_with_dependencies() {
        let toml_content = r#"
[package]
name = "mypackage"
version = "1.0.0"
edition = "2026"

[dependencies]
dep1 = { path = "../dep1" }
dep2 = { path = "./sub/dep2" }
"#;
        let result = parse_package_toml(toml_content, "tests/fixtures/packages/test_pkg");
        assert!(result.is_ok());
        let pkg = result.unwrap();
        assert_eq!(pkg.dependencies.len(), 2);
        assert_eq!(pkg.dependencies[0].name, "dep1");
        assert_eq!(pkg.dependencies[0].path, "../dep1");
        assert_eq!(pkg.dependencies[1].name, "dep2");
        assert_eq!(pkg.dependencies[1].path, "./sub/dep2");
    }

    #[test]
    fn test_parse_package_toml_no_dependencies() {
        let toml_content = r#"
[package]
name = "mypackage"
version = "1.0.0"
edition = "2026"
"#;
        let result = parse_package_toml(toml_content, "tests/fixtures/packages/test_pkg");
        assert!(result.is_ok());
        let pkg = result.unwrap();
        assert!(pkg.dependencies.is_empty());
    }

    #[test]
    fn test_error_missing_package_section() {
        let toml_content = r#"
[dependencies]
someDep = { path = "../dep" }
"#;
        let result = parse_package_toml(toml_content, "tests/fixtures/packages/test_pkg");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_missing_required_fields() {
        let toml_content = r#"
[package]
name = "mypackage"
"#;
        let result = parse_package_toml(toml_content, "tests/fixtures/packages/test_pkg");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_dependencies() {
        let deps_toml = r#"
dep1 = { path = "../dep1" }
dep2 = { path = "./sub/dep2" }
dep3 = "3.0.0"
"#;
        let deps = parse_dependencies(deps_toml);
        assert_eq!(deps.len(), 3);
        assert_eq!(deps[0].name, "dep1");
        assert_eq!(deps[0].path, "../dep1");
        assert_eq!(deps[1].name, "dep2");
        assert_eq!(deps[1].path, "./sub/dep2");
    }

    #[test]
    fn test_is_valid_rust_identifier() {
        assert!(is_valid_rust_identifier("mymodule"));
        assert!(is_valid_rust_identifier("my_module"));
        assert!(is_valid_rust_identifier("module1"));
        assert!(!is_valid_rust_identifier("1module"));
        assert!(!is_valid_rust_identifier("my-module"));
        assert!(!is_valid_rust_identifier("my module"));
    }

    #[test]
    fn test_error_invalid_edition() {
        let toml_content = r#"
[package]
name = "mypackage"
version = "1.0.0"
edition = "2024"
"#;
        let result = parse_package_toml(toml_content, "tests/fixtures/packages/test_pkg");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = err.to_string();
        assert!(err_str.contains("edition") || err_str.contains("2026"));
    }

    #[test]
    fn test_error_toml_parse_failure() {
        let toml_content = r#"
[package
name = "mypackage"
"#;
        let result = parse_package_toml(toml_content, "tests/fixtures/packages/test_pkg");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = err.to_string();
        assert!(err_str.contains("TOML") || err_str.contains("parse"));
    }

    #[test]
    fn test_error_missing_version() {
        let toml_content = r#"
[package]
name = "mypackage"
edition = "2026"
"#;
        let result = parse_package_toml(toml_content, "tests/fixtures/packages/test_pkg");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = err.to_string();
        assert!(err_str.contains("version") || err_str.contains("Missing"));
    }

    #[test]
    fn test_error_missing_name() {
        let toml_content = r#"
[package]
version = "1.0.0"
edition = "2026"
"#;
        let result = parse_package_toml(toml_content, "tests/fixtures/packages/test_pkg");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = err.to_string();
        assert!(err_str.contains("name") || err_str.contains("Missing"));
    }
}

#[allow(dead_code)]
pub fn is_valid_rust_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let mut chars = name.chars();
    let first = chars.next().unwrap();

    if !first.is_alphabetic() && first != '_' {
        return false;
    }

    chars.all(|c| c.is_alphanumeric() || c == '_')
}
