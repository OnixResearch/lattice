//! Content-addressed typed values that flow between nodes.
//!
//! Every value produced by a node gets blake3-hashed on creation.
//! redb stores values keyed by hash for dedup. Re-runs skip nodes
//! whose input hash tree hasn't changed.

use std::fmt;

use blake3::Hash;
use serde::Deserialize;
use serde::Serialize;

/// Blake3 content hash of a value.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ContentHash(pub [u8; 32]);

impl ContentHash {
    pub fn from_bytes(data: &[u8]) -> Self {
        Self(*blake3::hash(data).as_bytes())
    }

    pub fn as_blake3(&self) -> Hash {
        Hash::from_bytes(self.0)
    }

    pub fn to_hex(&self) -> String {
        self.as_blake3().to_hex().to_string()
    }

    /// Combine multiple hashes into one (for input-tree hashing).
    pub fn combine(hashes: &[ContentHash]) -> Self {
        let mut hasher = blake3::Hasher::new();
        for h in hashes {
            hasher.update(&h.0);
        }
        Self(*hasher.finalize().as_bytes())
    }
}

impl fmt::Debug for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}…", &self.to_hex()[..12])
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl Serialize for ContentHash {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for ContentHash {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let hash = Hash::from_hex(&s).map_err(serde::de::Error::custom)?;
        Ok(Self(*hash.as_bytes()))
    }
}

/// Type tag for port compatibility checking.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeTag {
    String,
    Bytes,
    Json,
    ExitCode,
    Any,
}

impl TypeTag {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Bytes => "bytes",
            Self::Json => "json",
            Self::ExitCode => "exitcode",
            Self::Any => "any",
        }
    }

    /// Check if `self` (output) is compatible with `target` (input).
    pub fn compatible_with(&self, target: &TypeTag) -> bool {
        match target {
            TypeTag::Any => true,
            _ => self == target,
        }
    }
}

impl fmt::Display for TypeTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A typed, content-addressed value flowing between nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Value {
    pub data: ValueData,
    pub hash: ContentHash,
    #[serde(default)]
    pub sensitive_fragments: Vec<String>,
}

/// The actual data payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValueData {
    String(String),
    Bytes(Vec<u8>),
    Json(serde_json::Value),
    ExitCode(i32),
    Error { message: String, stderr: String },
}

impl Value {
    pub fn from_string(s: String) -> Self {
        let hash = ContentHash::from_bytes(s.as_bytes());
        Self {
            data: ValueData::String(s),
            hash,
            sensitive_fragments: Vec::new(),
        }
    }

    pub fn from_bytes(b: Vec<u8>) -> Self {
        let hash = ContentHash::from_bytes(&b);
        Self {
            data: ValueData::Bytes(b),
            hash,
            sensitive_fragments: Vec::new(),
        }
    }

    pub fn from_json(v: serde_json::Value) -> Self {
        let serialized = serde_json::to_vec(&v).unwrap_or_default();
        let hash = ContentHash::from_bytes(&serialized);
        Self {
            data: ValueData::Json(v),
            hash,
            sensitive_fragments: Vec::new(),
        }
    }

    pub fn from_exit_code(code: i32) -> Self {
        let hash = ContentHash::from_bytes(&code.to_le_bytes());
        Self {
            data: ValueData::ExitCode(code),
            hash,
            sensitive_fragments: Vec::new(),
        }
    }

    pub fn error(message: String, stderr: String) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"error:");
        hasher.update(message.as_bytes());
        hasher.update(stderr.as_bytes());
        let hash = ContentHash(*hasher.finalize().as_bytes());
        Self {
            data: ValueData::Error { message, stderr },
            hash,
            sensitive_fragments: Vec::new(),
        }
    }

    pub fn from_sensitive_string(s: String) -> Self {
        let mut value = Self::from_string(s.clone());
        value.sensitive_fragments.push(s);
        value
    }

    pub fn with_sensitive_fragments(mut self, fragments: Vec<String>) -> Self {
        self.sensitive_fragments = fragments;
        self
    }

    pub fn type_tag(&self) -> TypeTag {
        match &self.data {
            ValueData::String(_) => TypeTag::String,
            ValueData::Bytes(_) => TypeTag::Bytes,
            ValueData::Json(_) => TypeTag::Json,
            ValueData::ExitCode(_) => TypeTag::ExitCode,
            ValueData::Error { .. } => TypeTag::String,
        }
    }

    /// Get string representation regardless of inner type.
    pub fn as_display_string(&self) -> String {
        match &self.data {
            ValueData::String(s) => s.clone(),
            ValueData::Bytes(b) => String::from_utf8_lossy(b).into_owned(),
            ValueData::Json(v) => serde_json::to_string_pretty(v).unwrap_or_default(),
            ValueData::ExitCode(c) => format!("exit code: {c}"),
            ValueData::Error { message, stderr } => {
                format!("ERROR: {message}\n{stderr}")
            }
        }
    }

    pub fn is_error(&self) -> bool {
        matches!(self.data, ValueData::Error { .. })
    }
}
