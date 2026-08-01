use std::fmt;

#[derive(Debug)]
pub struct SmsgParseError {
    pub message: String,
    pub line: usize,
    pub col: usize,
    pub source: Option<String>,
}

impl SmsgParseError {
    pub fn new(message: String, line: usize, col: usize) -> Self {
        Self {
            message,
            line,
            col,
            source: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_source(message: String, line: usize, col: usize, source: String) -> Self {
        Self {
            message,
            line,
            col,
            source: Some(source),
        }
    }

    #[allow(dead_code)]
    pub fn file_not_found(path: &str) -> Self {
        Self {
            message: format!("File not found: {}", path),
            line: 0,
            col: 0,
            source: None,
        }
    }

    #[allow(dead_code)]
    pub fn invalid_type(type_name: &str, line: usize, col: usize) -> Self {
        Self {
            message: format!("Invalid type: {}", type_name),
            line,
            col,
            source: None,
        }
    }

    pub fn duplicate_message(name: &str, line_num: usize) -> Self {
        Self {
            message: format!("Duplicate message definition: {}", name),
            line: line_num,
            col: 0,
            source: None,
        }
    }

    #[allow(dead_code)]
    pub fn to_compile_error(&self) -> String {
        if self.line > 0 {
            format!(
                "error: {} at line {}, column {}",
                self.message, self.line, self.col
            )
        } else {
            format!("error: {}", self.message)
        }
    }
}

impl fmt::Display for SmsgParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line > 0 {
            write!(
                f,
                "Parse error at line {}, column {}: {}",
                self.line, self.col, self.message
            )
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for SmsgParseError {}
