use std::collections::HashMap;
use std::ffi::CString;
use std::fmt;

use smsg_core::hash::{compute_message_hash, compute_name_hash};
use smsg_core::ir::{FieldType, MessageDef, PrimitiveType, SmsgFile};
use zenoh::bytes::ZBytes;
use zenoh_ext::{Deserialize, Serialize, VarInt, ZDeserializer, ZSerializer};

/// A dynamic, schema-agnostic message value. Mirrors the supported `.smsg` types.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Str(String),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Bool(bool),
    Array(Vec<Value>),
    Message(Vec<Value>),
}

#[derive(Debug, Clone)]
pub struct MessageSchema {
    pub def: MessageDef,
    pub name_hash: [u8; 32],
    pub version_hash: [u8; 32],
}

/// A parsed `.smsg` file with pre-computed hashes and C-friendly name lookups.
pub struct Schema {
    pub file: SmsgFile,
    pub messages: Vec<MessageSchema>,
    index: HashMap<String, usize>,
    /// NUL-terminated message names aligned with `messages`.
    pub message_names: Vec<CString>,
    /// NUL-terminated field names, indexed `[message][field]`.
    pub field_names: Vec<Vec<CString>>,
}

impl Schema {
    pub fn from_file(file: SmsgFile) -> Self {
        let messages: Vec<MessageSchema> = file
            .messages
            .iter()
            .map(|def| MessageSchema {
                def: def.clone(),
                name_hash: compute_name_hash(&def.name),
                version_hash: compute_message_hash(def),
            })
            .collect();
        let index = messages
            .iter()
            .enumerate()
            .map(|(i, m)| (m.def.name.clone(), i))
            .collect();

        let message_names = messages
            .iter()
            .map(|m| CString::new(m.def.name.as_str()).expect("message name cannot contain NUL"))
            .collect();
        let field_names = messages
            .iter()
            .map(|m| {
                m.def
                    .fields
                    .iter()
                    .map(|f| CString::new(f.name.as_str()).expect("field name cannot contain NUL"))
                    .collect()
            })
            .collect();

        Schema {
            file,
            messages,
            index,
            message_names,
            field_names,
        }
    }

    pub fn message_index(&self, name: &str) -> Option<usize> {
        self.index.get(name).copied()
    }

    pub fn message_by_name(&self, name: &str) -> Option<&MessageSchema> {
        self.index.get(name).map(|i| &self.messages[*i])
    }
}

#[derive(Debug)]
pub enum SmsgError {
    InvalidArg(String),
    Parse(smsg_core::SmsgParseError),
    Schema(String),
    ValueCountMismatch {
        expected: usize,
        actual: usize,
    },
    ValueKindMismatch {
        field: String,
        expected: String,
    },
    NoSuchMessage(String),
    TypeMismatch {
        expected: [u8; 32],
        actual: [u8; 32],
    },
    VersionMismatch {
        expected: [u8; 32],
        actual: [u8; 32],
    },
    Deserialize(String),
    NotAnEnvelope(String),
    Panic,
}

impl fmt::Display for SmsgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SmsgError::InvalidArg(msg) => write!(f, "Invalid argument: {}", msg),
            SmsgError::Parse(e) => write!(f, "Parse error: {}", e),
            SmsgError::Schema(msg) => write!(f, "Schema error: {}", msg),
            SmsgError::ValueCountMismatch { expected, actual } => write!(
                f,
                "Value count mismatch: expected {}, got {}",
                expected, actual
            ),
            SmsgError::ValueKindMismatch { field, expected } => {
                write!(
                    f,
                    "Value kind mismatch for field '{}': expected {}",
                    field, expected
                )
            }
            SmsgError::NoSuchMessage(name) => write!(f, "No such message in schema: {}", name),
            SmsgError::TypeMismatch { expected, actual } => write!(
                f,
                "Type mismatch: expected name_hash {:02x?}, got {:02x?}",
                expected, actual
            ),
            SmsgError::VersionMismatch { expected, actual } => write!(
                f,
                "Version mismatch: expected version_hash {:02x?}, got {:02x?}",
                expected, actual
            ),
            SmsgError::Deserialize(msg) => write!(f, "Deserialize error: {}", msg),
            SmsgError::NotAnEnvelope(msg) => write!(f, "Not an envelope: {}", msg),
            SmsgError::Panic => write!(f, "internal panic"),
        }
    }
}

fn kind_name(ft: &FieldType) -> &'static str {
    match ft {
        FieldType::Primitive(p) => p.rust_type(),
        FieldType::Array(..) => "array",
        FieldType::Nested(_) => "message",
    }
}

fn message_def<'a>(schema: &'a Schema, name: &str) -> Result<&'a MessageDef, SmsgError> {
    schema
        .message_by_name(name)
        .map(|m| &m.def)
        .ok_or_else(|| SmsgError::NoSuchMessage(name.to_string()))
}

fn check_message<'a>(schema: &'a Schema, idx: usize) -> Result<&'a MessageSchema, SmsgError> {
    schema
        .messages
        .get(idx)
        .ok_or_else(|| SmsgError::Schema(format!("message index {} out of range", idx)))
}

