//! The versioned frozen-policy artifact and its admission before any decision.
//!
//! The canonical encoding is the compact JSON of the typed document, whose keys are declared in
//! sorted order at every level, so input spacing and key order never change it. The artifact
//! identity is the domain-separated `b3:` identity of that encoding under [`ARTIFACT_SCHEMA`]; the
//! model identity covers the model section alone, so two artifacts with the same weights share it.

mod wire;

/// Artifact schema this crate admits.
pub const ARTIFACT_SCHEMA: &str = "animus.frozen-policy.v1";
/// Identity domain of an artifact's model section: its weights digest.
pub const MODEL_DOMAIN: &str = "animus.frozen-policy-model.v1";
/// The only feature normalization: `(value - min) * 1000 / (max - min)`, rounding down.
pub const NORMALIZATION: &str = "min-max-per-mille-v1";
/// Policy kind of a frozen binary decision tree over normalized features.
pub const TREE_POLICY_KIND: &str = "decision-tree-v1";
/// Policy kind of a small fixed-point multilayer perceptron over normalized features.
pub const MLP_POLICY_KIND: &str = "mlp-per-mille-v1";
/// Runtime crate an artifact must name.
pub const RUNTIME_CRATE: &str = "animus-frozen-policy";
/// Artifact API version this crate supports.
pub const ARTIFACT_API: u32 = 1;
/// Largest artifact encoding admitted, in bytes.
pub const MAX_ARTIFACT_BYTES: usize = 65_536;
/// Most features one artifact may declare.
pub const MAX_FEATURES: usize = 64;
/// Most actions one artifact may declare.
pub const MAX_ACTIONS: usize = 64;
/// Longest feature name or action label, in bytes.
pub const MAX_LABEL_BYTES: usize = 64;
/// Largest magnitude of a declared raw feature bound.
pub const MAX_FEATURE_MAGNITUDE: i64 = 1_000_000_000;
/// Largest magnitude of a weight, bias, leaf score, or score bound.
pub const MAX_PARAMETER_MAGNITUDE: i64 = 1_000_000;
/// Largest declared operation budget per decision.
pub const MAX_OPERATION_BUDGET: u64 = 1_000_000;
/// Most nodes of one decision tree.
pub const MAX_TREE_NODES: usize = 255;
/// Most splits on any root-to-leaf path of one decision tree.
pub const MAX_TREE_DEPTH: u64 = 16;
/// Most layers of one MLP.
pub const MAX_LAYERS: usize = 4;
/// Most outputs of one MLP layer.
pub const MAX_LAYER_WIDTH: usize = 64;

/// The declared raw range of one feature.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeatureBound {
    /// The feature's name.
    pub name: alloc::string::String,
    /// The smallest admitted raw value.
    pub min: i64,
    /// The largest admitted raw value.
    pub max: i64,
}

/// An artifact admitted for decisions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedPolicy {
    pub(crate) identity: alloc::string::String,
    pub(crate) model_identity: alloc::string::String,
    pub(crate) policy_kind: &'static str,
    pub(crate) feature_schema: alloc::string::String,
    pub(crate) action_schema: alloc::string::String,
    pub(crate) features: alloc::vec::Vec<FeatureBound>,
    pub(crate) actions: alloc::vec::Vec<alloc::string::String>,
    pub(crate) model: crate::inference::Model,
    pub(crate) score_min: i64,
    pub(crate) score_max: i64,
    pub(crate) abstain_below: Option<i64>,
    pub(crate) worst_case_operations: u64,
    pub(crate) provenance_kind: alloc::string::String,
}

impl AdmittedPolicy {
    /// The artifact's canonical `b3:` identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.identity
    }

    /// The `b3:` identity of the artifact's model section: its weights digest.
    #[must_use]
    pub fn model_identity(&self) -> &str {
        &self.model_identity
    }

    /// The policy kind, [`TREE_POLICY_KIND`] or [`MLP_POLICY_KIND`].
    #[must_use]
    pub const fn policy_kind(&self) -> &'static str {
        self.policy_kind
    }

    /// The feature schema identity.
    #[must_use]
    pub fn feature_schema(&self) -> &str {
        &self.feature_schema
    }

    /// The action schema identity.
    #[must_use]
    pub fn action_schema(&self) -> &str {
        &self.action_schema
    }

    /// The declared features, in input order.
    #[must_use]
    pub fn features(&self) -> &[FeatureBound] {
        &self.features
    }

    /// The declared action labels, in declared order; ties select the earliest.
    #[must_use]
    pub fn actions(&self) -> &[alloc::string::String] {
        &self.actions
    }

    /// The inclusive score bounds every output must satisfy.
    #[must_use]
    pub const fn score_bounds(&self) -> (i64, i64) {
        (self.score_min, self.score_max)
    }

    /// The score below which the best eligible action abstains, if declared.
    #[must_use]
    pub const fn abstain_below(&self) -> Option<i64> {
        self.abstain_below
    }

    /// The most inference operations one decision can take.
    #[must_use]
    pub const fn worst_case_operations(&self) -> u64 {
        self.worst_case_operations
    }

    /// The declared provenance kind: `fixture`, `trained`, or `distilled`.
    #[must_use]
    pub fn provenance_kind(&self) -> &str {
        &self.provenance_kind
    }
}

