type SimulationIdentityKind = super::nomenclature::SimulationIdentityKind;
type SimulationIdentityRegistry = super::nomenclature::SimulationIdentityRegistry;
type SimulationStateProjectionId = super::nomenclature::SimulationStateProjectionId;

const STATE_PROJECTION_VERSION: u16 = 1;
const MAX_STATE_PAIRS: usize = 32;
const MAX_STATE_KEY_BYTES: usize = 48;
const MAX_STATE_VALUE_BYTES: usize = 96;

/// One allowlisted canonical semantic state pair.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimulationStatePair {
    /// Stable allowlisted state key.
    pub key: String,
    /// Bounded semantic class value.
    pub value: String,
}

impl SimulationStatePair {
    /// Creates one state pair.
    #[must_use]
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

/// Policy used when a projection candidate exceeds its pair bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationProjectionOverflowPolicy {
    /// Reject the projection without producing evidence.
    Reject,
    /// Keep the first deterministic pairs and mark evidence truncated.
    Truncate,
}

/// Stable canonical-state validation code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationStateDiagnosticCode {
    /// Projection version is unsupported.
    UnknownProjectionVersion,
    /// Projection identity is unregistered.
    UnknownProjectionIdentity,
    /// Projection exceeds its reviewed pair bound.
    ProjectionTooLarge,
    /// Projection pair order is not strict and deterministic.
    ProjectionNotOrdered,
    /// State key is not allowlisted.
    StateKeyForbidden,
    /// State value is malformed or oversized.
    StateValueInvalid,
    /// State value appears sensitive or renderer-specific.
    SensitiveOrRendererState,
    /// Truncation flag does not match actual overflow handling.
    InvalidTruncationMarker,
}

/// Stable canonical-state diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationStateDiagnostic {
    /// Machine-comparable issue code.
    pub code: SimulationStateDiagnosticCode,
    /// Safe field class.
    pub field: &'static str,
}

/// Bounded versioned canonical semantic state projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationStateProjection {
    /// Projection schema version.
    pub version: u16,
    /// Registered projection identity.
    pub projection_id: SimulationStateProjectionId,
    /// Strictly ordered allowlisted state pairs.
    pub pairs: Vec<SimulationStatePair>,
    /// Whether deterministic truncation removed candidate pairs.
    pub is_truncated: bool,
}

impl SimulationStateProjection {
    /// Builds a version-1 projection and applies explicit overflow policy.
    ///
    /// # Errors
    ///
    /// Reject policy returns `ProjectionTooLarge`; other validation is performed
    /// by [`validate_state_projection`].
    pub fn build(
        projection_id: impl Into<SimulationStateProjectionId>,
        mut pairs: Vec<SimulationStatePair>,
        overflow: SimulationProjectionOverflowPolicy,
    ) -> Result<Self, SimulationStateDiagnostic> {
        let is_truncated = if pairs.len() > MAX_STATE_PAIRS {
            match overflow {
                SimulationProjectionOverflowPolicy::Reject => {
                    return Err(state_issue(SimulationStateDiagnosticCode::ProjectionTooLarge, "pairs"));
                }
                SimulationProjectionOverflowPolicy::Truncate => {
                    pairs.truncate(MAX_STATE_PAIRS);
                    true
                }
            }
        } else {
            false
        };
        Ok(Self {
            version: STATE_PROJECTION_VERSION,
            projection_id: projection_id.into(),
            pairs,
            is_truncated,
        })
    }

    /// Encodes canonical state without Rust debug formatting.
    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut output =
            format!("projection={}\nversion={}\ntruncated={}\n", self.projection_id, self.version, self.is_truncated);
        for pair in &self.pairs {
            output.push_str(&pair.key);
            output.push('=');
            output.push_str(&pair.value);
            output.push('\n');
        }
        output.into_bytes()
    }

    /// Returns the lowercase BLAKE3 identity of canonical projection bytes.
    #[must_use]
    pub fn blake3_identity(&self) -> String {
        blake3::hash(&self.canonical_bytes()).to_hex().to_string()
    }

    /// Returns borrowed pairs suitable for the public `ChaosControl` state API.
    #[must_use]
    pub fn coverage_pairs(&self) -> Vec<(&str, &str)> {
        self.pairs.iter().map(|pair| (pair.key.as_str(), pair.value.as_str())).collect()
    }
}

