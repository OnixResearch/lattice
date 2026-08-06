use alloc::string::ToString;

const STEP_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("recipe_dag_step_id");
const STEP_KIND_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("recipe_dag_step_kind");
const DEPENDENCY_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("recipe_dag_dependency");
const INPUT_REF_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("recipe_dag_input_ref");
const OUTPUT_REF_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("recipe_dag_output_ref");
const GATE_REF_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("recipe_dag_validation_gate");
const RETRY_REF_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("recipe_dag_retry_ref");
const SUBRECIPE_REF_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("recipe_dag_subrecipe_ref");
const CAPABILITY_REF_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("recipe_dag_capability_ref");
const DENIED_CAPABILITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("recipe_dag_denied_capability");
const DENIED_GATE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("recipe_dag_denied_gate");
const DENIED_SUBRECIPE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("recipe_dag_denied_subrecipe");

const KIND_PROMPT: &str = "prompt";
const KIND_TOOL: &str = "tool";
const KIND_VALIDATION_GATE: &str = "validation_gate";
const KIND_SUBRECIPE: &str = "subrecipe";
const KIND_RETRY: &str = "retry";
const KIND_CLEANUP: &str = "cleanup";
const RECIPE_DAG_SCHEMA: &str = "animus.recipe-dag.receipt.v1";
const REDACTION_MARKER: &str = "raw-workflow-material-redacted";
const OUTPUT_REFERENCE_SEPARATOR: char = '.';
const MAX_DAG_STEPS: usize = 32;
const MAX_DAG_EDGES_PER_STEP: usize = 16;
const MAX_DAG_REFS_PER_STEP: usize = 16;
const RECEIPT_ISSUE_NONE: &str = "none";
const BATCH_INDEX_ADVANCE: u32 = 1;
const ISSUE_CAPABILITY_DENIED: &str = "RECIPE_DAG_CAPABILITY_DENIED";
const ISSUE_VALIDATION_GATE_DENIED: &str = "RECIPE_DAG_VALIDATION_GATE_DENIED";
const ISSUE_SUBRECIPE_DENIED: &str = "RECIPE_DAG_SUBRECIPE_DENIED";
const ISSUE_DEPENDENCY_BLOCKED: &str = "RECIPE_DAG_DEPENDENCY_BLOCKED";
const UNSAFE_LABEL_PATTERNS: &[&str] = &[
    "/home/",
    "/tmp/",
    "raw-output",
    "raw_tool_body",
    "raw-model-response",
    "provider-payload",
    "provider_payload",
    "credential",
    "secret=",
    "token=",
    "password=",
    "diff --git",
    "unbounded-log",
];

const _: () = assert!(MAX_DAG_STEPS > 0);
const _: () = assert!(MAX_DAG_EDGES_PER_STEP > 0);
const _: () = assert!(MAX_DAG_REFS_PER_STEP > 0);

/// Untrusted optional recipe DAG step metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecipeDagStepDraft<'a> {
    /// Stable step identifier used by dependencies and receipts.
    pub step_id: &'a str,
    /// Step kind label.
    pub kind: &'a str,
    /// Upstream step identifiers that must finish before this step is ready.
    pub depends_on: &'a [&'a str],
    /// Declared parameter refs or upstream `step.output` refs consumed by this step.
    pub input_refs: &'a [&'a str],
    /// Safe output labels this step may publish for downstream fan-in.
    pub output_refs: &'a [&'a str],
    /// Validation gate labels required by this step.
    pub validation_gates: &'a [&'a str],
    /// Retry check labels this step may request from the host.
    pub retry_refs: &'a [&'a str],
    /// Subrecipe identities this step may invoke through host admission.
    pub subrecipe_refs: &'a [&'a str],
    /// Capability ceilings this step may require.
    pub capability_ceilings: &'a [&'a str],
}

/// Supported recipe DAG step kinds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecipeDagStepKind {
    /// Prompt/model-planning step represented as data only.
    Prompt,
    /// Tool request step represented as a host-effect descriptor only.
    Tool,
    /// Validation gate step.
    ValidationGate,
    /// Subrecipe call step.
    Subrecipe,
    /// Retry planning step.
    Retry,
    /// Cleanup request step.
    Cleanup,
}

impl RecipeDagStepKind {
    /// Parses a stable DAG step kind label.
    ///
    /// # Errors
    ///
    /// Returns [`super::RecipeWorkflowError`] when the label is empty or unsupported.
    pub fn parse(value: &str) -> Result<Self, super::RecipeWorkflowError> {
        super::parse_workflow_value(STEP_KIND_FIELD, value, Self::from_normalized)
    }

    fn from_normalized(value: &str) -> Option<Self> {
        match value {
            KIND_PROMPT => Some(Self::Prompt),
            KIND_TOOL => Some(Self::Tool),
            KIND_VALIDATION_GATE => Some(Self::ValidationGate),
            KIND_SUBRECIPE => Some(Self::Subrecipe),
            KIND_RETRY => Some(Self::Retry),
            KIND_CLEANUP => Some(Self::Cleanup),
            _ => None,
        }
    }

