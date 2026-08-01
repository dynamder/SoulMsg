use crate::error::SmsgParseError;
use crate::ir::{Field, FieldType, ImportStatement, MessageDef, PrimitiveType, SmsgFile};
use winnow::ascii::{digit0, multispace0, multispace1};
use winnow::combinator::opt;
use winnow::error::{ErrMode, InputError};
use winnow::prelude::*;
use winnow::token::take_while;

type WinnowError<'a> = InputError<&'a str>;
type WinnowResult<'a, T> = Result<T, ErrMode<WinnowError<'a>>>;

/// Compute the 1-based line number of the current position, given the full input and the
/// remaining (unconsumed) slice. Both must be byte-aligned substrings of the same buffer.
fn line_at(full: &str, remaining: &str) -> usize {
    let consumed = full.len().saturating_sub(remaining.len());
    full[..consumed].bytes().filter(|b| *b == b'\n').count() + 1
}

pub fn parse_smsg(input: &str) -> Result<SmsgFile, SmsgParseError> {
    let trimmed = input.trim();
    let mut remaining = trimmed;
    let mut messages = Vec::new();
    let mut seen_names = std::collections::HashSet::new();

    while !remaining.is_empty() {
        if let Err(e) = multispace0::<&str, InputError<&str>>.parse_next(&mut remaining) {
            return Err(SmsgParseError::new(
                format!("Whitespace error: {:?}", e),
                1,
                1,
            ));
        }

        if remaining.is_empty() {
            break;
        }

        if remaining.starts_with('#') {
            let _: Result<_, ErrMode<InputError<&str>>> =
                take_while(0.., |c| c != '\n').parse_next(&mut remaining);
            continue;
        }

        let line = line_at(trimmed, remaining);
        match parse_message.parse_next(&mut remaining) {
            Ok(msg) => {
                if seen_names.contains(&msg.name) {
                    return Err(SmsgParseError::duplicate_message(&msg.name, line));
                }
                seen_names.insert(msg.name.clone());
                messages.push(msg);
            }
            Err(e) => {
                let err_line = match &e {
                    ErrMode::Backtrack(InputError { input, .. })
                    | ErrMode::Cut(InputError { input, .. }) => line_at(trimmed, input),
                    ErrMode::Incomplete(_) => line,
                };
                return Err(SmsgParseError::new(
                    format!("Parse error: {:?}", e),
                    err_line.max(1),
                    1,
                ));
            }
        }
    }

    Ok(SmsgFile { messages })
}

fn parse_message<'a>(input: &mut &'a str) -> WinnowResult<'a, MessageDef> {
    "message".parse_next(input)?;
    multispace1.parse_next(input)?;
    let name =
        take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_').parse_next(input)?;
    multispace0.parse_next(input)?;
    "{".parse_next(input)?;

    let mut fields = Vec::new();

    loop {
        multispace0.parse_next(input)?;

        if input.starts_with('}') {
            "}".parse_next(input)?;
            break;
        }

        if input.starts_with('#') {
            take_while(0.., |c| c != '\n').parse_next(input)?;
            continue;
        }

        let field = parse_field.parse_next(input)?;
        fields.push(field);
    }

    let line = input.lines().count();
    Ok(MessageDef {
        name: name.to_string(),
        fields,
        line,
        col: 1,
    })
}

fn parse_field<'a>(input: &mut &'a str) -> WinnowResult<'a, Field> {
    let type_str: String = parse_type_token(input)?;

    multispace1.parse_next(input)?;
    let name = take_while(1.., |c| c != ' ' && c != '\t' && c != '\n').parse_next(input)?;
    multispace0.parse_next(input)?;
    opt(";".value(())).parse_next(input)?;
    multispace0.parse_next(input)?;

    let line = input.lines().count();
    let field_type =
        parse_field_type(&type_str).map_err(|_e| ErrMode::Backtrack(InputError::at(*input)))?;

    Ok(Field {
        name: name.to_string(),
        field_type,
        line,
        col: 1,
    })
}

