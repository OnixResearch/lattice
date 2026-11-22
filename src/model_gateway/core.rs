//! Wire records of the host model gateway: the Kiln effect authority frames
//! (`lattice.vm-effect-authority.v1`, C3), the forwarded broker frames
//! (`lattice.model-broker-forwarded-upstream.v1`, C4) and the upstream
//! configuration. Decisions over them live in `admission`. No I/O.

// r[impl lattice.vm_profile.effect_authority]
// r[impl lattice.vm_profile.forwarded_upstream]

pub const AUTHORITY_SCHEMA: &str = "lattice.vm-effect-authority.v1";
pub const FORWARDED_SCHEMA: &str = crate::candidate_worker::broker::FORWARDED_SCHEMA;
pub const UPSTREAM_SCHEMA: &str = "lattice.model-gateway-upstream.v1";
/// The served route name the gateway admits through the authority.
pub const ROUTE: &str = "primary";
/// C3 frames carry identities and small counters only.
pub const MAX_AUTHORITY_FRAME_BYTES: u32 = 65_536;
/// JSON escaping can grow a body sixfold; framing adds a bounded envelope.
const FRAME_ESCAPE_FACTOR: u32 = 6;
const FRAME_ENVELOPE_BYTES: u32 = 1_024;

/// The largest C4 frame carrying a body of at most `body_bytes`.
pub const fn maximum_forwarded_frame_bytes(body_bytes: u32) -> u32 {
    body_bytes.saturating_mul(FRAME_ESCAPE_FACTOR).saturating_add(FRAME_ENVELOPE_BYTES)
}

/// An absent optional field, omitted from serialization.
const fn absent<T>() -> Option<T> {
    None
}

/// C3 refusal codes. They are also the C4 refusal codes, with `malformed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityRefusal {
    NotCurrent,
    Deadline,
    Budget,
    Revoked,
    UnknownBinding,
}

/// How an admitted upstream call ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettleOutcome {
    Completed,
    Failed,
    TimedOut,
    Cancelled,
    Uncertain,
}

/// Provider-reported usage; absent counters stay absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Usage {
    #[serde(default = "absent", skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,
    #[serde(default = "absent", skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u64>,
}

/// One C3 request frame. Exactly one request per connection.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AuthorityRequest {
    Admit {
        binding: String,
        request_index: u32,
        model: String,
        route: String,
    },
    Settle {
        binding: String,
        request_index: u32,
        outcome: SettleOutcome,
        #[serde(default = "absent", skip_serializing_if = "Option::is_none")]
        usage: Option<Usage>,
    },
}

/// One C3 reply frame. `Admitted` answers only `Admit`, `Recorded` only `Settle`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AuthorityReply {
    Admitted {
        placement_host: String,
        placement_generation: u64,
        /// Requests of this binding consumed so far, including this one.
        consumed_requests: u32,
        /// Requests of this binding still admissible after this one.
        remaining_requests: u32,
        deadline_unix_ms: u64,
    },
    Recorded {
        consumed_requests: u32,
    },
    Refused {
        code: AuthorityRefusal,
    },
}

/// C4 refusal codes: every C3 code, plus `malformed` for a frame the gateway
/// refuses itself before any authority or upstream call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForwardedRefusal {
    NotCurrent,
    Deadline,
    Budget,
    Revoked,
    UnknownBinding,
    Malformed,
}

impl From<AuthorityRefusal> for ForwardedRefusal {
    fn from(value: AuthorityRefusal) -> Self {
        match value {
            AuthorityRefusal::NotCurrent => Self::NotCurrent,
            AuthorityRefusal::Deadline => Self::Deadline,
            AuthorityRefusal::Budget => Self::Budget,
            AuthorityRefusal::Revoked => Self::Revoked,
            AuthorityRefusal::UnknownBinding => Self::UnknownBinding,
        }
    }
}

/// One C4 request frame from the guest broker.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForwardedRequest {
    pub binding: String,
    pub request_index: u32,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Responded {
    pub request_index: u32,
    pub status: u16,
    pub body: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Refused {
    pub request_index: u32,
    pub refused: ForwardedRefusal,
}

/// `uncertain` is always `true` on the wire; `false` is malformed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Uncertain {
    pub request_index: u32,
    pub uncertain: bool,
}

/// One C4 reply frame. `Refused` means nothing reached the upstream;
/// `Uncertain` means the gateway delivers no response and the upstream may
/// have executed.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ForwardedResponse {
    Responded(Responded),
    Refused(Refused),
    Uncertain(Uncertain),
}

impl ForwardedResponse {
    #[must_use]
    pub const fn request_index(&self) -> u32 {
        match self {
            Self::Responded(value) => value.request_index,
            Self::Refused(value) => value.request_index,
            Self::Uncertain(value) => value.request_index,
        }
    }

    /// A reply for `request_index` that is well formed.
    #[must_use]
    pub const fn answers(&self, request_index: u32) -> bool {
        let is_valid = match self {
            Self::Uncertain(value) => value.uncertain,
            Self::Responded(_) | Self::Refused(_) => true,
        };
        is_valid && self.request_index() == request_index
    }
}

/// The host-only upstream of one gateway: endpoint, exact model, host
/// credential reference and the local request ceiling.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Upstream {
    pub schema: String,
    pub endpoint: String,
    pub model: String,
    #[serde(default = "absent", skip_serializing_if = "Option::is_none")]
    pub credential: Option<crate::credentials::SecretRef>,
    pub max_requests: u32,
}
