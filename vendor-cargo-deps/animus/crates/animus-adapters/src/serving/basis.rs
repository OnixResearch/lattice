//! Opaque tokens, canonical references, separate identities, and framing.

use alloc::string::ToString;

const REF_PREFIX: &str = "blake3:";
const REF_HEX_CHARS: usize = 64;
const REF_CHARS: usize = 71;
const FORBIDDEN_TOKEN_BYTES: &[u8] = b"/\\\"'`<>{}";
const TEXT_TAG: u8 = 0;
const NUMBER_TAG: u8 = 1;

/// Typed contract failure for snapshots, profiles, candidates, and replies.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractIssue {
    /// A token was empty, too long, or contained whitespace, control, path, or quote bytes.
    UnsafeToken {
        /// Field name.
        field: &'static str,
    },
    /// A reference was not `blake3:` followed by 64 lowercase hex characters.
    NoncanonicalRef {
        /// Field name.
        field: &'static str,
    },
    /// A count, generation, sequence, or tick that must be positive was zero.
    ZeroValue {
        /// Field name.
        field: &'static str,
    },
    /// A list was empty when at least one item is required.
    Empty {
        /// Field name.
        field: &'static str,
    },
    /// A list exceeded its named bound.
    TooMany {
        /// Field name.
        field: &'static str,
    },
    /// A list repeated an identity that must be unique.
    Duplicate {
        /// Field name.
        field: &'static str,
    },
    /// An observation or validity window ended at or before it started.
    InvertedWindow {
        /// Field name.
        field: &'static str,
    },
    /// A capacity unit does not measure the declared capacity kind.
    UnitMismatch,
    /// A request envelope had zero or inverted token bounds.
    InvalidEnvelope,
    /// Receipt facts contradict each other, for example usage without dispatch.
    InconsistentReceipt,
}

/// A bounded opaque identifier. It never carries prose, paths, or quotes.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Token(alloc::string::String);

impl Token {
    /// Validates one opaque token of 1 to [`super::MAX_TOKEN_BYTES`] graphic
    /// ASCII bytes without `/`, `\`, quotes, backticks, angle brackets, or braces.
    ///
    /// # Errors
    ///
    /// Returns [`ContractIssue::UnsafeToken`] for empty, oversized, whitespace,
    /// control, path, or quote material.
    pub fn new(field: &'static str, value: &str) -> Result<Self, ContractIssue> {
        let is_within_bound = u32::try_from(value.len())
            .is_ok_and(|bytes| bytes > 0 && bytes <= super::MAX_TOKEN_BYTES);
        let has_safe_bytes = value.bytes().all(is_safe_byte);
        if is_within_bound && has_safe_bytes {
            debug_assert!(!value.is_empty());
            debug_assert!(value.bytes().all(|byte| byte.is_ascii_graphic()));
            return Ok(Self(value.to_string()));
        }
        Err(ContractIssue::UnsafeToken { field })
    }

    /// Returns the token text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

fn is_safe_byte(byte: u8) -> bool {
    byte.is_ascii_graphic() && !FORBIDDEN_TOKEN_BYTES.contains(&byte)
}

/// A canonical `blake3:<64 lowercase hex>` reference compared exactly.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ContentRef(alloc::string::String);

impl ContentRef {
    /// Validates one canonical reference.
    ///
    /// # Errors
    ///
    /// Returns [`ContractIssue::NoncanonicalRef`] for any other spelling.
    pub fn new(field: &'static str, value: &str) -> Result<Self, ContractIssue> {
        let Some(hex) = value.strip_prefix(REF_PREFIX) else {
            return Err(ContractIssue::NoncanonicalRef { field });
        };
        let is_lower_hex = hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
        if hex.len() == REF_HEX_CHARS && is_lower_hex {
            debug_assert_eq!(value.len(), REF_CHARS);
            debug_assert!(value.starts_with(REF_PREFIX));
            return Ok(Self(value.to_string()));
        }
        Err(ContractIssue::NoncanonicalRef { field })
    }

    /// Returns the reference text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub(super) fn from_hash(hash: &blake3::Hash) -> Self {
        let mut text = alloc::string::String::with_capacity(REF_CHARS);
        text.push_str(REF_PREFIX);
        text.push_str(hash.to_hex().as_str());
        debug_assert_eq!(text.len(), REF_CHARS);
        Self(text)
    }
}

/// Who established a serving fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Attestation {
    /// The snapshot source asserted the fact.
    Claimed,
    /// The host observed the fact independently of the source's claim.
    Observed,
}

impl Attestation {
    /// Stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Claimed => "claimed",
            Self::Observed => "observed",
        }
    }
}

/// Exact semantic model: the model identifier and its configuration identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelBinding {
    model_id: Token,
    configuration_ref: ContentRef,
}

impl ModelBinding {
    /// Validates a model binding.
    ///
    /// # Errors
    ///
    /// Returns [`ContractIssue`] for an unsafe model id or noncanonical configuration.
    pub fn new(model_id: &str, configuration_ref: &str) -> Result<Self, ContractIssue> {
        Ok(Self {
            model_id: Token::new("model_id", model_id)?,
            configuration_ref: ContentRef::new("configuration_ref", configuration_ref)?,
        })
    }

    /// Exact model identifier.
    #[must_use]
    pub fn model_id(&self) -> &str {
        self.model_id.as_str()
    }

    /// Exact configuration identity.
    #[must_use]
    pub fn configuration_ref(&self) -> &str {
        self.configuration_ref.as_str()
    }

    pub(super) fn matches(&self, model: (&str, &str)) -> bool {
        self.model_id.as_str() == model.0 && self.configuration_ref.as_str() == model.1
    }
}