// ---------------------------------------------------------------------------
// Dynamic serialization (byte-identical to the compile-time zenoh-ext path)
// ---------------------------------------------------------------------------

pub fn serialize_payload(
    schema: &Schema,
    msg_idx: usize,
    values: &[Value],
) -> Result<Vec<u8>, SmsgError> {
    let msg = check_message(schema, msg_idx)?;
    let mut ser = ZSerializer::new();
    serialize_message_fields(&msg.def, values, &mut ser, schema)?;
    Ok(ser.finish().to_bytes().to_vec())
}

pub fn serialize_envelope(
    schema: &Schema,
    msg_idx: usize,
    values: &[Value],
) -> Result<Vec<u8>, SmsgError> {
    let msg = check_message(schema, msg_idx)?;
    let mut ser = ZSerializer::new();
    msg.name_hash.serialize(&mut ser);
    msg.version_hash.serialize(&mut ser);
    serialize_message_fields(&msg.def, values, &mut ser, schema)?;
    Ok(ser.finish().to_bytes().to_vec())
}

fn serialize_message_fields(
    def: &MessageDef,
    values: &[Value],
    ser: &mut ZSerializer,
    schema: &Schema,
) -> Result<(), SmsgError> {
    if values.len() != def.fields.len() {
        return Err(SmsgError::ValueCountMismatch {
            expected: def.fields.len(),
            actual: values.len(),
        });
    }
    for (field, value) in def.fields.iter().zip(values.iter()) {
        serialize_field(&field.field_type, value, ser, schema)?;
    }
    Ok(())
}

fn serialize_field(
    field_type: &FieldType,
    value: &Value,
    ser: &mut ZSerializer,
    schema: &Schema,
) -> Result<(), SmsgError> {
    match (field_type, value) {
        (FieldType::Primitive(PrimitiveType::String), Value::Str(s)) => s.serialize(ser),
        (FieldType::Primitive(PrimitiveType::Int8), Value::I8(v)) => v.serialize(ser),
        (FieldType::Primitive(PrimitiveType::Int16), Value::I16(v)) => v.serialize(ser),
        (FieldType::Primitive(PrimitiveType::Int32), Value::I32(v)) => v.serialize(ser),
        (FieldType::Primitive(PrimitiveType::Int64), Value::I64(v)) => v.serialize(ser),
        (FieldType::Primitive(PrimitiveType::Uint8), Value::U8(v)) => v.serialize(ser),
        (FieldType::Primitive(PrimitiveType::Uint16), Value::U16(v)) => v.serialize(ser),
        (FieldType::Primitive(PrimitiveType::Uint32), Value::U32(v)) => v.serialize(ser),
        (FieldType::Primitive(PrimitiveType::Uint64), Value::U64(v)) => v.serialize(ser),
        (FieldType::Primitive(PrimitiveType::Float32), Value::F32(v)) => v.serialize(ser),
        (FieldType::Primitive(PrimitiveType::Float64), Value::F64(v)) => v.serialize(ser),
        (FieldType::Primitive(PrimitiveType::Bool), Value::Bool(v)) => v.serialize(ser),
        (FieldType::Array(inner, _), Value::Array(items)) => {
            ser.serialize(VarInt(items.len()));
            for item in items {
                serialize_field(inner, item, ser, schema)?;
            }
        }
        (FieldType::Nested(name), Value::Message(fields)) => {
            let def = message_def(schema, name)?;
            serialize_message_fields(def, fields, ser, schema)?;
        }
        (_, v) => {
            return Err(SmsgError::ValueKindMismatch {
                field: value_field_label(v),
                expected: kind_name(field_type).to_string(),
            });
        }
    }
    Ok(())
}

fn value_field_label(v: &Value) -> String {
    match v {
        Value::Str(_) => "string".into(),
        Value::I8(_) => "i8".into(),
        Value::I16(_) => "i16".into(),
        Value::I32(_) => "i32".into(),
        Value::I64(_) => "i64".into(),
        Value::U8(_) => "u8".into(),
        Value::U16(_) => "u16".into(),
        Value::U32(_) => "u32".into(),
        Value::U64(_) => "u64".into(),
        Value::F32(_) => "f32".into(),
        Value::F64(_) => "f64".into(),
        Value::Bool(_) => "bool".into(),
        Value::Array(_) => "array".into(),
        Value::Message(_) => "message".into(),
    }
}

// ---------------------------------------------------------------------------
// Dynamic deserialization
// ---------------------------------------------------------------------------

pub fn deserialize_payload(
    schema: &Schema,
    msg_idx: usize,
    bytes: &[u8],
) -> Result<Vec<Value>, SmsgError> {
    let msg = check_message(schema, msg_idx)?;
    let zbytes = ZBytes::from(bytes.to_vec());
    let mut de = ZDeserializer::new(&zbytes);
    let values = deserialize_message_fields(&msg.def, &mut de, schema)?;
    if !de.done() {
        return Err(SmsgError::NotAnEnvelope(
            "Trailing data after the payload.".to_string(),
        ));
    }
    Ok(values)
}