fn parse_type_token<'a>(input: &mut &'a str) -> WinnowResult<'a, String> {
    let base_type: String = take_while(1.., |c| c != ' ' && c != '\t' && c != '\n' && c != '[')
        .map(|s: &str| s.to_string())
        .parse_next(input)?;

    if input.starts_with('[') {
        "[".parse_next(input)?;
        let size: Option<&str> = opt(digit0).parse_next(input)?;
        "]".parse_next(input)?;

        Ok(if let Some(s) = size {
            if s.is_empty() {
                format!("{}[]", base_type)
            } else {
                format!("{}[{}]", base_type, s)
            }
        } else {
            format!("{}[]", base_type)
        })
    } else {
        Ok(base_type)
    }
}

fn parse_field_type(type_str: &str) -> Result<FieldType, SmsgParseError> {
    if let Some(arr_start) = type_str.find('[') {
        let arr_end = type_str
            .find(']')
            .ok_or_else(|| SmsgParseError::new("Missing ']'".to_string(), 1, 1))?;
        let base_type = &type_str[..arr_start];
        let size_str = &type_str[arr_start + 1..arr_end];

        let size = if size_str.is_empty() {
            None
        } else {
            Some(
                size_str
                    .parse()
                    .map_err(|_| SmsgParseError::new("Invalid array size".to_string(), 1, 1))?,
            )
        };

        let inner = parse_field_type(base_type)?;
        return Ok(FieldType::Array(Box::new(inner), size));
    }

    if let Some(primitive) = PrimitiveType::from_str(type_str) {
        return Ok(FieldType::Primitive(primitive));
    }

    Ok(FieldType::Nested(type_str.to_string()))
}

#[allow(dead_code)]
pub fn parse_import(input: &str) -> Result<ImportStatement, SmsgParseError> {
    let trimmed = input.trim();
    let mut remaining = trimmed;

    "import"
        .parse_next(&mut remaining)
        .map_err(|_: ErrMode<WinnowError<'_>>| {
            SmsgParseError::new("Expected 'import' keyword".to_string(), 1, 1)
        })?;

    multispace1
        .parse_next(&mut remaining)
        .map_err(|_: ErrMode<WinnowError<'_>>| {
            SmsgParseError::new("Expected whitespace after 'import'".to_string(), 1, 1)
        })?;

    let package: &str = take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_')
        .parse_next(&mut remaining)
        .map_err(|_: ErrMode<WinnowError<'_>>| {
            SmsgParseError::new("Invalid package name".to_string(), 1, 1)
        })?;

    if remaining.is_empty() {
        return Err(SmsgParseError::new(
            "Import must contain package and message type".to_string(),
            1,
            1,
        ));
    }

    if !remaining.starts_with('.') {
        return Err(SmsgParseError::new(
            "Expected '.' after package name".to_string(),
            1,
            1,
        ));
    }

    ".".parse_next(&mut remaining)
        .map_err(|_: ErrMode<WinnowError<'_>>| {
            SmsgParseError::new("Expected '.' after package name".to_string(), 1, 1)
        })?;

    if remaining.is_empty() {
        return Err(SmsgParseError::new(
            "Import must contain message type".to_string(),
            1,
            1,
        ));
    }

    let all_parts: Vec<&str> = remaining.split('.').filter(|s| !s.is_empty()).collect();

    if all_parts.is_empty() {
        return Err(SmsgParseError::new(
            "Import must contain message type".to_string(),
            1,
            1,
        ));
    }

    let message_type = all_parts[all_parts.len() - 1].to_string();
    let module_parts: Vec<String> = all_parts[..all_parts.len() - 1]
        .iter()
        .map(|s| s.to_string())
        .collect();

    Ok(ImportStatement {
        package: package.to_string(),
        module_path: module_parts,
        message_type,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_message() {
        let input = r#"message ChatMessage {
    string sender
    string content
    int64 timestamp
}"#;
        let result = parse_smsg(input).unwrap();
        assert_eq!(result.messages.len(), 1);
        let msg = &result.messages[0];
        assert_eq!(msg.name, "ChatMessage");
        assert_eq!(msg.fields.len(), 3);
        assert_eq!(msg.fields[0].name, "sender");
    }

    #[test]
    fn test_parse_message_name_with_digits_and_underscore() {
        let input = r#"message Message2 {
    string a
}

message Robot_State {
    string b
}"#;
        let result = parse_smsg(input).unwrap();
        assert_eq!(result.messages.len(), 2);
        assert_eq!(result.messages[0].name, "Message2");
        assert_eq!(result.messages[1].name, "Robot_State");
    }

    #[test]
    fn test_parse_array_type() {
        let input = r#"message Test {
    float64[] values
    int32[3] point
}"#;
        let result = parse_smsg(input).unwrap();
        assert_eq!(result.messages.len(), 1);
    }

    #[test]
    fn test_parse_mixed_array_and_scalar_fields() {
        let input = r#"message Mixed {
    int32 id
    float64[] values
    string name
    int32[3] point
    bool active
}

