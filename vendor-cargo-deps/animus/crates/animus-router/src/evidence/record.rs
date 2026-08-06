const EMPTY_MODELS_FIELD: &str = "router_selected_models";
const EMPTY_REDACTIONS_FIELD: &str = "router_receipt_redactions";
const EMPTY_BUCKETS_FIELD: &str = "router_signal_buckets";
const RECIPE_ID_FIELD: crate::FieldName = crate::FieldName::new("router_recipe_id");
const MODEL_LABEL_FIELD: crate::FieldName = crate::FieldName::new("router_model_label");
const TASK_FIELD: crate::FieldName = crate::FieldName::new("router_task_shape");
const HASH_SCHEMA: &str = "animus.router-recipe-receipt.v1";

/// Safe router-recipe receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Receipt {
    recipe_id: alloc::string::String,
    signal_buckets: alloc::vec::Vec<alloc::string::String>,
    selected_models: alloc::vec::Vec<alloc::string::String>,
    worker_count: u32,
    success_count: u32,
    quorum_class: super::QuorumClass,
    fallback_policy: crate::FallbackPolicy,
    policy_hash: crate::SafeHash,
    duration_bucket: super::DurationBucket,
    redactions: alloc::vec::Vec<super::Redaction>,
    identity_hash: alloc::string::String,
}

