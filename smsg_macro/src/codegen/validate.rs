use crate::ir::{Field, MessageDef, Module, ModuleStructure, SmsgFile};

const RUST_KEYWORDS: &[&str] = &[
    "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "crate",
    "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "if", "impl", "in",
    "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
    "return", "Self", "self", "static", "struct", "super", "trait", "true", "try", "type",
    "typeof", "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
];

pub fn validate_ident(name: &str, kind: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err(format!("{} name cannot be empty", kind));
    }

    let mut chars = name.chars();
    let first = chars.next().unwrap();
    if first != '_' && !first.is_alphabetic() {
        return Err(format!(
            "Invalid {} name '{}': must start with a letter or underscore",
            kind, name
        ));
    }

    if !chars.all(|c| c.is_alphanumeric() || c == '_') {
        return Err(format!(
            "Invalid {} name '{}': only alphanumeric characters and underscores are allowed",
            kind, name
        ));
    }

    if RUST_KEYWORDS.contains(&name) {
        return Err(format!(
            "Invalid {} name '{}': is a reserved Rust keyword",
            kind, name
        ));
    }

    syn::parse_str::<syn::Ident>(name)
        .map(|_| ())
        .map_err(|_| format!("Invalid {} name '{}'", kind, name))
}

pub fn validate_field(field: &Field, message_name: &str) -> Result<(), String> {
    validate_ident(&field.name, "field")
        .map_err(|e| format!("In message '{}', {}", message_name, e))
}

pub fn validate_message(message: &MessageDef) -> Result<(), String> {
    validate_ident(&message.name, "message")?;
    for field in &message.fields {
        validate_field(field, &message.name)?;
    }
    Ok(())
}

pub fn validate_smsg_file(file: &SmsgFile) -> Result<(), String> {
    for message in &file.messages {
        validate_message(message)?;
    }
    Ok(())
}

pub fn validate_module(module: &Module) -> Result<(), String> {
    validate_ident(&module.name, "module")?;
    for message in &module.messages {
        validate_message(message)?;
    }
    for child in &module.children {
        validate_module(child)?;
    }
    Ok(())
}

pub fn validate_module_structure(structure: &ModuleStructure) -> Result<(), String> {
    let root = &structure.root_module;
    for message in &root.messages {
        validate_message(message)?;
    }
    for child in &root.children {
        validate_module(child)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_identifiers() {
        assert!(validate_ident("ChatMessage", "message").is_ok());
        assert!(validate_ident("sender", "field").is_ok());
        assert!(validate_ident("_private", "field").is_ok());
        assert!(validate_ident("Robot_State", "message").is_ok());
        assert!(validate_ident("Message2", "message").is_ok());
    }

    #[test]
    fn test_invalid_identifiers() {
        assert!(validate_ident("", "message").is_err());
        assert!(validate_ident("2bad", "message").is_err());
        assert!(validate_ident("my-field", "field").is_err());
        assert!(validate_ident("has space", "field").is_err());
    }

    #[test]
    fn test_rust_keywords_rejected() {
        for kw in ["type", "struct", "match", "fn", "impl", "Self"] {
            assert!(
                validate_ident(kw, "field").is_err(),
                "keyword '{}' not rejected",
                kw
            );
        }
    }

    #[test]
    fn test_message_with_keyword_field_fails() {
        let file = SmsgFile {
            messages: vec![MessageDef {
                name: "ChatMessage".to_string(),
                fields: vec![Field {
                    name: "type".to_string(),
                    field_type: crate::ir::FieldType::Primitive(crate::ir::PrimitiveType::String),
                    line: 1,
                    col: 1,
                }],
                line: 1,
                col: 1,
            }],
        };
        assert!(validate_smsg_file(&file).is_err());
    }
}
