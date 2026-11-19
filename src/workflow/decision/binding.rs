//! Binding an admitted profile to one declared Condition node, projection, and invocation identity.

const INPUT_DOMAIN: &str = "lattice.typed-policy-input.v1";
const CANDIDATE_SET_DOMAIN: &str = "lattice.typed-policy-candidate-set.v1";
const INVOCATION_DOMAIN: &str = "lattice.typed-policy-invocation.v1";

/// Why a profile cannot bind to the node it names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The named node is not a Condition node.
    NotCondition,
    /// A profile label is not a declared branch of the Condition node.
    UndeclaredLabel(String),
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotCondition => write!(formatter, "typed policy binding refused: node is not a Condition node"),
            Self::UndeclaredLabel(label) => {
                write!(formatter, "typed policy binding refused: label `{label}` is not a declared branch")
            }
        }
    }
}

impl std::error::Error for Refusal {}

/// Returns the Condition configuration of `config` when it declares every profile label as a
/// branch.
///
/// # Errors
///
/// Returns [`Refusal`] when the node cannot route the profile's labels.
// r[impl lattice.typed_policy.route]
pub fn check_node<'a>(
    profile: &super::admission::Profile,
    config: &'a crate::engine::workflow::NodeConfig,
) -> Result<&'a crate::control_flow::ConditionNodeConfig, Refusal> {
    let crate::engine::workflow::NodeConfig::Condition(condition) = config else {
        return Err(Refusal::NotCondition);
    };
    let declared = condition
        .branches
        .iter()
        .map(|branch| branch.label.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    match profile.labels.iter().find(|label| !declared.contains(label.as_str())) {
        Some(label) => Err(Refusal::UndeclaredLabel(label.clone())),
        None => Ok(condition),
    }
}

/// Why the Condition input could not be projected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionFault {
    /// No input value reached the node.
    Missing,
    /// The input exceeds the declared byte limit.
    Oversized,
    /// The input is not one JSON document.
    Malformed,
    /// A declared pointer is absent or not an integer.
    MissingFeature,
    /// A feature lies outside its declared range.
    OutOfRange,
}

impl ProjectionFault {
    /// The stable code of the fault.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Missing => "projection-missing-input",
            Self::Oversized => "projection-oversized-input",
            Self::Malformed => "projection-malformed-input",
            Self::MissingFeature => "projection-missing-feature",
            Self::OutOfRange => "projection-out-of-range",
        }
    }
}

/// Projects the Condition input into raw features in declared order.
///
/// # Errors
///
/// Returns the first [`ProjectionFault`]; values are refused, never clamped.
pub fn project(
    input: &super::admission::InputSchema,
    value: Option<&crate::engine::value::Value>,
) -> Result<Vec<i64>, ProjectionFault> {
    let document = input_document(input.max_bytes, value)?;
    let features = input
        .features
        .iter()
        .map(|feature| project_feature(feature, &document))
        .collect::<Result<Vec<_>, _>>()?;
    debug_assert_eq!(features.len(), input.features.len());
    Ok(features)
}

fn input_document(
    max_bytes: u64,
    value: Option<&crate::engine::value::Value>,
) -> Result<serde_json::Value, ProjectionFault> {
    let value = value.ok_or(ProjectionFault::Missing)?;
    let encoded = match &value.data {
        crate::engine::ValueData::String(text) => text.clone(),
        crate::engine::ValueData::Json(json) => json.to_string(),
        crate::engine::ValueData::Bytes(_)
        | crate::engine::ValueData::Image(_)
        | crate::engine::ValueData::ExitCode(_)
        | crate::engine::ValueData::Error { .. } => return Err(ProjectionFault::Malformed),
    };
    let size_bytes = u64::try_from(encoded.len()).map_err(|_| ProjectionFault::Oversized)?;
    if size_bytes > max_bytes {
        return Err(ProjectionFault::Oversized);
    }
    serde_json::from_str(encoded.trim()).map_err(|_| ProjectionFault::Malformed)
}

fn project_feature(feature: &super::admission::Feature, document: &serde_json::Value) -> Result<i64, ProjectionFault> {
    let raw = document.pointer(&feature.pointer).and_then(serde_json::Value::as_i64);
    let value = raw.ok_or(ProjectionFault::MissingFeature)?;
    if (feature.min..=feature.max).contains(&value) {
        return Ok(value);
    }
    Err(ProjectionFault::OutOfRange)
}

/// The identities of a projected input under the profile's feature schema.
#[must_use]
pub fn input_identity(profile: &super::admission::Profile, features: &[i64]) -> String {
    let rendered = features.iter().map(i64::to_string).collect::<Vec<_>>();
    let mut fields = Vec::with_capacity(rendered.len().saturating_add(1));
    fields.push(profile.cohort.feature_schema.as_str());
    fields.extend(rendered.iter().map(String::as_str));
    super::identity(INPUT_DOMAIN, &fields)
}

/// The identity of the candidate set under the profile's action schema.
#[must_use]
pub fn candidate_set_identity(profile: &super::admission::Profile) -> String {
    let mut sorted = profile.labels.iter().map(String::as_str).collect::<Vec<_>>();
    sorted.sort_unstable();
    let mut fields = Vec::with_capacity(sorted.len().saturating_add(1));
    fields.push(profile.cohort.action_schema.as_str());
    fields.extend(sorted);
    super::identity(CANDIDATE_SET_DOMAIN, &fields)
}

/// One bounded frozen-policy invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    /// The invocation identity.
    pub identity: String,
    /// The profile identity.
    pub profile: String,
    /// The Condition node.
    pub node: crate::engine::workflow::NodeId,
    /// The projected-input identity.
    pub projected_input: String,
    /// The candidate-set identity.
    pub candidate_set: String,
    /// Raw features in declared order.
    pub features: Vec<i64>,
    /// The declared candidate labels, in profile order.
    pub candidates: Vec<String>,
    /// The immutable artifact and schema pins.
    pub binding: animus_frozen_policy::PolicyBinding,
    /// The deterministic operation budget.
    pub max_operation_count: u64,
}

/// Builds the invocation for `features` at `node`.
#[must_use]
pub fn invocation(
    admitted: &super::admission::Admitted,
    node: crate::engine::workflow::NodeId,
    features: Vec<i64>,
) -> Invocation {
    let profile = &admitted.profile;
    let projected_input = input_identity(profile, &features);
    let candidate_set = candidate_set_identity(profile);
    let node_text = node.0.to_string();
    let identity = super::identity(INVOCATION_DOMAIN, &[
        admitted.identity.as_str(),
        node_text.as_str(),
        projected_input.as_str(),
        candidate_set.as_str(),
    ]);
    debug_assert_eq!(features.len(), profile.input.features.len());
    debug_assert!(!profile.labels.is_empty());
    Invocation {
        identity,
        profile: admitted.identity.clone(),
        node,
        projected_input,
        candidate_set,
        features,
        candidates: profile.labels.clone(),
        binding: animus_frozen_policy::PolicyBinding {
            action_schema: profile.cohort.action_schema.clone(),
            artifact: profile.cohort.artifact.clone(),
            feature_schema: profile.cohort.feature_schema.clone(),
        },
        max_operation_count: profile.limits.max_operation_count,
    }
}
