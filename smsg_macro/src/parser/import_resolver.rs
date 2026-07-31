use crate::error::ImportError;
use crate::ir::{Dependency, ImportStatement};
use std::path::PathBuf;

#[allow(dead_code)]
pub struct ImportResolver {
    package_root: PathBuf,
    dependencies: Vec<Dependency>,
}

impl ImportResolver {
    #[allow(dead_code)]
    pub fn new(package_root: PathBuf, dependencies: Vec<Dependency>) -> Self {
        Self {
            package_root,
            dependencies,
        }
    }

    #[allow(dead_code)]
    pub fn resolve(&self, import: &ImportStatement) -> Result<ResolvedImport, ImportError> {
        self.validate_package_name(&import.package)?;

        let dep_path = self.find_dependency_path(&import.package)?;

        let full_path = self.package_root.join(&dep_path);

        if !full_path.exists() {
            return Err(ImportError::UnresolvableImport(format!(
                "Package '{}' not found at path '{}'",
                import.package, dep_path
            )));
        }

        let module_path = if import.module_path.is_empty() {
            Vec::new()
        } else {
            import.module_path.clone()
        };

        Ok(ResolvedImport {
            package_name: import.package.clone(),
            package_path: full_path,
            module_path,
            message_type: import.message_type.clone(),
        })
    }