/// Validates a canonical projection against identity, ordering, and redaction policy.
#[must_use]
pub fn validate_state_projection(
    registry: &SimulationIdentityRegistry,
    projection: &SimulationStateProjection,
) -> Vec<SimulationStateDiagnostic> {
    let mut issues = Vec::new();
    debug_assert!(issues.is_empty());
    debug_assert!(projection.pairs.len() <= projection.pairs.capacity());
    if projection.version != STATE_PROJECTION_VERSION {
        issues.push(state_issue(SimulationStateDiagnosticCode::UnknownProjectionVersion, "version"));
    }
    if !registry.contains(SimulationIdentityKind::StateProjection, projection.projection_id.as_str()) {
        issues.push(state_issue(SimulationStateDiagnosticCode::UnknownProjectionIdentity, "projection_id"));
    }
    if projection.pairs.len() > MAX_STATE_PAIRS {
        issues.push(state_issue(SimulationStateDiagnosticCode::ProjectionTooLarge, "pairs"));
    }
    if projection.is_truncated && projection.pairs.len() != MAX_STATE_PAIRS {
        issues.push(state_issue(SimulationStateDiagnosticCode::InvalidTruncationMarker, "truncated"));
    }
    issues.extend(validate_state_pairs(projection));
    issues
}

fn validate_state_pairs(projection: &SimulationStateProjection) -> Vec<SimulationStateDiagnostic> {
    let mut issues = Vec::new();
    debug_assert!(issues.is_empty());
    debug_assert!(projection.pairs.len() <= projection.pairs.capacity());
    let mut previous: Option<&str> = None;
    for pair in projection.pairs.iter().take(MAX_STATE_PAIRS.saturating_add(1)) {
        if issues.len() < MAX_STATE_PAIRS.saturating_mul(4) && previous.is_some_and(|key| key >= pair.key.as_str()) {
            issues.push(state_issue(SimulationStateDiagnosticCode::ProjectionNotOrdered, "pairs"));
        }
        previous = Some(&pair.key);
        for issue in validate_state_pair(pair).into_iter().flatten() {
            if issues.len() < MAX_STATE_PAIRS.saturating_mul(4) {
                issues.push(issue);
            }
        }
    }
    issues
}

fn validate_state_pair(pair: &SimulationStatePair) -> [Option<SimulationStateDiagnostic>; 3] {
    debug_assert!(pair.key.len() <= pair.key.capacity());
    debug_assert!(pair.value.len() <= pair.value.capacity());
    let key_issue = (pair.key.len() > MAX_STATE_KEY_BYTES || !allowlisted_state_key(&pair.key))
        .then(|| state_issue(SimulationStateDiagnosticCode::StateKeyForbidden, "pair.key"));
    let value_issue = (pair.value.len() > MAX_STATE_VALUE_BYTES || !valid_state_value(&pair.value))
        .then(|| state_issue(SimulationStateDiagnosticCode::StateValueInvalid, "pair.value"));
    let sensitive_issue = sensitive_or_renderer_value(&pair.value)
        .then(|| state_issue(SimulationStateDiagnosticCode::SensitiveOrRendererState, "pair.value"));
    [key_issue, value_issue, sensitive_issue]
}

fn allowlisted_state_key(key: &str) -> bool {
    matches!(
        key,
        "choice.class"
            | "diagnostic.class"
            | "effect.class"
            | "effect.status"
            | "lifecycle.class"
            | "logical.tick"
            | "mode.class"
            | "selection.class"
            | "shutdown.class"
            | "viewport.columns"
            | "viewport.rows"
    )
}

fn valid_state_value(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_STATE_VALUE_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b':' | b'+' | b'='))
}

fn sensitive_or_renderer_value(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    value.starts_with('/')
        || value.contains("../")
        || value.contains('\\')
        || lower.contains("password")
        || lower.contains("credential")
        || lower.contains("secret")
        || lower.contains("token=")
        || lower.contains("clipboard")
        || lower.contains("terminal")
        || lower.contains("process-output")
        || lower.contains("dom-node")
        || lower.contains("ratatui-buffer")
        || lower.contains("pointer-address")
        || lower.contains("renderer-handle")
}

const fn state_issue(code: SimulationStateDiagnosticCode, field: &'static str) -> SimulationStateDiagnostic {
    SimulationStateDiagnostic { code, field }
}
