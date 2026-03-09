pub use smsg_macro::smsg;

pub trait MessageMeta {
    fn version_hash() -> [u8; 32];
    fn name_hash() -> [u8; 32];
    fn message_name() -> &'static str;
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnvelopeError {
    NotAnEnvelope(String),
    TypeMismatch {
        expected_name_hash: [u8; 32],
        actual_name_hash: [u8; 32],
    },
    VersionMismatch {
        expected_version_hash: [u8; 32],
        actual_version_hash: [u8; 32],
    },
    DeserializeError(String),
}

impl std::fmt::Display for EnvelopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnvelopeError::NotAnEnvelope(msg) => {
                write!(f, "Not an envelope: {}", msg)
            }
            EnvelopeError::TypeMismatch {
                expected_name_hash,
                actual_name_hash,
            } => {
                write!(
                    f,
                    "Type mismatch: expected name_hash {:02x?}, got {:02x?}",
                    expected_name_hash, actual_name_hash
                )
            }
            EnvelopeError::VersionMismatch {
                expected_version_hash,
                actual_version_hash,
            } => {
                write!(
                    f,
                    "Version mismatch: expected version_hash {:02x?}, got {:02x?}",
                    expected_version_hash, actual_version_hash
                )
            }
            EnvelopeError::DeserializeError(msg) => {
                write!(f, "Deserialize error: {}", msg)
            }
        }
    }
}

impl std::error::Error for EnvelopeError {}

#[derive(Debug, Clone, PartialEq)]
pub struct SmsgEnvelope<T> {
    version_hash: [u8; 32],
    name_hash: [u8; 32],
    pub payload: T,
}

impl<T: MessageMeta + zenoh_ext::Deserialize> SmsgEnvelope<T> {
    pub fn new(payload: T) -> Self {
        Self {
            version_hash: T::version_hash(),
            name_hash: T::name_hash(),
            payload,
        }
    }

    pub fn into_parts(self) -> ([u8; 32], [u8; 32], T) {
        (self.version_hash, self.name_hash, self.payload)
    }

    pub fn into_payload(self) -> T {
        self.payload
    }

    pub fn version_hash(&self) -> &[u8; 32] {
        &self.version_hash
    }

    pub fn name_hash(&self) -> &[u8; 32] {
        &self.name_hash
    }

    pub fn verify_version(&self, expected_version: &[u8; 32]) -> bool {
        &self.version_hash == expected_version
    }

    pub fn verify_name(&self, expected_name_hash: &[u8; 32]) -> bool {
        &self.name_hash == expected_name_hash
    }

    pub fn try_deserialize(data: &zenoh::bytes::ZBytes) -> Result<T, EnvelopeError> {
        let bytes = data.to_bytes();

        //两个32bit的hash，同时，ZBytes会附加两个长度前缀
        if bytes.len() < 66 {
            return Err(EnvelopeError::NotAnEnvelope(
                "Data too short: need at least 66 bytes for name_hash (32) + version_hash (32) and two length prefix (2)"
                    .to_string(),
            ));
        }

        if bytes[0] != 32 {
            return Err(EnvelopeError::NotAnEnvelope(
                "The name hash length prefix is not 32.".to_string(),
            ));
        }

        let mut offset = 1;

        let actual_name_hash: [u8; 32] = bytes[offset..offset + 32]
            .try_into()
            .map_err(|_| EnvelopeError::NotAnEnvelope("Failed to read name_hash".to_string()))?;
        offset += 32;

        let expected_name_hash = T::name_hash();
        if actual_name_hash != expected_name_hash {
            return Err(EnvelopeError::TypeMismatch {
                expected_name_hash,
                actual_name_hash,
            });
        }

        if bytes[offset] != 32 {
            return Err(EnvelopeError::NotAnEnvelope(
                "The version hash length prefix is not 32.".to_string(),
            ));
        }

        //跳过Version Hash 的长度前缀所在的位置
        offset += 1;

        let actual_version_hash: [u8; 32] = bytes[offset..offset + 32]
            .try_into()
            .map_err(|_| EnvelopeError::NotAnEnvelope("Failed to read version_hash".to_string()))?;
        offset += 32;

        let expected_version_hash = T::version_hash();
        if actual_version_hash != expected_version_hash {
            return Err(EnvelopeError::VersionMismatch {
                expected_version_hash,
                actual_version_hash,
            });
        }

        //跳过Payload 长度前缀
        // offset += 1;

        let payload_bytes = &bytes[offset..];
        let payload_zbytes = zenoh::bytes::ZBytes::from(payload_bytes);

        zenoh_ext::z_deserialize(&payload_zbytes)
            .map_err(|e| EnvelopeError::DeserializeError(e.to_string()))
    }
}

impl<T: MessageMeta + zenoh_ext::Serialize> zenoh_ext::Serialize for SmsgEnvelope<T> {
    fn serialize(&self, serializer: &mut zenoh_ext::ZSerializer) {
        self.name_hash.serialize(serializer);
        self.version_hash.serialize(serializer);
        self.payload.serialize(serializer);
    }
}

impl<T: MessageMeta + zenoh_ext::Deserialize> zenoh_ext::Deserialize for SmsgEnvelope<T> {
    fn deserialize(
        deserializer: &mut zenoh_ext::ZDeserializer,
    ) -> Result<Self, zenoh_ext::ZDeserializeError> {
        let name_hash: [u8; 32] = zenoh_ext::Deserialize::deserialize(deserializer)?;
        let version_hash: [u8; 32] = zenoh_ext::Deserialize::deserialize(deserializer)?;
        let payload: T = zenoh_ext::Deserialize::deserialize(deserializer)?;

        Ok(SmsgEnvelope {
            name_hash,
            version_hash,
            payload,
        })
    }
}