    /// Returns the stable DAG kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Prompt => KIND_PROMPT,
            Self::Tool => KIND_TOOL,
            Self::ValidationGate => KIND_VALIDATION_GATE,
            Self::Subrecipe => KIND_SUBRECIPE,
            Self::Retry => KIND_RETRY,
            Self::Cleanup => KIND_CLEANUP,
        }
    }
}

/// Validated recipe DAG step metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipeDagStep {
    step_id: alloc::string::String,
    kind: RecipeDagStepKind,
    depends_on: alloc::vec::Vec<alloc::string::String>,
    input_refs: alloc::vec::Vec<alloc::string::String>,
    output_refs: alloc::vec::Vec<alloc::string::String>,
    validation_gates: alloc::vec::Vec<alloc::string::String>,
    retry_refs: alloc::vec::Vec<alloc::string::String>,
    subrecipe_refs: alloc::vec::Vec<alloc::string::String>,
    capability_ceilings: alloc::vec::Vec<alloc::string::String>,
}

impl RecipeDagStep {
    /// Returns the stable step identifier.
    #[must_use]
    pub fn step_id(&self) -> &str {
        self.step_id.as_str()
    }

    /// Returns the step kind.
    #[must_use]
    pub const fn kind(&self) -> RecipeDagStepKind {
        self.kind
    }

    /// Returns upstream step identifiers.
    #[must_use]
    pub fn depends_on(&self) -> &[alloc::string::String] {
        self.depends_on.as_slice()
    }

    /// Returns declared input references.
    #[must_use]
    pub fn input_refs(&self) -> &[alloc::string::String] {
        self.input_refs.as_slice()
    }

    /// Returns declared output references.
    #[must_use]
    pub fn output_refs(&self) -> &[alloc::string::String] {
        self.output_refs.as_slice()
    }

    /// Returns validation gate references.
    #[must_use]
    pub fn validation_gates(&self) -> &[alloc::string::String] {
        self.validation_gates.as_slice()
    }

    /// Returns retry check references.
    #[must_use]
    pub fn retry_refs(&self) -> &[alloc::string::String] {
        self.retry_refs.as_slice()
    }

    /// Returns subrecipe references.
    #[must_use]
    pub fn subrecipe_refs(&self) -> &[alloc::string::String] {
        self.subrecipe_refs.as_slice()
    }

    /// Returns step capability ceilings.
    #[must_use]
    pub fn capability_ceilings(&self) -> &[alloc::string::String] {
        self.capability_ceilings.as_slice()
    }
}

/// Deterministic execution batch for host-owned scheduling.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipeDagBatch {
    index: u32,
    step_ids: alloc::vec::Vec<alloc::string::String>,
}

impl RecipeDagBatch {
    /// Returns the deterministic batch index.
    #[must_use]
    pub const fn index(&self) -> u32 {
        self.index
    }

    /// Returns sorted step identifiers in this batch.
    #[must_use]
    pub fn step_ids(&self) -> &[alloc::string::String] {
        self.step_ids.as_slice()
    }
}

/// Reason a DAG step is blocked by host admission state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecipeDagBlockReason {
    /// The host denied a declared capability ceiling.
    CapabilityDenied,
    /// The host denied a required validation gate.
    ValidationGateDenied,
    /// The host denied or has not trusted a subrecipe reference.
    SubrecipeDenied,
    /// An upstream dependency is blocked.
    DependencyBlocked,
}

impl RecipeDagBlockReason {
    /// Returns a stable safe issue code for receipt projection.
    #[must_use]
    pub const fn issue_code(self) -> &'static str {
        match self {
            Self::CapabilityDenied => ISSUE_CAPABILITY_DENIED,
            Self::ValidationGateDenied => ISSUE_VALIDATION_GATE_DENIED,
            Self::SubrecipeDenied => ISSUE_SUBRECIPE_DENIED,
            Self::DependencyBlocked => ISSUE_DEPENDENCY_BLOCKED,
        }
    }
}

/// Host-admission explanation for a blocked recipe DAG step.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipeDagBlockedStep {
    step_id: alloc::string::String,
    reason: RecipeDagBlockReason,
    detail: alloc::string::String,
}

impl RecipeDagBlockedStep {
    /// Returns the blocked step identifier.
    #[must_use]
    pub fn step_id(&self) -> &str {
        self.step_id.as_str()
    }

    /// Returns the stable block reason.
    #[must_use]
    pub const fn reason(&self) -> RecipeDagBlockReason {
        self.reason
    }

    /// Returns the safe label that caused the block.
    #[must_use]
    pub fn detail(&self) -> &str {
        self.detail.as_str()
    }
}

/// Safe recipe DAG receipt projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipeDagReceipt {
    schema_label: &'static str,
    identity_hash: alloc::string::String,
    step_count: u32,
    edge_count: u32,
    issue_codes: alloc::vec::Vec<alloc::string::String>,
    redaction_markers: alloc::vec::Vec<alloc::string::String>,
}