    #[allow(dead_code)]
    fn validate_package_name(&self, name: &str) -> Result<(), ImportError> {
        if name.is_empty() {
            return Err(ImportError::InvalidPackageName(
                "Package name cannot be empty".to_string(),
            ));
        }

        let first_char = name.chars().next().unwrap();
        if !first_char.is_ascii_lowercase() {
            return Err(ImportError::InvalidPackageName(format!(
                "Package name '{}' must start with a lowercase letter",
                name
            )));
        }

        for c in name.chars() {
            if !c.is_alphanumeric() && c != '_' {
                return Err(ImportError::InvalidPackageName(format!(
                    "Package name '{}' contains invalid character '{}'",
                    name, c
                )));
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    fn find_dependency_path(&self, package_name: &str) -> Result<String, ImportError> {
        for dep in &self.dependencies {
            if dep.name == package_name {
                return Ok(dep.path.clone());
            }
        }

        Err(ImportError::UnresolvableImport(format!(
            "Unknown package: {}",
            package_name
        )))
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ResolvedImport {
    pub package_name: String,
    pub package_path: PathBuf,
    pub module_path: Vec<String>,
    pub message_type: String,
}

impl ResolvedImport {
    #[allow(dead_code)]
    pub fn get_smsg_file_path(&self) -> Option<PathBuf> {
        if self.module_path.is_empty() {
            return None;
        }

        let mut path = self.package_path.clone();
        for part in &self.module_path {
            path = path.join(part);
        }

        if !self.message_type.is_empty() {
            path = path.join(format!("{}.smsg", self.message_type));
        }

        Some(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_import;

    #[test]
    fn test_parse_valid_import() {
        let result = parse_import("import mypackage.module.MessageType");
        assert!(result.is_ok());
        let imp = result.unwrap();
        assert_eq!(imp.package, "mypackage");
        assert_eq!(imp.module_path, vec!["module"]);
        assert_eq!(imp.message_type, "MessageType");
    }

    #[test]
    fn test_parse_import_nested_module() {
        let result = parse_import("import pkg.sub.sub2.Msg");
        assert!(result.is_ok());
        let imp = result.unwrap();
        assert_eq!(imp.package, "pkg");
        assert_eq!(imp.module_path, vec!["sub", "sub2"]);
        assert_eq!(imp.message_type, "Msg");
    }

    #[test]
    fn test_parse_import_single_level() {
        let result = parse_import("import mypackage.Msg");
        assert!(result.is_ok());
        let imp = result.unwrap();
        assert_eq!(imp.package, "mypackage");
        assert_eq!(imp.module_path, Vec::<String>::new());
        assert_eq!(imp.message_type, "Msg");
    }

    #[test]
    fn test_parse_import_deeply_nested() {
        let result = parse_import("import a.b.c.d.e.F");
        assert!(result.is_ok());
        let imp = result.unwrap();
        assert_eq!(imp.package, "a");
        assert_eq!(imp.module_path, vec!["b", "c", "d", "e"]);
        assert_eq!(imp.message_type, "F");
    }

    #[test]
    fn test_parse_import_without_message_type_fails() {
        let result = parse_import("import mypackage.");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_import_with_underscore_in_names() {
        let result = parse_import("import my_package.sub_module.MyType");
        assert!(result.is_ok());
        let imp = result.unwrap();
        assert_eq!(imp.package, "my_package");
        assert_eq!(imp.module_path, vec!["sub_module"]);
        assert_eq!(imp.message_type, "MyType");
    }

    #[test]
    fn test_parse_import_with_numbers() {
        let result = parse_import("import pkg1.mod2.Type3");
        assert!(result.is_ok());
        let imp = result.unwrap();
        assert_eq!(imp.package, "pkg1");
        assert_eq!(imp.module_path, vec!["mod2"]);
        assert_eq!(imp.message_type, "Type3");
    }

    #[test]
    fn test_parse_import_with_whitespace() {
        let result = parse_import("  import   mypackage.module.Type  ");
        assert!(result.is_ok());
        let imp = result.unwrap();
        assert_eq!(imp.package, "mypackage");
        assert_eq!(imp.module_path, vec!["module"]);
        assert_eq!(imp.message_type, "Type");
    }

    #[test]
    fn test_error_invalid_package_name() {
        let resolver = ImportResolver::new(PathBuf::from("."), vec![]);

        let invalid_import = ImportStatement {
            package: "InvalidPackage".to_string(),
            module_path: Vec::new(),
            message_type: "Msg".to_string(),
        };

        let result = resolver.resolve(&invalid_import);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_invalid_package_name_special_chars() {
        let resolver = ImportResolver::new(PathBuf::from("."), vec![]);

        let invalid_import = ImportStatement {
            package: "my-package".to_string(),
            module_path: vec![],
            message_type: "Msg".to_string(),
        };

        let result = resolver.resolve(&invalid_import);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_empty_package_name() {
        let resolver = ImportResolver::new(PathBuf::from("."), vec![]);

        let invalid_import = ImportStatement {
            package: "".to_string(),
            module_path: vec![],
            message_type: "Msg".to_string(),
        };

        let result = resolver.resolve(&invalid_import);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_starts_with_number() {
        let resolver = ImportResolver::new(PathBuf::from("."), vec![]);

        let invalid_import = ImportStatement {
            package: "1pkg".to_string(),
            module_path: vec![],
            message_type: "Msg".to_string(),
        };

        let result = resolver.resolve(&invalid_import);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_unresolvable_import() {
        let resolver = ImportResolver::new(PathBuf::from("."), vec![]);

        let import = ImportStatement {
            package: "unknown_pkg".to_string(),
            module_path: vec!["mod".to_string()],
            message_type: "Msg".to_string(),
        };

        let result = resolver.resolve(&import);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolver_with_valid_dependency() {
        let deps = vec![Dependency {
            name: "testpkg".to_string(),
            path: "../testpkg".to_string(),
        }];
        let resolver =
            ImportResolver::new(PathBuf::from("tests/fixtures/packages/dependentpkg"), deps);

        let import = ImportStatement {
            package: "testpkg".to_string(),
            module_path: vec!["person".to_string()],
            message_type: "Person".to_string(),
        };

        let result = resolver.resolve(&import);
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert_eq!(resolved.package_name, "testpkg");
    }

    #[test]
    fn test_resolver_finds_dependency_path() {
        let deps = vec![
            Dependency {
                name: "dep1".to_string(),
                path: "./deps/dep1".to_string(),
            },
            Dependency {
                name: "testpkg".to_string(),
                path: "../testpkg".to_string(),
            },
        ];
        let resolver =
            ImportResolver::new(PathBuf::from("tests/fixtures/packages/dependentpkg"), deps);

        let import = ImportStatement {
            package: "testpkg".to_string(),
            module_path: vec!["person".to_string()],
            message_type: "Person".to_string(),
        };

        let result = resolver.resolve(&import);
        assert!(result.is_ok());
    }
}
