const EMPTY_MODELS_FIELD: &str = "router_selected_models";
const EMPTY_REDACTIONS_FIELD: &str = "router_receipt_redactions";
const EMPTY_BUCKETS_FIELD: &str = "router_signal_buckets";
const RECIPE_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("router_recipe_id");
const MODEL_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("router_model_label");
const TASK_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("router_task_shape");
const QUORUM_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("router_quorum_class");
const DURATION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("router_duration_bucket");
const REDACTION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("router_redaction_marker");
const HASH_SCHEMA: &str = "animus.router-recipe-receipt.v1";
const QUORUM_NONE: &str = "none";
const QUORUM_MET: &str = "met";
const QUORUM_MISSING: &str = "missing";
const DURATION_FAST: &str = "fast";
const DURATION_BOUNDED: &str = "bounded";
const DURATION_SLOW: &str = "slow";
const REDACTION_PROMPT_OMITTED: &str = "prompt_omitted";
const REDACTION_PROVIDER_PAYLOAD_OMITTED: &str = "provider_payload_omitted";
const REDACTION_WORKER_OUTPUT_OMITTED: &str = "worker_output_omitted";
const REDACTION_JUDGE_RATIONALE_OMITTED: &str = "judge_rationale_omitted";
const REDACTION_HIDDEN_REASONING_OMITTED: &str = "hidden_reasoning_omitted";
const REDACTION_CREDENTIAL_OMITTED: &str = "credential_omitted";
const REDACTION_PATH_REDACTED: &str = "path_redacted";
const REDACTION_TRACE_BOUNDED: &str = "trace_bounded";
/// Quorum class recorded in router receipts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuorumClass {
    /// Quorum was not required.
    None,
    /// Quorum was met.
    Met,
    /// Quorum was missing.
    Missing,
}
impl QuorumClass {
    /// Parses a quorum class.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for unsupported labels.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::super::parse_value(QUORUM_FIELD, value, |normalized| match normalized {
            QUORUM_NONE => Some(Self::None),
            QUORUM_MET => Some(Self::Met),
            QUORUM_MISSING => Some(Self::Missing),
            _ => None,
        })
    }
    /// Returns stable quorum label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => QUORUM_NONE,
            Self::Met => QUORUM_MET,
            Self::Missing => QUORUM_MISSING,
        }
    }
}
/// Duration bucket recorded in router receipts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DurationBucket {
    /// Fast host-reported duration bucket.
    Fast,
    /// Bounded host-reported duration bucket.
    Bounded,
    /// Slow host-reported duration bucket.
    Slow,
}
impl DurationBucket {
    /// Parses a duration bucket.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for unsupported labels.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::super::parse_value(DURATION_FIELD, value, |normalized| match normalized {
            DURATION_FAST => Some(Self::Fast),
            DURATION_BOUNDED => Some(Self::Bounded),
            DURATION_SLOW => Some(Self::Slow),
            _ => None,
        })
    }
    /// Returns stable duration label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fast => DURATION_FAST,
            Self::Bounded => DURATION_BOUNDED,
            Self::Slow => DURATION_SLOW,
        }
    }
}
/// Router receipt redaction marker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Redaction {
    /// Prompt text omitted.
    PromptOmitted,
    /// Provider payload omitted.
    ProviderPayloadOmitted,
    /// Worker output omitted.
    WorkerOutputOmitted,
    /// Judge rationale omitted.
    JudgeRationaleOmitted,
    /// Hidden reasoning omitted.
    HiddenReasoningOmitted,
    /// Credentials omitted.
    CredentialOmitted,
    /// Full paths redacted.
    PathRedacted,
    /// Trace details bounded.
    TraceBounded,
}
impl Redaction {
    /// Parses a redaction marker.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for unsupported labels.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::super::parse_value(REDACTION_FIELD, value, |normalized| match normalized {
            REDACTION_PROMPT_OMITTED => Some(Self::PromptOmitted),
            REDACTION_PROVIDER_PAYLOAD_OMITTED => Some(Self::ProviderPayloadOmitted),
            REDACTION_WORKER_OUTPUT_OMITTED => Some(Self::WorkerOutputOmitted),
            REDACTION_JUDGE_RATIONALE_OMITTED => Some(Self::JudgeRationaleOmitted),
            REDACTION_HIDDEN_REASONING_OMITTED => Some(Self::HiddenReasoningOmitted),
            REDACTION_CREDENTIAL_OMITTED => Some(Self::CredentialOmitted),
            REDACTION_PATH_REDACTED => Some(Self::PathRedacted),
            REDACTION_TRACE_BOUNDED => Some(Self::TraceBounded),
            _ => None,
        })
    }
    /// Returns stable redaction label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PromptOmitted => REDACTION_PROMPT_OMITTED,
            Self::ProviderPayloadOmitted => REDACTION_PROVIDER_PAYLOAD_OMITTED,
            Self::WorkerOutputOmitted => REDACTION_WORKER_OUTPUT_OMITTED,
            Self::JudgeRationaleOmitted => REDACTION_JUDGE_RATIONALE_OMITTED,
            Self::HiddenReasoningOmitted => REDACTION_HIDDEN_REASONING_OMITTED,
            Self::CredentialOmitted => REDACTION_CREDENTIAL_OMITTED,
            Self::PathRedacted => REDACTION_PATH_REDACTED,
            Self::TraceBounded => REDACTION_TRACE_BOUNDED,
        }
    }
}
/// Safe router-recipe receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Receipt {
    recipe_id: alloc::string::String,
    signal_buckets: alloc::vec::Vec<alloc::string::String>,
    selected_models: alloc::vec::Vec<alloc::string::String>,
    worker_count: u32,
    success_count: u32,
    quorum_class: QuorumClass,
    fallback_policy: super::super::FallbackPolicy,
    policy_hash: crate::effect::SafeHash,
    duration_bucket: DurationBucket,
    redactions: alloc::vec::Vec<Redaction>,
    identity_hash: alloc::string::String,
}
impl Receipt {
    /// Builds a safe router receipt and deterministic BLAKE3 identity.
    ///
    /// # Errors
    ///
    /// Returns [`super::super::ValidationError`] when required safe labels, hashes, or
    /// redaction evidence are malformed or missing.
    pub fn new(draft: ReceiptDraft<'_>) -> Result<Self, super::super::ValidationError> {
        let recipe_id =
            crate::validation::normalized_adapter_field(RECIPE_ID_FIELD, draft.recipe_id)?;
        let signal_buckets = collect_labels(EMPTY_BUCKETS_FIELD, TASK_FIELD, draft.signal_buckets)?;
        let selected_models =
            collect_labels(EMPTY_MODELS_FIELD, MODEL_LABEL_FIELD, draft.selected_models)?;
        let quorum_class = QuorumClass::parse(draft.quorum_class)
            .map_err(super::super::ValidationError::Contract)?;
        let fallback_policy = super::super::FallbackPolicy::parse(draft.fallback_policy)
            .map_err(super::super::ValidationError::Contract)?;
        let policy_hash = crate::effect::SafeHash::new(draft.policy_hash)
            .map_err(super::super::ValidationError::Contract)?;
        let observed_bucket = DurationBucket::parse(draft.duration_bucket)
            .map_err(super::super::ValidationError::Contract)?;
        let redactions = collect_redactions(draft.redactions)?;
        let identity_hash = super::super::hash_parts(&[
            HASH_SCHEMA,
            recipe_id.as_str(),
            quorum_class.as_str(),
            fallback_policy.as_str(),
            policy_hash.as_str(),
            observed_bucket.as_str(),
        ]);
        Ok(Self {
            recipe_id,
            signal_buckets,
            selected_models,
            worker_count: draft.worker_count,
            success_count: draft.success_count,
            quorum_class,
            fallback_policy,
            policy_hash,
            duration_bucket: observed_bucket,
            redactions,
            identity_hash,
        })
    }
    /// Returns recipe identifier.
    #[must_use]
    pub fn recipe_id(&self) -> &str {
        self.recipe_id.as_str()
    }
    /// Returns bounded worker count.
    #[must_use]
    pub const fn worker_count(&self) -> u32 {
        self.worker_count
    }
    /// Returns bounded success count.
    #[must_use]
    pub const fn success_count(&self) -> u32 {
        self.success_count
    }
    /// Returns quorum class.
    #[must_use]
    pub const fn quorum_class(&self) -> QuorumClass {
        self.quorum_class
    }
    /// Returns deterministic BLAKE3 receipt identity.
    #[must_use]
    pub fn identity_hash(&self) -> &str {
        self.identity_hash.as_str()
    }
}
/// Borrowed fields used to build a safe router receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReceiptDraft<'a> {
    /// Recipe identifier.
    pub recipe_id: &'a str,
    /// Safe signal bucket labels.
    pub signal_buckets: &'a [&'a str],
    /// Safe selected model labels.
    pub selected_models: &'a [&'a str],
    /// Bounded worker count.
    pub worker_count: u32,
    /// Bounded success count.
    pub success_count: u32,
    /// Quorum class label.
    pub quorum_class: &'a str,
    /// Fallback policy label.
    pub fallback_policy: &'a str,
    /// BLAKE3 policy hash.
    pub policy_hash: &'a str,
    /// Host-reported duration bucket label.
    pub duration_bucket: &'a str,
    /// Redaction marker labels.
    pub redactions: &'a [&'a str],
}
fn collect_labels(
    empty_field: &'static str,
    field: crate::validation::FieldName,
    values: &[&str],
) -> Result<alloc::vec::Vec<alloc::string::String>, super::super::ValidationError> {
    if values.is_empty() {
        return Err(crate::AdapterContractError::EmptyCollection { field: empty_field }.into());
    }
    values
        .iter()
        .map(|value| crate::validation::normalized_adapter_field(field, value))
        .collect::<Result<alloc::vec::Vec<_>, _>>()
        .map_err(super::super::ValidationError::Contract)
}
fn collect_redactions(
    values: &[&str],
) -> Result<alloc::vec::Vec<Redaction>, super::super::ValidationError> {
    if values.is_empty() {
        return Err(crate::AdapterContractError::EmptyCollection {
            field: EMPTY_REDACTIONS_FIELD,
        }
        .into());
    }
    values
        .iter()
        .map(|value| Redaction::parse(value))
        .collect::<Result<alloc::vec::Vec<_>, _>>()
        .map_err(super::super::ValidationError::Contract)
}