message Second {
    string tag
    uint8[4] mac
}"#;
        let result = parse_smsg(input).unwrap();
        assert_eq!(result.messages.len(), 2);

        let mixed = &result.messages[0];
        assert_eq!(mixed.fields.len(), 5);
        assert!(matches!(
            mixed.fields[0].field_type,
            FieldType::Primitive(PrimitiveType::Int32)
        ));
        assert!(matches!(mixed.fields[1].field_type, FieldType::Array(..)));
        assert!(matches!(
            mixed.fields[2].field_type,
            FieldType::Primitive(PrimitiveType::String)
        ));
        assert!(matches!(mixed.fields[3].field_type, FieldType::Array(..)));
        assert!(matches!(
            mixed.fields[4].field_type,
            FieldType::Primitive(PrimitiveType::Bool)
        ));

        let second = &result.messages[1];
        assert_eq!(second.fields.len(), 2);
        assert!(matches!(second.fields[1].field_type, FieldType::Array(..)));
    }

    #[test]
    fn test_parse_nested_type() {
        let input = r#"message Position {
    float64 x
    float64 y
}

message RobotState {
    string name
    Position position
}"#;
        let result = parse_smsg(input).unwrap();
        assert_eq!(result.messages.len(), 2);
    }

    #[test]
    fn test_parse_error_invalid_keyword() {
        let input = r#"msg ChatMessage {
    string sender
}"#;
        let result = parse_smsg(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Parse error") || err.message.contains("message"));
        assert!(err.line > 0);
    }

    #[test]
    fn test_parse_error_missing_brace() {
        let input = r#"message ChatMessage 
    string sender
}"#;
        let result = parse_smsg(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.line > 0);
    }

    #[test]
    fn test_parse_error_missing_field_name() {
        let input = r#"message ChatMessage {
    string 
}"#;
        let result = parse_smsg(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.line > 0);
    }

    #[test]
    fn test_parse_error_unclosed_bracket() {
        let input = r#"message ChatMessage {
    string[ sender
}"#;
        let result = parse_smsg(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.line > 0);
    }

    #[test]
    fn test_parse_error_empty_message_name() {
        let input = r#"message  {
    string sender
}"#;
        let result = parse_smsg(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_invalid_array_size() {
        let input = r#"message ChatMessage {
    string[abc] field1
}"#;
        let result = parse_smsg(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("array") || err.message.contains("Parse error"));
    }

    #[test]
    fn test_parse_error_missing_closing_bracket() {
        let input = r#"message ChatMessage {
    string sender"#;
        let result = parse_smsg(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.line > 0);
    }

    #[test]
    fn test_error_message_contains_line_info() {
        let input = r#"message TestMessage {
    invalid_syntax here is some more text to trigger error
}"#;
        let result = parse_smsg(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = err.to_string();
        assert!(err_str.contains("line") || err_str.contains("column"));
    }

    #[test]
    fn test_parse_error_duplicate_message() {
        let input = r#"message ChatMessage {
    string sender
}

message ChatMessage {
    string content
}"#;
        let result = parse_smsg(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Duplicate") || err.message.contains("already exists"));
    }

    #[test]
    fn test_error_line_number_is_accurate() {
        let input = r#"message Good {
    string a
}

message Bad {
    string
}"#;
        let result = parse_smsg(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        // The parser stops at the '}' that terminates the malformed field.
        assert_eq!(err.line, 7, "expected error at line 7, got {}", err.line);
    }

    #[test]
    fn test_duplicate_error_line_number_is_accurate() {
        let input = r#"message Foo {
    string a
}

message Foo {
    string b
}"#;
        let result = parse_smsg(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.line, 5,
            "expected duplicate at line 5, got {}",
            err.line
        );
    }
}