/// Why an artifact was refused before any decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmissionError {
    /// No artifact bytes were supplied.
    Unavailable,
    /// The encoding exceeds [`MAX_ARTIFACT_BYTES`].
    TooLarge,
    /// The bytes are not one artifact document, or hold unknown fields or another policy kind.
    Malformed,
    /// Another artifact schema.
    Schema,
    /// Another runtime crate or artifact API.
    Runtime,
    /// The feature schema identity, normalization, names, count, or raw bounds are invalid.
    FeatureSchema,
    /// The action schema identity, labels, or count are invalid.
    ActionSchema,
    /// The model's shape does not match its features and actions, or is out of bounds.
    Model,
    /// A weight, bias, or leaf score exceeds [`MAX_PARAMETER_MAGNITUDE`].
    Parameters,
    /// Score bounds, abstention threshold, or operation budget are invalid.
    Bounds,
    /// The provenance kind is undeclared, or its producer, evaluation, or non-claims are empty.
    Provenance,
    /// The canonical identity differs from the selected identity.
    Identity,
}

impl AdmissionError {
    /// The stable kebab-case code of the refusal.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Unavailable => "unavailable",
            Self::TooLarge => "too-large",
            Self::Malformed => "malformed",
            Self::Schema => "schema",
            Self::Runtime => "runtime",
            Self::FeatureSchema => "feature-schema",
            Self::ActionSchema => "action-schema",
            Self::Model => "model",
            Self::Parameters => "parameters",
            Self::Bounds => "bounds",
            Self::Provenance => "provenance",
            Self::Identity => "identity",
        }
    }
}

/// The canonical `b3:` identity of one artifact encoding, for tooling that selects an artifact.
///
/// # Errors
///
/// Returns [`AdmissionError::Unavailable`], [`AdmissionError::TooLarge`], or
/// [`AdmissionError::Malformed`] for bytes that are not one bounded artifact document.
pub fn policy_identity(bytes: &[u8]) -> Result<alloc::string::String, AdmissionError> {
    wire::canonical_identity(&wire::parse(bytes)?)
}

/// Admits the artifact `bytes` when every contract holds and their canonical identity equals
/// `expected_identity`.
///
/// A file name, model name, or path is never identity: only the canonical encoding is.
///
/// # Errors
///
/// Returns the first [`AdmissionError`] that refuses the artifact; nothing is evaluated.
// r[impl agent_harness_runtime.frozen_policy.artifact]
pub fn admit_policy(
    bytes: &[u8],
    expected_identity: &str,
) -> Result<AdmittedPolicy, AdmissionError> {
    let document = wire::parse(bytes)?;
    let checked = wire::check(&document)?;
    let shape = crate::inference::Shape {
        feature_count: checked.features.len(),
        action_count: checked.actions.len(),
    };
    let (model, worst_case_operations) = crate::inference::admit(&document.model, shape)?;
    if worst_case_operations > document.bounds.operation_budget {
        return Err(AdmissionError::Bounds);
    }
    let identity = wire::canonical_identity(&document)?;
    if identity != expected_identity {
        return Err(AdmissionError::Identity);
    }
    let model_encoding =
        serde_json::to_string(&document.model).map_err(|_| AdmissionError::Malformed)?;
    debug_assert!(crate::digest::is_b3_identity(&identity));
    debug_assert!(worst_case_operations <= MAX_OPERATION_BUDGET);
    Ok(AdmittedPolicy {
        identity,
        model_identity: crate::digest::domain_identity(MODEL_DOMAIN, &[&model_encoding]),
        policy_kind: document.model.kind(),
        feature_schema: document.feature_schema.id,
        action_schema: document.action_schema.id,
        features: checked.features,
        actions: checked.actions,
        model,
        score_min: document.bounds.score_min,
        score_max: document.bounds.score_max,
        abstain_below: document.bounds.abstain_below,
        worst_case_operations,
        provenance_kind: document.provenance.kind,
    })
}
