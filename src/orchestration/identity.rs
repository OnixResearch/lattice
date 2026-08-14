//! Checked nominal identities for the orchestration decision core.
//!
//! Each type preserves its existing string wire representation. Constructors
//! reject malformed boundary values before they enter orchestration decisions.

const MAX_TEXT_ID_BYTES: usize = 256;
const HEX_CHARS_PER_BYTE: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionError {
    role: &'static str,
    issue: AdmissionIssue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AdmissionIssue {
    Empty,
    TooLong,
    Malformed,
    InvalidBlake3,
}

#[derive(Debug, Clone, Copy)]
struct AdmissionInput<'a> {
    role: &'static str,
    value: &'a str,
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
        let expected_length_bytes = blake3::OUT_LEN.checked_mul(HEX_CHARS_PER_BYTE)?;
        let is_lower_hex = input.value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
        (input.value.len() != expected_length_bytes || !is_lower_hex).then_some(Self {
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

// r[impl lattice.agent_orchestration.nominal_identities.types]
// r[impl lattice.agent_orchestration.nominal_identities.validation]
macro_rules! text_identity {
    ($(#[$meta:meta])* $name:ident, $role:literal) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn try_from_text(value: impl Into<String>) -> Result<Self, AdmissionError> {
                let value = value.into();
                let input = AdmissionInput {
                    role: $role,
                    value: &value,
                };
                if let Some(error) = AdmissionError::text(input) {
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

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(self.as_str())
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
                let input = AdmissionInput {
                    role: $role,
                    value: &value,
                };
                if let Some(error) = AdmissionError::blake3(input) {
                    return Err(error);
                }
                Ok(Self(value))
            }

            pub(super) fn from_generated(value: String) -> Self {
                let input = AdmissionInput {
                    role: $role,
                    value: &value,
                };
                debug_assert!(AdmissionError::blake3(input).is_none());
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

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(self.as_str())
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
    };
}

text_identity!(
    /// Stable orchestration-definition identity.
    DefinitionId,
    "orchestration definition identity"
);
text_identity!(
    /// Stable trigger binding identity.
    TriggerId,
    "orchestration trigger identity"
);
text_identity!(
    /// Caller-supplied trigger idempotency identity.
    TriggerIdempotencyKey,
    "orchestration trigger idempotency key"
);
text_identity!(
    /// Budget reservation identity. Dispatch identities require an explicit projection.
    ReservationId,
    "orchestration reservation identity"
);

// r[impl lattice.agent_orchestration.nominal_identities.events]
digest_identity!(DefinitionRevision, "orchestration definition revision");
digest_identity!(WorkflowRevision, "orchestration workflow revision");
digest_identity!(InstanceId, "orchestration instance identity");
digest_identity!(DispatchId, "orchestration dispatch identity");
digest_identity!(ProposalId, "orchestration proposal identity");
digest_identity!(ApprovalId, "orchestration approval identity");
digest_identity!(FactId, "orchestration fact identity");
digest_identity!(AuthorityPolicyIdentity, "orchestration authority-policy identity");
digest_identity!(EventId, "orchestration event identity");
digest_identity!(PreviousEventId, "orchestration previous-event identity");
digest_identity!(LedgerHeadId, "orchestration ledger-head identity");
digest_identity!(StateDigest, "orchestration state digest");
digest_identity!(TriggerScopedKey, "orchestration trigger scoped key");

impl ProposalId {
    pub(super) fn clear_for_hash(&mut self) {
        self.0.clear();
    }
}

impl ApprovalId {
    pub(super) fn clear_for_hash(&mut self) {
        self.0.clear();
    }
}

impl EventId {
    pub(super) fn clear_for_hash(&mut self) {
        self.0.clear();
    }
}

impl ReservationId {
    #[must_use]
    pub(super) fn from_dispatch(dispatch: &DispatchId) -> Self {
        Self(dispatch.as_str().to_owned())
    }
}

impl PreviousEventId {
    #[must_use]
    pub(super) fn from_ledger_head(head: &LedgerHeadId) -> Self {
        Self::from_generated(head.as_str().to_owned())
    }
}

impl LedgerHeadId {
    #[must_use]
    pub(super) fn from_event(event: &EventId) -> Self {
        Self::from_generated(event.as_str().to_owned())
    }
}
