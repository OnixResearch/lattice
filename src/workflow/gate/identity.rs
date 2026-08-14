//! Checked nominal identities for durable workflow approvals.

const MAX_TEXT_ID_BYTES: usize = 256;
const HEX_CHARS_PER_BYTE: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionError {
    role: &'static str,
    issue: AdmissionIssue,
}

struct AdmissionInput<'a> {
    role: &'static str,
    value: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AdmissionIssue {
    Empty,
    TooLong,
    Malformed,
    InvalidBlake3,
}

impl AdmissionError {
    fn text(input: AdmissionInput<'_>) -> Option<Self> {
        let issue = if input.value.trim().is_empty() {
            Some(AdmissionIssue::Empty)
        } else if input.value.len() > MAX_TEXT_ID_BYTES {
            Some(AdmissionIssue::TooLong)
        } else if input.value.trim() != input.value || input.value.chars().any(char::is_control) {
            Some(AdmissionIssue::Malformed)
        } else {
            None
        };
        issue.map(|issue| Self {
            role: input.role,
            issue,
        })
    }

    fn blake3(input: AdmissionInput<'_>) -> Option<Self> {
        let expected_hex_chars = blake3::OUT_LEN.checked_mul(HEX_CHARS_PER_BYTE)?;
        let is_lower_hex = input.value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
        (input.value.len() != expected_hex_chars || !is_lower_hex).then_some(Self {
            role: input.role,
            issue: AdmissionIssue::InvalidBlake3,
        })
    }
}

impl std::fmt::Display for AdmissionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.issue {
            AdmissionIssue::Empty => write!(formatter, "{} cannot be empty", self.role),
            AdmissionIssue::TooLong => write!(formatter, "{} exceeds {MAX_TEXT_ID_BYTES} bytes", self.role),
            AdmissionIssue::Malformed => {
                write!(formatter, "{} contains unsupported whitespace or control text", self.role)
            }
            AdmissionIssue::InvalidBlake3 => {
                write!(formatter, "{} must be one lowercase BLAKE3 hex digest", self.role)
            }
        }
    }
}

impl std::error::Error for AdmissionError {}

macro_rules! identity_common {
    ($name:ident) => {
        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
                serializer.serialize_str(self.as_str())
            }
        }
    };
}

macro_rules! text_identity {
    ($(#[$meta:meta])* $name:ident, $role:literal) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn try_from_text(value: impl Into<String>) -> Result<Self, AdmissionError> {
                let value = value.into();
                if let Some(error) = AdmissionError::text(AdmissionInput { role: $role, value: &value }) {
                    return Err(error);
                }
                Ok(Self(value))
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }

        impl TryFrom<String> for $name {
            type Error = AdmissionError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::try_from_text(value)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = AdmissionError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::try_from_text(value)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = <String as serde::Deserialize>::deserialize(deserializer)?;
                Self::try_from_text(value).map_err(serde::de::Error::custom)
            }
        }

        identity_common!($name);
    };
}

macro_rules! digest_identity {
    ($(#[$meta:meta])* $name:ident, $role:literal) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn try_from_hex(value: impl Into<String>) -> Result<Self, AdmissionError> {
                let value = value.into();
                if let Some(error) = AdmissionError::blake3(AdmissionInput { role: $role, value: &value }) {
                    return Err(error);
                }
                Ok(Self(value))
            }

            #[allow(
                dead_code,
                reason = "some digest roles only admit external reviewed identities"
            )]
            pub(crate) fn from_generated(value: String) -> Self {
                debug_assert!(AdmissionError::blake3(AdmissionInput { role: $role, value: &value }).is_none());
                Self(value)
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }

        impl TryFrom<String> for $name {
            type Error = AdmissionError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::try_from_hex(value)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = AdmissionError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::try_from_hex(value)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = <String as serde::Deserialize>::deserialize(deserializer)?;
                Self::try_from_hex(value).map_err(serde::de::Error::custom)
            }
        }

        identity_common!($name);
    };
}

text_identity!(
    /// Stable author-selected identity for one approval gate.
    GateId,
    "workflow approval gate identity"
);
text_identity!(
    /// Stable local or delegated actor identity admitted by policy.
    ActorId,
    "workflow approval actor identity"
);
text_identity!(
    /// Stable name for a declared resume value or credential binding.
    ResumeBindingName,
    "workflow approval resume binding name"
);
text_identity!(
    /// Shell-owned identity used to claim one resume mutation.
    ResumeOwnerId,
    "workflow approval resume owner identity"
);

digest_identity!(ApprovalRequestId, "workflow approval request identity");
digest_identity!(ApprovalDecisionId, "workflow approval decision identity");
digest_identity!(ApprovalAuthorityInputId, "workflow approval authority-input identity");
digest_identity!(ApprovalResumeId, "workflow approval resume identity");
digest_identity!(ApprovalEventId, "workflow approval event identity");
digest_identity!(ApprovalReceiptId, "workflow approval receipt identity");
digest_identity!(ApprovalArtifactId, "workflow approval artifact identity");
digest_identity!(ApprovalInputId, "workflow approval selected-input identity");
digest_identity!(ApprovalAuthorityPolicyId, "workflow approval authority-policy identity");
digest_identity!(ApprovalExpiryPolicyId, "workflow approval expiry-policy identity");
digest_identity!(ApprovalResumeSchemaId, "workflow approval resume-schema identity");
digest_identity!(ApprovalStateDigest, "workflow approval state digest");
