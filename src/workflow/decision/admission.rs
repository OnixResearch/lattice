//! Versioned `lattice.typed-policy-profile.v1` operation profiles and their admission.

/// Schema of an operation profile.
pub const SCHEMA: &str = "lattice.typed-policy-profile.v1";
/// The only implemented task shape: select one declared label.
pub const SELECT_SHAPE: &str = "select";
/// Largest profile encoding admitted, in bytes.
pub const MAX_PROFILE_BYTES: u64 = 65_536;
/// Most labels a profile may declare.
pub const MAX_LABEL_COUNT: u64 = 16;
/// Longest label, in bytes.
pub const MAX_LABEL_BYTES: u64 = 64;
/// Most projected features.
pub const MAX_FEATURE_COUNT: u64 = 64;
/// Largest projected input, in bytes.
pub const MAX_INPUT_BYTES: u64 = 65_536;
/// Largest retained receipt, in bytes; the producer's own receipt ceiling.
pub const MAX_RECEIPT_BYTES: u64 = 16_384;
/// Largest deterministic operation budget.
pub const MAX_OPERATION_COUNT: u64 = 1_000_000;
/// Longest wall-clock deadline, in milliseconds.
pub const MAX_DEADLINE_MILLIS: u64 = 60_000;
const IDENTITY_DOMAIN: &str = "lattice.typed-policy-profile-identity.v1";

/// The Condition node a profile selects for.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeSelector {
    /// The workflow name.
    pub workflow: String,
    /// The Condition node label.
    pub node_label: String,
}

/// One projected integer feature.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Feature {
    /// The feature name in the artifact's declared order.
    pub name: String,
    /// The JSON pointer into the Condition input.
    pub pointer: String,
    /// Smallest admitted raw value.
    pub min: i64,
    /// Largest admitted raw value.
    pub max: i64,
}

/// The input projection and its schema.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputSchema {
    /// The input schema identity.
    pub schema: String,
    /// The largest Condition input projected, in bytes.
    pub max_bytes: u64,
    /// Features, in the artifact's declared order.
    pub features: Vec<Feature>,
}

/// The result schema and retention.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultSchema {
    /// The result schema identity.
    pub schema: String,
    /// The largest receipt admitted, in bytes.
    pub max_receipt_bytes: u64,
    /// Whether the raw receipt is retained in the decision record.
    #[serde(rename = "retain_receipt")]
    pub is_receipt_retained: bool,
}

/// Invocation limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    /// The deterministic operation budget passed to the runtime.
    pub max_operation_count: u64,
    /// The host wall-clock deadline, in milliseconds.
    pub deadline_millis: u64,
}

/// The exact artifact and runtime cohort a profile supports.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cohort {
    /// The runtime crate.
    pub runtime_crate: String,
    /// The runtime crate version.
    pub runtime_version: String,
    /// The artifact API.
    pub artifact_api: u32,
    /// The artifact identity.
    pub artifact: String,
    /// The model (weights) identity.
    pub model: String,
    /// The model kind.
    pub policy_kind: String,
    /// The feature schema identity.
    pub feature_schema: String,
    /// The action schema identity.
    pub action_schema: String,
}

/// The declared bounded fallback for every non-accepted outcome.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Fallback {
    /// Route through one declared label.
    Label {
        /// The declared label.
        label: String,
    },
    /// Use the Condition node's own predicates when exactly one matches.
    DeclaredPredicates,
    /// Fail the node explicitly.
    Terminal,
}

/// A typed operation profile.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    /// The profile schema.
    pub schema: String,
    /// The profile name.
    pub id: String,
    /// The profile revision.
    pub version: u32,
    /// Whether the profile binds at all.
    #[serde(rename = "enabled")]
    pub is_enabled: bool,
    /// The task shape.
    pub shape: String,
    /// Whether the profile requests session or recurrent state.
    #[serde(rename = "stateful")]
    pub is_stateful: bool,
    /// The bound Condition node.
    pub binding: NodeSelector,
    /// The input projection.
    pub input: InputSchema,
    /// The bounded label set.
    pub labels: Vec<String>,
    /// The result schema.
    pub result: ResultSchema,
    /// The invocation limits.
    pub limits: Limits,
    /// The supported artifact and runtime cohort.
    pub cohort: Cohort,
    /// The declared fallback.
    pub fallback: Fallback,
}

/// An admitted profile and its identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Admitted {
    /// The admitted profile.
    pub profile: Profile,
    /// The canonical `b3:` profile identity.
    pub identity: String,
}

impl Admitted {
    /// Whether this enabled profile selects `node` of `workflow`.
    #[must_use]
    pub fn selects(&self, workflow: &crate::engine::workflow::Workflow, node: &crate::engine::workflow::Node) -> bool {
        let binding = &self.profile.binding;
        let is_named = binding.workflow == workflow.meta.name && binding.node_label == node.label;
        self.profile.is_enabled && is_named
    }
}

/// Why a profile was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// The encoding is empty or too large.
    Size,
    /// The bytes are not one profile document or hold unknown fields.
    Malformed,
    /// Another profile schema.
    Schema,
    /// An unsupported task shape.
    Shape,
    /// Session or recurrent state was requested.
    Stateful,
    /// The node selector is empty.
    Selector,
    /// A label is empty, unsafe, too long or repeated, or the set is empty or too large.
    Labels,
    /// The input projection is empty, too large or malformed.
    Input,
    /// The result schema or receipt limit is invalid.
    Result,
    /// An operation budget or deadline is out of range.
    Limits,
    /// The cohort names another runtime or an unsafe identity.
    Cohort,
    /// The fallback label is not declared.
    Fallback,
}

impl Refusal {
    /// The stable code of the refusal.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Size => "size",
            Self::Malformed => "malformed",
            Self::Schema => "schema",
            Self::Shape => "shape",
            Self::Stateful => "stateful",
            Self::Selector => "selector",
            Self::Labels => "labels",
            Self::Input => "input",
            Self::Result => "result",
            Self::Limits => "limits",
            Self::Cohort => "cohort",
            Self::Fallback => "fallback",
        }
    }
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "typed policy profile refused: {}", self.code())
    }
}

impl std::error::Error for Refusal {}

/// Admits one profile encoding.
///
/// # Errors
///
/// Returns the first [`Refusal`]; nothing binds.
// r[impl lattice.typed_policy.profile]
// r[impl lattice.typed_policy.stateless]
pub fn admit(bytes: &[u8]) -> Result<Admitted, Refusal> {
    let size_bytes = u64::try_from(bytes.len()).map_err(|_| Refusal::Size)?;
    if size_bytes == 0 || size_bytes > MAX_PROFILE_BYTES {
        return Err(Refusal::Size);
    }
    let profile = serde_json::from_slice::<Profile>(bytes).map_err(|_| Refusal::Malformed)?;
    super::checks::check(&profile)?;
    let canonical = serde_json::to_string(&profile).map_err(|_| Refusal::Malformed)?;
    let identity = super::identity(IDENTITY_DOMAIN, &[canonical.as_str()]);
    debug_assert!(identity.starts_with(super::IDENTITY_PREFIX));
    debug_assert_eq!(profile.schema, SCHEMA);
    Ok(Admitted { profile, identity })
}