impl RecipeDagReceipt {
    /// Returns the receipt schema label.
    #[must_use]
    pub const fn schema_label(&self) -> &'static str {
        self.schema_label
    }

    /// Returns the BLAKE3 identity for the safe DAG projection.
    #[must_use]
    pub fn identity_hash(&self) -> &str {
        self.identity_hash.as_str()
    }

    /// Returns the bounded step count.
    #[must_use]
    pub const fn step_count(&self) -> u32 {
        self.step_count
    }

    /// Returns the bounded edge count.
    #[must_use]
    pub const fn edge_count(&self) -> u32 {
        self.edge_count
    }

    /// Returns safe issue codes included in the receipt.
    #[must_use]
    pub fn issue_codes(&self) -> &[alloc::string::String] {
        self.issue_codes.as_slice()
    }

    /// Returns redaction markers for omitted raw workflow material.
    #[must_use]
    pub fn redaction_markers(&self) -> &[alloc::string::String] {
        self.redaction_markers.as_slice()
    }
}

/// Validated recipe DAG execution plan data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipeDagPlan {
    steps: alloc::vec::Vec<RecipeDagStep>,
    batches: alloc::vec::Vec<RecipeDagBatch>,
    blocked_steps: alloc::vec::Vec<RecipeDagBlockedStep>,
    receipt: RecipeDagReceipt,
}

impl RecipeDagPlan {
    /// Returns validated DAG steps.
    #[must_use]
    pub fn steps(&self) -> &[RecipeDagStep] {
        self.steps.as_slice()
    }

    /// Returns deterministic topological batches for host-owned scheduling.
    #[must_use]
    pub fn batches(&self) -> &[RecipeDagBatch] {
        self.batches.as_slice()
    }

    /// Returns host-admission blocked step explanations.
    #[must_use]
    pub fn blocked_steps(&self) -> &[RecipeDagBlockedStep] {
        self.blocked_steps.as_slice()
    }

    /// Returns the safe plan receipt.
    #[must_use]
    pub const fn receipt(&self) -> &RecipeDagReceipt {
        &self.receipt
    }
}

/// Host admission state used to block DAG steps without executing them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecipeDagAdmission<'a> {
    /// Capability labels denied by host policy.
    pub capabilities: &'a [&'a str],
    /// Validation gate labels denied or unavailable in the host.
    pub validation_gates: &'a [&'a str],
    /// Subrecipe identities denied, stale, or not trusted by the host.
    pub subrecipes: &'a [&'a str],
}

#[derive(Clone, Copy)]
pub(super) struct RecipeDagContext<'a> {
    pub(super) workflow_id: &'a str,
    pub(super) parameters: &'a [super::field::RecipeParameter],
    pub(super) capability_ceilings: &'a [alloc::string::String],
    pub(super) validation_gates: &'a [alloc::string::String],
    pub(super) retry_policy: &'a super::attempt::RecipeRetryPolicy,
    pub(super) subrecipes: &'a [alloc::string::String],
}