/// Logical service, endpoint, deployment, and deployment generation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeploymentBinding {
    pub(super) service_id: Token,
    pub(super) endpoint_identity: Token,
    pub(super) deployment_id: Token,
    pub(super) deployment_generation: u64,
}

impl DeploymentBinding {
    /// Logical service identifier.
    #[must_use]
    pub fn service_id(&self) -> &str {
        self.service_id.as_str()
    }

    /// Safe endpoint identity; never a URL.
    #[must_use]
    pub fn endpoint_identity(&self) -> &str {
        self.endpoint_identity.as_str()
    }

    /// Serving deployment identifier.
    #[must_use]
    pub fn deployment_id(&self) -> &str {
        self.deployment_id.as_str()
    }

    /// Deployment generation; a replacement increments it.
    #[must_use]
    pub const fn deployment_generation(&self) -> u64 {
        self.deployment_generation
    }
}

/// Replica placement as exposed by the provider. Opaque placement stays unknown.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplicaFact {
    /// The provider exposed no replica identity.
    Unknown,
    /// The provider exposed this opaque replica identity.
    Observed(Token),
}

impl ReplicaFact {
    /// Validates an optional replica identity.
    ///
    /// # Errors
    ///
    /// Returns [`ContractIssue::UnsafeToken`] for an unsafe identity.
    pub fn new(replica: Option<&str>) -> Result<Self, ContractIssue> {
        match replica {
            None => Ok(Self::Unknown),
            Some(value) => Ok(Self::Observed(Token::new("replica", value)?)),
        }
    }

    /// The replica identity, or `unknown`.
    #[must_use]
    pub fn label(&self) -> &str {
        match self {
            Self::Unknown => "unknown",
            Self::Observed(token) => token.as_str(),
        }
    }
}

/// Agent, session, and attempt identities, kept separate from serving identities.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallerBinding {
    agent: Token,
    session: Token,
    attempt: Token,
}

/// Borrowed caller identity fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CallerDraft<'a> {
    /// Stable agent identity.
    pub agent: &'a str,
    /// Session identity.
    pub session: &'a str,
    /// Attempt identity.
    pub attempt: &'a str,
}

impl CallerBinding {
    /// Validates caller identities.
    ///
    /// # Errors
    ///
    /// Returns [`ContractIssue::UnsafeToken`] for any unsafe identity.
    pub fn new(draft: CallerDraft<'_>) -> Result<Self, ContractIssue> {
        Ok(Self {
            agent: Token::new("agent_id", draft.agent)?,
            session: Token::new("session_id", draft.session)?,
            attempt: Token::new("attempt_id", draft.attempt)?,
        })
    }

    /// Agent identity.
    #[must_use]
    pub fn agent_id(&self) -> &str {
        self.agent.as_str()
    }

    /// Session identity.
    #[must_use]
    pub fn session_id(&self) -> &str {
        self.session.as_str()
    }

    /// Attempt identity.
    #[must_use]
    pub fn attempt_id(&self) -> &str {
        self.attempt.as_str()
    }
}

/// A caller's named model route and its optional caller-owned route identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteBinding {
    pub(super) route_name: Token,
    pub(super) route_ref: Option<ContentRef>,
}

impl RouteBinding {
    /// Named route, for example `primary`.
    #[must_use]
    pub fn route_name(&self) -> &str {
        self.route_name.as_str()
    }

    /// Caller-owned route identity, for example Lattice `lattice-model-route-v1`.
    #[must_use]
    pub fn route_ref(&self) -> Option<&str> {
        self.route_ref.as_ref().map(ContentRef::as_str)
    }
}

/// Domain-separated, tagged BLAKE3 builder for stack-owned identities.
///
/// Text values are graphic ASCII (tokens, references, and static labels), so a
/// leading tag byte below `0x21` frames each value unambiguously; numbers are
/// tagged and fixed-width little-endian.
pub(super) struct Framer(blake3::Hasher);

impl Framer {
    pub(super) fn new(domain: &str) -> Self {
        let mut framer = Self(blake3::Hasher::new());
        framer.text(domain);
        framer
    }

    pub(super) fn text(&mut self, value: &str) {
        debug_assert!(value.bytes().all(|byte| byte.is_ascii_graphic()));
        self.0.update(&[TEXT_TAG]);
        self.0.update(value.as_bytes());
    }

    pub(super) fn number(&mut self, value: u64) {
        self.0.update(&[NUMBER_TAG]);
        self.0.update(&value.to_le_bytes());
    }

    pub(super) fn optional(&mut self, value: Option<&str>) {
        match value {
            None => self.text("none"),
            Some(text) => {
                self.text("some");
                self.text(text);
            }
        }
    }

    pub(super) fn finish(&self) -> ContentRef {
        ContentRef::from_hash(&self.0.finalize())
    }
}

/// Validates a bounded list length and returns it as `u32`.
pub(super) fn bounded_len(
    field: &'static str,
    item_count: usize,
    maximum: u32,
) -> Result<u32, ContractIssue> {
    match u32::try_from(item_count) {
        Ok(count) if count <= maximum => Ok(count),
        Ok(_) | Err(_) => Err(ContractIssue::TooMany { field }),
    }
}

/// Rejects a zero value.
pub(super) fn positive(field: &'static str, value: u64) -> Result<u64, ContractIssue> {
    if value == 0 {
        return Err(ContractIssue::ZeroValue { field });
    }
    Ok(value)
}

/// Rejects a `(start, end)` window that does not end strictly after it starts.
pub(super) fn window(field: &'static str, bounds: (u64, u64)) -> Result<(), ContractIssue> {
    if bounds.1 <= bounds.0 {
        return Err(ContractIssue::InvertedWindow { field });
    }
    Ok(())
}