pub fn deserialize_envelope(
    schema: &Schema,
    msg_idx: usize,
    bytes: &[u8],
) -> Result<Vec<Value>, SmsgError> {
    let msg = check_message(schema, msg_idx)?;
    let zbytes = ZBytes::from(bytes.to_vec());
    let mut de = ZDeserializer::new(&zbytes);

    let actual_name_hash: [u8; 32] = <[u8; 32]>::deserialize(&mut de)
        .map_err(|e| SmsgError::NotAnEnvelope(format!("Failed to read name_hash: {}", e)))?;
    if actual_name_hash != msg.name_hash {
        return Err(SmsgError::TypeMismatch {
            expected: msg.name_hash,
            actual: actual_name_hash,
        });
    }

    let actual_version_hash: [u8; 32] = <[u8; 32]>::deserialize(&mut de)
        .map_err(|e| SmsgError::NotAnEnvelope(format!("Failed to read version_hash: {}", e)))?;
    if actual_version_hash != msg.version_hash {
        return Err(SmsgError::VersionMismatch {
            expected: msg.version_hash,
            actual: actual_version_hash,
        });
    }

    let values = deserialize_message_fields(&msg.def, &mut de, schema)?;
    if !de.done() {
        return Err(SmsgError::NotAnEnvelope(
            "Trailing data after the payload.".to_string(),
        ));
    }
    Ok(values)
}

/// Header layout is exactly `[varint:1][name_hash:32][varint:1][version_hash:32]`.
pub const ENVELOPE_HEADER_LEN: usize = 66;

pub struct PeekResult<'a> {
    pub name_hash: [u8; 32],
    pub version_hash: [u8; 32],
    pub payload: &'a [u8],
}

pub fn peek_envelope(bytes: &[u8]) -> Result<PeekResult<'_>, SmsgError> {
    if bytes.len() < ENVELOPE_HEADER_LEN {
        return Err(SmsgError::NotAnEnvelope(
            "Data too short for an envelope header.".to_string(),
        ));
    }
    let zbytes = ZBytes::from(bytes.to_vec());
    let mut de = ZDeserializer::new(&zbytes);
    let name_hash: [u8; 32] = <[u8; 32]>::deserialize(&mut de)
        .map_err(|e| SmsgError::NotAnEnvelope(format!("Failed to read name_hash: {}", e)))?;
    let version_hash: [u8; 32] = <[u8; 32]>::deserialize(&mut de)
        .map_err(|e| SmsgError::NotAnEnvelope(format!("Failed to read version_hash: {}", e)))?;
    Ok(PeekResult {
        name_hash,
        version_hash,
        payload: &bytes[ENVELOPE_HEADER_LEN..],
    })
}

fn deserialize_message_fields(
    def: &MessageDef,
    de: &mut ZDeserializer,
    schema: &Schema,
) -> Result<Vec<Value>, SmsgError> {
    let mut values = Vec::with_capacity(def.fields.len());
    for field in &def.fields {
        values.push(deserialize_field(&field.field_type, de, schema)?);
    }
    Ok(values)
}

fn deserialize_field(
    field_type: &FieldType,
    de: &mut ZDeserializer,
    schema: &Schema,
) -> Result<Value, SmsgError> {
    match field_type {
        FieldType::Primitive(p) => deserialize_primitive(p, de),
        FieldType::Array(inner, _) => {
            let len = <VarInt<usize>>::deserialize(de)
                .map_err(|e| SmsgError::Deserialize(e.to_string()))?
                .0;
            let mut items = Vec::with_capacity(len);
            for _ in 0..len {
                items.push(deserialize_field(inner, de, schema)?);
            }
            Ok(Value::Array(items))
        }
        FieldType::Nested(name) => {
            let def = message_def(schema, name)?;
            Ok(Value::Message(deserialize_message_fields(def, de, schema)?))
        }
    }
}

fn deserialize_primitive(p: &PrimitiveType, de: &mut ZDeserializer) -> Result<Value, SmsgError> {
    macro_rules! read {
        ($ty:ty, $variant:ident) => {
            <$ty>::deserialize(de)
                .map(Value::$variant)
                .map_err(|e| SmsgError::Deserialize(e.to_string()))
        };
    }
    match p {
        PrimitiveType::String => read!(String, Str),
        PrimitiveType::Int8 => read!(i8, I8),
        PrimitiveType::Int16 => read!(i16, I16),
        PrimitiveType::Int32 => read!(i32, I32),
        PrimitiveType::Int64 => read!(i64, I64),
        PrimitiveType::Uint8 => read!(u8, U8),
        PrimitiveType::Uint16 => read!(u16, U16),
        PrimitiveType::Uint32 => read!(u32, U32),
        PrimitiveType::Uint64 => read!(u64, U64),
        PrimitiveType::Float32 => read!(f32, F32),
        PrimitiveType::Float64 => read!(f64, F64),
        PrimitiveType::Bool => read!(bool, Bool),
    }
}