pub(super) fn validate_recipe_dag(
    drafts: &[RecipeDagStepDraft<'_>],
    context: RecipeDagContext<'_>,
) -> Result<Option<RecipeDagPlan>, super::RecipeWorkflowError> {
    if drafts.is_empty() {
        return Ok(None);
    }
    ensure_limit(ItemLimit {
        field_name: STEP_ID_FIELD,
        actual_items: drafts.len(),
        max_items: MAX_DAG_STEPS,
    })?;
    assert!(!drafts.is_empty(), "non-empty DAG must have drafts");
    assert!(drafts.len() <= MAX_DAG_STEPS, "DAG step cap must hold");
    let steps = collect_steps(drafts)?;
    validate_structure(steps.as_slice())?;
    validate_recipe_gates(steps.as_slice(), &context)?;
    let batches = topological_batches(steps.as_slice(), &alloc::collections::BTreeSet::new());
    let blocked_steps = alloc::vec::Vec::new();
    let receipt = build_receipt(context.workflow_id, steps.as_slice(), blocked_steps.as_slice())?;
    Ok(Some(RecipeDagPlan {
        steps,
        batches,
        blocked_steps,
        receipt,
    }))
}

/// Projects a validated recipe DAG with host admission state.
///
/// This function returns plan data only. It does not execute steps, select live
/// parallelism, call providers, invoke tools, persist state, or run cleanup.
///
/// # Errors
///
/// Returns [`RecipeWorkflowError`](super::RecipeWorkflowError) when host-supplied
/// denial labels contain unsafe metadata.
///
/// # Panics
///
/// Panics if a previously validated DAG plan is empty or exceeds the bounded step count. This
/// indicates an internal workflow validation contract violation before host admission projection.
pub fn plan_recipe_dag(
    workflow: &super::data::RecipeWorkflow,
    admission: RecipeDagAdmission<'_>,
) -> Result<Option<RecipeDagPlan>, super::RecipeWorkflowError> {
    let Some(base_plan) = workflow.dag_plan() else {
        return Ok(None);
    };
    assert!(!base_plan.steps().is_empty(), "validated DAG must keep steps");
    assert!(base_plan.steps().len() <= MAX_DAG_STEPS, "validated DAG must stay bounded");
    let denied_capabilities =
        collect_safe_texts(DENIED_CAPABILITY_FIELD, admission.capabilities, MAX_DAG_REFS_PER_STEP, false)?;
    let denied_validation_gates =
        collect_safe_texts(DENIED_GATE_FIELD, admission.validation_gates, MAX_DAG_REFS_PER_STEP, false)?;
    let denied_subrecipes =
        collect_safe_texts(DENIED_SUBRECIPE_FIELD, admission.subrecipes, MAX_DAG_REFS_PER_STEP, false)?;
    let blocked_steps = blocked_steps(
        base_plan.steps(),
        denied_capabilities.as_slice(),
        denied_validation_gates.as_slice(),
        denied_subrecipes.as_slice(),
    );
    let blocked_ids: alloc::collections::BTreeSet<alloc::string::String> =
        blocked_steps.iter().map(|blocked| blocked.step_id.clone()).collect();
    let batches = topological_batches(base_plan.steps(), &blocked_ids);
    let receipt = build_receipt(workflow.identity(), base_plan.steps(), blocked_steps.as_slice())?;
    Ok(Some(RecipeDagPlan {
        steps: base_plan.steps.clone(),
        batches,
        blocked_steps,
        receipt,
    }))
}

fn collect_steps(
    drafts: &[RecipeDagStepDraft<'_>],
) -> Result<alloc::vec::Vec<RecipeDagStep>, super::RecipeWorkflowError> {
    assert!(!drafts.is_empty(), "DAG step drafts must be present");
    assert!(drafts.len() <= MAX_DAG_STEPS, "DAG step cap must hold");
    let mut steps = alloc::vec::Vec::with_capacity(drafts.len());
    let mut seen = alloc::collections::BTreeSet::new();
    for draft in drafts {
        let step_id = safe_required(STEP_ID_FIELD, draft.step_id)?;
        if step_id.contains(OUTPUT_REFERENCE_SEPARATOR) {
            return Err(super::RecipeWorkflowError::UnsafeDagLabel {
                field: STEP_ID_FIELD.as_str(),
                value: step_id,
            });
        }
        if !seen.insert(step_id.clone()) {
            return Err(super::RecipeWorkflowError::DuplicateDagStep { step_id });
        }
        let kind = RecipeDagStepKind::parse(draft.kind)?;
        let depends_on = collect_dependencies(&step_id, draft.depends_on)?;
        let input_refs = collect_safe_texts(INPUT_REF_FIELD, draft.input_refs, MAX_DAG_REFS_PER_STEP, false)?;
        let output_refs = collect_safe_texts(OUTPUT_REF_FIELD, draft.output_refs, MAX_DAG_REFS_PER_STEP, false)?;
        let validation_gates =
            collect_safe_texts(GATE_REF_FIELD, draft.validation_gates, MAX_DAG_REFS_PER_STEP, false)?;
        let retry_refs = collect_safe_texts(RETRY_REF_FIELD, draft.retry_refs, MAX_DAG_REFS_PER_STEP, false)?;
        let subrecipe_refs =
            collect_safe_texts(SUBRECIPE_REF_FIELD, draft.subrecipe_refs, MAX_DAG_REFS_PER_STEP, false)?;
        let capability_ceilings =
            collect_safe_texts(CAPABILITY_REF_FIELD, draft.capability_ceilings, MAX_DAG_REFS_PER_STEP, false)?;
        steps.push(RecipeDagStep {
            step_id,
            kind,
            depends_on,
            input_refs,
            output_refs,
            validation_gates,
            retry_refs,
            subrecipe_refs,
            capability_ceilings,
        });
    }
    Ok(steps)
}

fn collect_dependencies(
    step_id: &str,
    values: &[&str],
) -> Result<alloc::vec::Vec<alloc::string::String>, super::RecipeWorkflowError> {
    ensure_limit(ItemLimit {
        field_name: DEPENDENCY_FIELD,
        actual_items: values.len(),
        max_items: MAX_DAG_EDGES_PER_STEP,
    })?;
    assert!(!step_id.is_empty(), "dependency owner step must be named");
    assert!(values.len() <= MAX_DAG_EDGES_PER_STEP, "DAG dependency cap must hold");
    let mut dependencies: alloc::vec::Vec<alloc::string::String> = alloc::vec::Vec::with_capacity(values.len());
    for value in values {
        let dependency = safe_required(DEPENDENCY_FIELD, value)?;
        if dependency == step_id {
            return Err(super::RecipeWorkflowError::SelfDagEdge {
                step_id: step_id.to_string(),
            });
        }
        if dependencies.iter().any(|existing| existing.as_str() == dependency) {
            return Err(super::RecipeWorkflowError::DuplicateDagEdge {
                step_id: step_id.to_string(),
                dependency,
            });
        }
        dependencies.push(dependency);
    }
    Ok(dependencies)
}

fn collect_safe_texts(
    field_name: crate::validation::FieldName,
    values: &[&str],
    max_items: usize,
    required: bool,
) -> Result<alloc::vec::Vec<alloc::string::String>, super::RecipeWorkflowError> {
    ensure_limit(ItemLimit {
        field_name,
        actual_items: values.len(),
        max_items,
    })?;
    assert!(max_items > 0, "collection bound must be positive");
    assert!(values.len() <= max_items, "collection bound must hold");
    if required && values.is_empty() {
        return Err(super::RecipeWorkflowError::EmptyCollection {
            field: field_name.as_str(),
        });
    }
    let mut normalized = alloc::vec::Vec::with_capacity(values.len());
    for value in values {
        let text = safe_required(field_name, value)?;
        if normalized.iter().any(|existing| existing == &text) {
            return Err(super::RecipeWorkflowError::DuplicateName {
                field: field_name.as_str(),
                value: text,
            });
        }
        normalized.push(text);
    }
    Ok(normalized)
}

#[derive(Clone, Copy)]
struct ItemLimit {
    field_name: crate::validation::FieldName,
    actual_items: usize,
    max_items: usize,
}

fn ensure_limit(limit: ItemLimit) -> Result<(), super::RecipeWorkflowError> {
    if limit.actual_items > limit.max_items {
        return Err(super::RecipeWorkflowError::DagLimitExceeded {
            field: limit.field_name.as_str(),
            max_items: limit.max_items,
            actual_items: limit.actual_items,
        });
    }
    Ok(())
}

fn safe_required(
    field_name: crate::validation::FieldName,
    value: &str,
) -> Result<alloc::string::String, super::RecipeWorkflowError> {
    let normalized = super::required_text(field_name, value)?;
    if contains_unsafe_label(normalized.as_str()) {
        return Err(super::RecipeWorkflowError::UnsafeDagLabel {
            field: field_name.as_str(),
            value: normalized,
        });
    }
    Ok(normalized)
}

fn contains_unsafe_label(value: &str) -> bool {
    if value.starts_with('/') || value.chars().any(char::is_control) {
        return true;
    }
    let lower = value.to_ascii_lowercase();
    UNSAFE_LABEL_PATTERNS.iter().any(|pattern| lower.contains(pattern))
}

fn validate_structure(steps: &[RecipeDagStep]) -> Result<(), super::RecipeWorkflowError> {
    assert!(!steps.is_empty(), "DAG structure must include steps");
    assert!(steps.len() <= MAX_DAG_STEPS, "DAG structure must stay bounded");
    let step_ids: alloc::collections::BTreeSet<alloc::string::String> =
        steps.iter().map(|step| step.step_id.clone()).collect();
    for step in steps {
        for dependency in &step.depends_on {
            if !step_ids.contains(dependency) {
                return Err(super::RecipeWorkflowError::UnknownDagDependency {
                    step_id: step.step_id.clone(),
                    dependency: dependency.clone(),
                });
            }
        }
    }
    if let Some(cycle_node) = first_cycle_node(steps) {
        return Err(super::RecipeWorkflowError::DagCycle { step_id: cycle_node });
    }
    Ok(())
}

fn validate_recipe_gates(
    steps: &[RecipeDagStep],
    context: &RecipeDagContext<'_>,
) -> Result<(), super::RecipeWorkflowError> {
    assert!(!steps.is_empty(), "validated DAG must have steps");
    assert!(steps.len() <= MAX_DAG_STEPS, "validated DAG must stay bounded");
    let output_map = output_map(steps);
    for step in steps {
        validate_input_refs(step, context.parameters, &output_map)?;
        validate_subset(
            step.step_id(),
            step.validation_gates(),
            context.validation_gates,
            GateFailure::ValidationGate,
        )?;
        validate_retry_refs(step, context.retry_policy)?;
        validate_subset(step.step_id(), step.subrecipe_refs(), context.subrecipes, GateFailure::Subrecipe)?;
        validate_subset(
            step.step_id(),
            step.capability_ceilings(),
            context.capability_ceilings,
            GateFailure::Capability,
        )?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum GateFailure {
    ValidationGate,
    Subrecipe,
    Capability,
}

fn validate_subset(
    step_id: &str,
    requested: &[alloc::string::String],
    admitted: &[alloc::string::String],
    failure: GateFailure,
) -> Result<(), super::RecipeWorkflowError> {
    assert!(!step_id.is_empty(), "subset owner step must be named");
    assert!(requested.len() <= MAX_DAG_REFS_PER_STEP, "requested DAG references must stay bounded");
    for value in requested {
        if admitted.iter().any(|allowed| allowed == value) {
            continue;
        }
        return match failure {
            GateFailure::ValidationGate => Err(super::RecipeWorkflowError::MissingDagGate {
                step_id: step_id.to_string(),
                gate: value.clone(),
            }),
            GateFailure::Subrecipe => Err(super::RecipeWorkflowError::UntrustedDagSubrecipe {
                step_id: step_id.to_string(),
                subrecipe: value.clone(),
            }),
            GateFailure::Capability => Err(super::RecipeWorkflowError::CapabilityWidening {
                step_id: step_id.to_string(),
                capability: value.clone(),
            }),
        };
    }
    Ok(())
}

fn validate_retry_refs(
    step: &RecipeDagStep,
    policy: &super::attempt::RecipeRetryPolicy,
) -> Result<(), super::RecipeWorkflowError> {
    for retry_ref in step.retry_refs() {
        if policy.checks().iter().any(|check| check.label() == retry_ref) {
            continue;
        }
        return Err(super::RecipeWorkflowError::RetryBypass {
            step_id: step.step_id.clone(),
            retry_ref: retry_ref.clone(),
        });
    }
    Ok(())
}

fn validate_input_refs(
    step: &RecipeDagStep,
    parameters: &[super::field::RecipeParameter],
    output_map: &alloc::collections::BTreeMap<
        alloc::string::String,
        alloc::collections::BTreeSet<alloc::string::String>,
    >,
) -> Result<(), super::RecipeWorkflowError> {
    assert!(!step.step_id().is_empty(), "input owner step must be named");
    assert!(output_map.len() <= MAX_DAG_STEPS, "DAG output map must stay bounded");
    for input_ref in step.input_refs() {
        if parameters.iter().any(|parameter| parameter.name() == input_ref) {
            continue;
        }
        let Some((upstream_step, output_ref)) = input_ref.split_once(OUTPUT_REFERENCE_SEPARATOR) else {
            return Err(super::RecipeWorkflowError::UndeclaredDagInput {
                step_id: step.step_id.clone(),
                reference: input_ref.clone(),
            });
        };
        if !step.depends_on().iter().any(|dependency| dependency == upstream_step) {
            return Err(super::RecipeWorkflowError::UndeclaredDagInput {
                step_id: step.step_id.clone(),
                reference: input_ref.clone(),
            });
        }
        let Some(outputs) = output_map.get(upstream_step) else {
            return Err(super::RecipeWorkflowError::UndeclaredDagOutput {
                step_id: step.step_id.clone(),
                reference: input_ref.clone(),
            });
        };
        if !outputs.contains(output_ref) {
            return Err(super::RecipeWorkflowError::UndeclaredDagOutput {
                step_id: step.step_id.clone(),
                reference: input_ref.clone(),
            });
        }
    }
    Ok(())
}

fn output_map(
    steps: &[RecipeDagStep],
) -> alloc::collections::BTreeMap<alloc::string::String, alloc::collections::BTreeSet<alloc::string::String>> {
    assert!(steps.len() <= MAX_DAG_STEPS, "output map input must stay bounded");
    assert!(!steps.is_empty(), "output map must have DAG steps");
    steps
        .iter()
        .map(|step| {
            (step.step_id.clone(), step.output_refs.iter().cloned().collect::<alloc::collections::BTreeSet<_>>())
        })
        .collect()
}

fn first_cycle_node(steps: &[RecipeDagStep]) -> Option<alloc::string::String> {
    assert!(!steps.is_empty(), "cycle detection requires DAG steps");
    assert!(steps.len() <= MAX_DAG_STEPS, "cycle detection input must stay bounded");
    let all_ids: alloc::collections::BTreeSet<alloc::string::String> =
        steps.iter().map(|step| step.step_id.clone()).collect();
    let mut dependent_edges: alloc::collections::BTreeMap<
        alloc::string::String,
        alloc::vec::Vec<alloc::string::String>,
    > = alloc::collections::BTreeMap::new();
    let mut indegree: alloc::collections::BTreeMap<alloc::string::String, usize> = alloc::collections::BTreeMap::new();
    for step in steps {
        dependent_edges.entry(step.step_id.clone()).or_default();
        indegree.entry(step.step_id.clone()).or_default();
    }
    for step in steps {
        for dependency in &step.depends_on {
            dependent_edges.entry(dependency.clone()).or_default().push(step.step_id.clone());
            *indegree.entry(step.step_id.clone()).or_default() += 1;
        }
    }
    let mut ready = alloc::collections::VecDeque::new();
    for (step_id, count) in &indegree {
        if *count == 0 {
            ready.push_back(step_id.clone());
        }
    }
    let mut processed = alloc::collections::BTreeSet::new();
    while let Some(step_id) = ready.pop_front() {
        if !processed.insert(step_id.clone()) {
            continue;
        }
        if let Some(dependents) = dependent_edges.get(&step_id) {
            for dependent in dependents {
                if let Some(count) = indegree.get_mut(dependent) {
                    if *count > 0 {
                        *count -= 1;
                    }
                    if *count == 0 {
                        ready.push_back(dependent.clone());
                    }
                }
            }
        }
    }
    all_ids.iter().find(|step_id| !processed.contains(*step_id)).cloned()
}

fn topological_batches(
    steps: &[RecipeDagStep],
    blocked_ids: &alloc::collections::BTreeSet<alloc::string::String>,
) -> alloc::vec::Vec<RecipeDagBatch> {
    assert!(steps.len() <= MAX_DAG_STEPS, "batch input must stay bounded");
    assert!(blocked_ids.len() <= steps.len(), "blocked IDs must be a DAG subset");
    let available_ids: alloc::collections::BTreeSet<alloc::string::String> = steps
        .iter()
        .map(|step| step.step_id.clone())
        .filter(|step_id| !blocked_ids.contains(step_id))
        .collect();
    let EdgeMaps {
        dependent_edges,
        mut indegree,
    } = edge_maps(EdgeMapRequest {
        steps,
        available_ids: &available_ids,
    });
    let mut ready: alloc::collections::BTreeSet<alloc::string::String> = indegree
        .iter()
        .filter(|(_step_id, count)| **count == 0)
        .map(|(step_id, _count)| step_id.clone())
        .collect();
    let mut processed = alloc::collections::BTreeSet::new();
    let mut batches = alloc::vec::Vec::with_capacity(steps.len());
    let mut next_batch_index = 0;
    while !ready.is_empty() {
        let step_ids: alloc::vec::Vec<alloc::string::String> = ready.iter().cloned().collect();
        let batch = next_batch(&mut next_batch_index, step_ids.clone());
        batches.push(batch);
        let mut next_ready = alloc::collections::BTreeSet::new();
        for step_id in step_ids {
            if !processed.insert(step_id.clone()) {
                continue;
            }
            if let Some(dependents) = dependent_edges.get(&step_id) {
                for dependent in dependents {
                    if let Some(count) = indegree.get_mut(dependent) {
                        if *count > 0 {
                            *count -= 1;
                        }
                        if *count == 0 && !processed.contains(dependent) {
                            next_ready.insert(dependent.clone());
                        }
                    }
                }
            }
        }
        ready = next_ready;
    }
    batches
}

struct EdgeMaps {
    dependent_edges: alloc::collections::BTreeMap<alloc::string::String, alloc::vec::Vec<alloc::string::String>>,
    indegree: alloc::collections::BTreeMap<alloc::string::String, usize>,
}

#[derive(Clone, Copy)]
struct EdgeMapRequest<'a> {
    steps: &'a [RecipeDagStep],
    available_ids: &'a alloc::collections::BTreeSet<alloc::string::String>,
}

fn edge_maps(request: EdgeMapRequest<'_>) -> EdgeMaps {
    assert!(request.available_ids.len() <= MAX_DAG_STEPS, "available DAG IDs must stay bounded");
    assert!(request.steps.len() <= MAX_DAG_STEPS, "edge-map input must stay bounded");
    let mut dependent_edges = alloc::collections::BTreeMap::new();
    let mut indegree = alloc::collections::BTreeMap::new();
    for step_id in request.available_ids {
        dependent_edges.entry(step_id.clone()).or_default();
        indegree.entry(step_id.clone()).or_default();
    }
    for step in request.steps {
        add_available_edges(step, request.available_ids, &mut dependent_edges, &mut indegree);
    }
    EdgeMaps {
        dependent_edges,
        indegree,
    }
}

fn add_available_edges(
    step: &RecipeDagStep,
    available_ids: &alloc::collections::BTreeSet<alloc::string::String>,
    dependent_edges: &mut alloc::collections::BTreeMap<alloc::string::String, alloc::vec::Vec<alloc::string::String>>,
    indegree: &mut alloc::collections::BTreeMap<alloc::string::String, usize>,
) {
    assert!(available_ids.len() <= MAX_DAG_STEPS, "available IDs must stay bounded");
    assert!(indegree.len() <= MAX_DAG_STEPS, "indegree map must stay bounded");
    if !available_ids.contains(step.step_id()) {
        return;
    }
    for dependency in step.depends_on() {
        if available_ids.contains(dependency) {
            dependent_edges.entry(dependency.clone()).or_default().push(step.step_id.clone());
            *indegree.entry(step.step_id.clone()).or_default() += 1;
        }
    }
}

fn next_batch(next_batch_index: &mut u32, step_ids: alloc::vec::Vec<alloc::string::String>) -> RecipeDagBatch {
    assert!(!step_ids.is_empty(), "batch must contain steps");
    assert!(step_ids.len() <= MAX_DAG_STEPS, "batch width must stay bounded");
    let batch = RecipeDagBatch {
        index: *next_batch_index,
        step_ids,
    };
    *next_batch_index = next_batch_index.saturating_add(BATCH_INDEX_ADVANCE);
    batch
}

fn blocked_steps(
    steps: &[RecipeDagStep],
    denied_capabilities: &[alloc::string::String],
    denied_validation_gates: &[alloc::string::String],
    denied_subrecipes: &[alloc::string::String],
) -> alloc::vec::Vec<RecipeDagBlockedStep> {
    assert!(steps.len() <= MAX_DAG_STEPS, "blocked DAG input must stay bounded");
    assert!(denied_capabilities.len() <= MAX_DAG_REFS_PER_STEP, "denied capability refs must stay bounded");
    let mut accumulator = BlockAccumulator::with_capacity(steps.len());
    for step in steps {
        if let Some(capability) = first_overlap(step.capability_ceilings(), denied_capabilities) {
            accumulator.push(step.step_id(), RecipeDagBlockReason::CapabilityDenied, capability);
            continue;
        }
        if let Some(gate) = first_overlap(step.validation_gates(), denied_validation_gates) {
            accumulator.push(step.step_id(), RecipeDagBlockReason::ValidationGateDenied, gate);
            continue;
        }
        if let Some(subrecipe) = first_overlap(step.subrecipe_refs(), denied_subrecipes) {
            accumulator.push(step.step_id(), RecipeDagBlockReason::SubrecipeDenied, subrecipe);
        }
    }

    let mut is_changed = true;
    while is_changed {
        is_changed = false;
        for step in steps {
            if accumulator.contains(step.step_id()) {
                continue;
            }
            if let Some(dependency) = step.depends_on().iter().find(|dependency| accumulator.contains(dependency)) {
                accumulator.push(step.step_id(), RecipeDagBlockReason::DependencyBlocked, dependency);
                is_changed = true;
            }
        }
    }
    accumulator.into_blocked()
}

fn first_overlap<'a>(values: &'a [alloc::string::String], denied: &[alloc::string::String]) -> Option<&'a str> {
    values
        .iter()
        .find(|value| denied.iter().any(|denied_value| denied_value == *value))
        .map(alloc::string::String::as_str)
}

struct BlockAccumulator {
    blocked: alloc::vec::Vec<RecipeDagBlockedStep>,
    blocked_ids: alloc::collections::BTreeSet<alloc::string::String>,
}

impl BlockAccumulator {
    fn with_capacity(step_count: usize) -> Self {
        assert!(step_count <= MAX_DAG_STEPS, "blocked accumulator must stay bounded");
        Self {
            blocked: alloc::vec::Vec::with_capacity(step_count),
            blocked_ids: alloc::collections::BTreeSet::new(),
        }
    }

    fn push(&mut self, step_id: &str, reason: RecipeDagBlockReason, detail: &str) {
        if self.blocked_ids.insert(step_id.to_string()) {
            self.blocked.push(RecipeDagBlockedStep {
                step_id: step_id.to_string(),
                reason,
                detail: detail.to_string(),
            });
        }
    }

    fn contains(&self, step_id: &str) -> bool {
        self.blocked_ids.contains(step_id)
    }

    fn into_blocked(self) -> alloc::vec::Vec<RecipeDagBlockedStep> {
        self.blocked
    }
}

fn build_receipt(
    workflow_id: &str,
    steps: &[RecipeDagStep],
    blocked_steps: &[RecipeDagBlockedStep],
) -> Result<RecipeDagReceipt, super::RecipeWorkflowError> {
    assert!(!workflow_id.is_empty(), "receipt workflow ID must be present");
    assert!(steps.len() <= MAX_DAG_STEPS, "receipt step count must stay bounded");
    let step_count = bounded_count(ItemLimit {
        field_name: STEP_ID_FIELD,
        actual_items: steps.len(),
        max_items: MAX_DAG_STEPS,
    })?;
    let edge_count_items: usize = steps.iter().map(|step| step.depends_on().len()).sum();
    let max_edge_items = MAX_DAG_STEPS.saturating_mul(MAX_DAG_EDGES_PER_STEP);
    let edge_count = bounded_count(ItemLimit {
        field_name: DEPENDENCY_FIELD,
        actual_items: edge_count_items,
        max_items: max_edge_items,
    })?;
    let issue_codes = issue_codes(blocked_steps);
    let mut canonical = alloc::string::String::new();
    canonical.push_str(RECIPE_DAG_SCHEMA);
    canonical.push('|');
    canonical.push_str(workflow_id);
    canonical.push('|');
    canonical.push_str(step_count.to_string().as_str());
    canonical.push('|');
    canonical.push_str(edge_count.to_string().as_str());
    for step in steps {
        canonical.push('|');
        canonical.push_str(step.step_id());
        canonical.push(':');
        canonical.push_str(step.kind().as_str());
    }
    for code in &issue_codes {
        canonical.push('|');
        canonical.push_str(code);
    }
    let identity_hash = alloc::format!("{}", blake3::hash(canonical.as_bytes()).to_hex());
    Ok(RecipeDagReceipt {
        schema_label: RECIPE_DAG_SCHEMA,
        identity_hash,
        step_count,
        edge_count,
        issue_codes,
        redaction_markers: alloc::vec![REDACTION_MARKER.to_string()],
    })
}

fn bounded_count(limit: ItemLimit) -> Result<u32, super::RecipeWorkflowError> {
    ensure_limit(limit)?;
    match u32::try_from(limit.actual_items) {
        Ok(count) => Ok(count),
        Err(_overflow) => Err(super::RecipeWorkflowError::DagLimitExceeded {
            field: limit.field_name.as_str(),
            max_items: limit.max_items,
            actual_items: limit.actual_items,
        }),
    }
}

fn issue_codes(blocked_steps: &[RecipeDagBlockedStep]) -> alloc::vec::Vec<alloc::string::String> {
    let mut codes = alloc::collections::BTreeSet::new();
    for blocked in blocked_steps {
        codes.insert(blocked.reason().issue_code().to_string());
    }
    if codes.is_empty() {
        return alloc::vec![RECEIPT_ISSUE_NONE.to_string()];
    }
    codes.into_iter().collect()
}
