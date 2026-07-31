use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct SmsgFile {
    pub messages: Vec<MessageDef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MessageDef {
    pub name: String,
    pub fields: Vec<Field>,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    Primitive(PrimitiveType),
    Array(Box<FieldType>, Option<usize>),
    Nested(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    Int8,
    Int16,
    Int32,
    Int64,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Float32,
    Float64,
    Bool,
    String,
}

impl PrimitiveType {
    pub fn rust_type(&self) -> &'static str {
        match self {
            PrimitiveType::Int8 => "i8",
            PrimitiveType::Int16 => "i16",
            PrimitiveType::Int32 => "i32",
            PrimitiveType::Int64 => "i64",
            PrimitiveType::Uint8 => "u8",
            PrimitiveType::Uint16 => "u16",
            PrimitiveType::Uint32 => "u32",
            PrimitiveType::Uint64 => "u64",
            PrimitiveType::Float32 => "f32",
            PrimitiveType::Float64 => "f64",
            PrimitiveType::Bool => "bool",
            PrimitiveType::String => "String",
        }
    }

    pub fn from_str(s: &str) -> Option<PrimitiveType> {
        match s {
            "int8" => Some(PrimitiveType::Int8),
            "int16" => Some(PrimitiveType::Int16),
            "int32" => Some(PrimitiveType::Int32),
            "int64" => Some(PrimitiveType::Int64),
            "uint8" => Some(PrimitiveType::Uint8),
            "uint16" => Some(PrimitiveType::Uint16),
            "uint32" => Some(PrimitiveType::Uint32),
            "uint64" => Some(PrimitiveType::Uint64),
            "float32" => Some(PrimitiveType::Float32),
            "float64" => Some(PrimitiveType::Float64),
            "bool" => Some(PrimitiveType::Bool),
            "string" => Some(PrimitiveType::String),
            _ => None,
        }
    }
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rust_type())
    }
}

impl fmt::Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldType::Primitive(p) => write!(f, "{}", p),
            FieldType::Array(inner, size) => {
                if let Some(s) = size {
                    write!(f, "{}[{}]", inner, s)
                } else {
                    write!(f, "{}[]", inner)
                }
            }
            FieldType::Nested(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SmsgPackage {
    pub name: String,
    pub version: String,
    pub edition: String,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Dependency {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleStructure {
    pub root_module: Module,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub name: String,
    pub path: String,
    pub messages: Vec<MessageDef>,
    pub children: Vec<Module>,
}

impl Module {
    pub fn new(name: String, path: String) -> Self {
        Self {
            name,
            path,
            messages: Vec::new(),
            children: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct ImportStatement {
    pub package: String,
    pub module_path: Vec<String>,
    pub message_type: String,
}