impl Receipt {
    /// Builds a safe router receipt and deterministic BLAKE3 identity.
    ///
    /// # Errors
    ///
    /// Returns [`crate::ValidationError`] when required safe labels, hashes, or
    /// redaction evidence are malformed or missing.
    pub fn new(draft: ReceiptDraft<'_>) -> Result<Self, crate::ValidationError> {
        let recipe_id = crate::normalized_adapter_field(RECIPE_ID_FIELD, draft.recipe_id)?;
        let signal_buckets = collect_labels(EMPTY_BUCKETS_FIELD, TASK_FIELD, draft.signal_buckets)?;
        let selected_models = collect_labels(EMPTY_MODELS_FIELD, MODEL_LABEL_FIELD, draft.selected_models)?;
        let quorum_class = super::QuorumClass::parse(draft.quorum_class).map_err(crate::ValidationError::Contract)?;
        let fallback_policy =
            crate::FallbackPolicy::parse(draft.fallback_policy).map_err(crate::ValidationError::Contract)?;
        let policy_hash = crate::SafeHash::new(draft.policy_hash).map_err(crate::ValidationError::Contract)?;
        let observed_bucket =
            super::DurationBucket::parse(draft.duration_bucket).map_err(crate::ValidationError::Contract)?;
        let redactions = collect_redactions(draft.redactions)?;
        let identity_hash = crate::hash_parts(&[
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
    pub const fn quorum_class(&self) -> super::QuorumClass {
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
    field: crate::FieldName,
    values: &[&str],
) -> Result<alloc::vec::Vec<alloc::string::String>, crate::ValidationError> {
    if values.is_empty() {
        return Err(crate::AdapterContractError::EmptyCollection { field: empty_field }.into());
    }
    values
        .iter()
        .map(|value| crate::normalized_adapter_field(field, value))
        .collect::<Result<alloc::vec::Vec<_>, _>>()
        .map_err(crate::ValidationError::Contract)
}

fn collect_redactions(values: &[&str]) -> Result<alloc::vec::Vec<super::Redaction>, crate::ValidationError> {
    if values.is_empty() {
        return Err(crate::AdapterContractError::EmptyCollection {
            field: EMPTY_REDACTIONS_FIELD,
        }
        .into());
    }
    values
        .iter()
        .map(|value| super::Redaction::parse(value))
        .collect::<Result<alloc::vec::Vec<_>, _>>()
        .map_err(crate::ValidationError::Contract)
}

const RAW_PROMPT_SENTINELS: &[&str] = &["raw prompt", "prompt:"];
const PROVIDER_SENTINELS: &[&str] = &["provider json", "provider payload"];
const WORKER_SENTINEL: &str = "worker output";
const JUDGE_SENTINEL: &str = "judge rationale";
const HIDDEN_SENTINEL: &str = "hidden reasoning";
const CREDENTIAL_SENTINELS: &[&str] = &["credential", "api_key", "secret"];
const PATH_SENTINELS: &[&str] = &["/home/", "/tmp/"];
const TRACE_SENTINELS: &[&str] = &["unbounded trace", "full trace"];

/// Router receipt leak class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptLeak {
    /// Raw prompt text was present.
    RawPrompt,
    /// Provider JSON or payload was present.
    ProviderPayload,
    /// Worker output was present.
    WorkerOutput,
    /// Judge rationale was present.
    JudgeRationale,
    /// Hidden reasoning text was present.
    HiddenReasoning,
    /// Credential-like material was present.
    Credential,
    /// Full path was present.
    FullPath,
    /// Unbounded trace was present.
    UnboundedTrace,
}

/// Validates safe receipt text for known leak sentinels.
///
/// # Errors
///
/// Returns [`ReceiptLeak`] when unsafe material is detected.
///
/// # Panics
///
/// Panics only if built-in leak sentinel tables are accidentally emptied.
pub fn validate_receipt_text(text: &str) -> Result<(), ReceiptLeak> {
    assert!(!RAW_PROMPT_SENTINELS.is_empty(), "raw prompt sentinels must be configured");
    assert!(!TRACE_SENTINELS.is_empty(), "trace sentinels must be configured");
    leak_for_text(text).map_or(Ok(()), Err)
}

fn leak_for_text(text: &str) -> Option<ReceiptLeak> {
    assert!(!PROVIDER_SENTINELS.is_empty(), "provider sentinels must be configured");
    assert!(!CREDENTIAL_SENTINELS.is_empty(), "credential sentinels must be configured");
    let lower = text.to_ascii_lowercase();
    if contains_any(&lower, RAW_PROMPT_SENTINELS) {
        return Some(ReceiptLeak::RawPrompt);
    }
    if contains_any(&lower, PROVIDER_SENTINELS) {
        return Some(ReceiptLeak::ProviderPayload);
    }
    if lower.contains(WORKER_SENTINEL) {
        return Some(ReceiptLeak::WorkerOutput);
    }
    if lower.contains(JUDGE_SENTINEL) {
        return Some(ReceiptLeak::JudgeRationale);
    }
    if lower.contains(HIDDEN_SENTINEL) {
        return Some(ReceiptLeak::HiddenReasoning);
    }
    if contains_any(&lower, CREDENTIAL_SENTINELS) {
        return Some(ReceiptLeak::Credential);
    }
    if contains_any(&lower, PATH_SENTINELS) {
        return Some(ReceiptLeak::FullPath);
    }
    if contains_any(&lower, TRACE_SENTINELS) {
        return Some(ReceiptLeak::UnboundedTrace);
    }
    None
}

fn contains_any(text: &str, sentinels: &[&str]) -> bool {
    sentinels.iter().any(|sentinel| text.contains(sentinel))
}

pub(super) fn assemble_execution_plan(
    selected: &crate::SelectedPlan,
    role_steps: alloc::vec::Vec<super::screen::RoleStep>,
    fallback_chain: super::screen::FallbackChain,
    rotation_decisions: alloc::vec::Vec<crate::RouterRotationDecision>,
) -> super::screen::RouterExecutionPlan {
    super::screen::RouterExecutionPlan {
        recipe_id: alloc::string::String::from(selected.recipe_id()),
        family: selected.family(),
        selected_models: selected.selected_models().to_vec(),
        output_contract: selected.output_contract(),
        bounds: selected.bounds().clone(),
        privacy: selected.privacy(),
        fallback_eligibility: selected.fallback_eligibility(),
        role_steps,
        remaining_synthesis_attempts: if selected.family() == crate::RouteFamily::Single {
            crate::limit::ZERO_COUNT
        } else {
            selected.bounds().max_synthesis_attempts()
        },
        remaining_repair_attempts: if selected.family() == crate::RouteFamily::Single {
            crate::limit::ZERO_COUNT
        } else {
            selected.bounds().max_repair_attempts()
        },
        fallback_chain,
        rotation_decisions,
    }
}
