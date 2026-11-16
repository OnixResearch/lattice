#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowRevisionRef {
    pub workflow_name: String,
    pub content_hash: crate::orchestration::identity::WorkflowRevision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionPolicy {
    RunSucceeded,
    AwaitProposal,
}

// r[impl lattice.agent_orchestration.observed_failure_retry]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetryPolicy {
    pub max_observed_failure_retries: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityPolicy {
    pub approvers: std::collections::BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerKind {
    Manual,
    Schedule,
    Webhook,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TriggerBinding {
    pub trigger_id: crate::orchestration::identity::TriggerId,
    pub kind: TriggerKind,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedactionPolicy {
    pub retain_safe_summaries: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub schema_version: u32,
    pub id: crate::orchestration::identity::DefinitionId,
    pub objective: String,
    pub workflow: WorkflowRevisionRef,
    pub completion: CompletionPolicy,
    #[serde(default = "absent_retry_policy", skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetryPolicy>,
    pub budgets: super::super::BudgetPolicy,
    pub authority: AuthorityPolicy,
    pub triggers: Vec<TriggerBinding>,
    pub redaction: RedactionPolicy,
}

pub fn current_schema_version() -> u32 {
    super::super::CURRENT_SCHEMA_VERSION
}

// r[impl lattice.agent_orchestration.model]
pub fn validate_definition(
    definition: &Definition,
    actual_workflow_revision: Option<&WorkflowRevisionRef>,
) -> Vec<super::super::Diagnostic> {
    let mut diagnostics = Vec::new();
    if let Some(diagnostic) = schema_version_diagnostic(definition.schema_version) {
        diagnostics.extend([diagnostic]);
    }
    if let Some(diagnostic) = empty_objective_diagnostic(&definition.objective) {
        diagnostics.extend([diagnostic]);
    }
    validate_workflow_ref(&definition.workflow, &mut diagnostics);
    validate_actual_workflow_revision(definition, actual_workflow_revision, &mut diagnostics);
    validate_authority_policy(&definition.authority, &mut diagnostics);
    validate_trigger_bindings(&definition.triggers, &mut diagnostics);
    validate_budget_policy(&definition.budgets, &mut diagnostics);
    if let Some(diagnostic) = retry_policy_diagnostic(definition.retry) {
        diagnostics.extend([diagnostic]);
    }
    debug_assert!(!diagnostics.is_empty() || definition.schema_version == super::super::CURRENT_SCHEMA_VERSION);
    debug_assert!(!diagnostics.is_empty() || !definition.authority.approvers.is_empty());
    diagnostics
}

pub(super) fn schema_version_diagnostic(schema_version: u32) -> Option<super::super::Diagnostic> {
    (schema_version != super::super::CURRENT_SCHEMA_VERSION).then(|| {
        super::failures::diagnostic(
            super::super::DiagnosticCode::InvalidSchemaVersion,
            format!(
                "orchestration schema version {schema_version} is unsupported; expected {}",
                super::super::CURRENT_SCHEMA_VERSION,
            ),
        )
    })
}

pub(super) fn empty_objective_diagnostic(objective: &str) -> Option<super::super::Diagnostic> {
    objective.trim().is_empty().then(|| {
        super::failures::diagnostic(
            super::super::DiagnosticCode::EmptyObjective,
            "orchestration objective cannot be empty",
        )
    })
}

pub(in crate::orchestration::core) fn validate_workflow_ref(
    reference: &WorkflowRevisionRef,
    diagnostics: &mut impl Extend<super::super::Diagnostic>,
) {
    if reference.workflow_name.trim().is_empty() {
        diagnostics.extend([super::failures::diagnostic(
            super::super::DiagnosticCode::EmptyWorkflowName,
            "workflow revision name cannot be empty",
        )]);
    }
}

pub(super) fn validate_actual_workflow_revision(
    definition: &Definition,
    actual: Option<&WorkflowRevisionRef>,
    diagnostics: &mut impl Extend<super::super::Diagnostic>,
) {
    let Some(actual) = actual else {
        diagnostics.extend([super::failures::diagnostic(
            super::super::DiagnosticCode::WorkflowRevisionMismatch,
            "referenced workflow revision is unavailable",
        )]);
        return;
    };
    if actual != &definition.workflow {
        diagnostics.extend([super::failures::diagnostic(
            super::super::DiagnosticCode::WorkflowRevisionMismatch,
            "referenced workflow content does not match the immutable workflow revision",
        )]);
    }
}

pub(super) fn validate_authority_policy(
    policy: &AuthorityPolicy,
    diagnostics: &mut impl Extend<super::super::Diagnostic>,
) {
    if policy.approvers.is_empty() {
        diagnostics.extend([super::failures::diagnostic(
            super::super::DiagnosticCode::MissingApprover,
            "authority policy must declare at least one approver",
        )]);
    }
    if policy.approvers.iter().any(|approver| approver.trim().is_empty()) {
        diagnostics.extend([super::failures::diagnostic(
            super::super::DiagnosticCode::DuplicateApprover,
            "authority policy approver identities must be non-empty",
        )]);
    }
}

pub(super) fn validate_trigger_bindings(
    bindings: &[TriggerBinding],
    diagnostics: &mut impl Extend<super::super::Diagnostic>,
) {
    let mut ids = std::collections::BTreeSet::new();
    for binding in bindings {
        if !ids.insert(binding.trigger_id.clone()) {
            diagnostics.extend([super::failures::diagnostic(
                super::super::DiagnosticCode::DuplicateTriggerId,
                format!("orchestration trigger '{}' is declared more than once", binding.trigger_id),
            )]);
        }
    }
}

// r[impl lattice.agent_orchestration.observed_failure_retry]
pub(super) fn retry_policy_diagnostic(policy: Option<RetryPolicy>) -> Option<super::super::Diagnostic> {
    match policy {
        Some(policy) if policy.max_observed_failure_retries == 0 => Some(super::failures::diagnostic(
            super::super::DiagnosticCode::RetryDisabled,
            "enabled retry policy must allow at least one observed-failure retry",
        )),
        Some(_) | None => None,
    }
}

fn validate_budget_policy(policy: &super::super::BudgetPolicy, output: &mut impl Extend<super::super::Diagnostic>) {
    for (is_enforced, limit, charge, label) in budget_policy_dimensions(policy) {
        if is_enforced && charge > limit {
            output.extend([super::failures::diagnostic(
                super::super::DiagnosticCode::BudgetExceeded,
                format!("per-round {label} upper bound exceeds the orchestration limit"),
            )]);
        }
    }
}

fn budget_policy_dimensions(
    policy: &super::super::BudgetPolicy,
) -> impl IntoIterator<Item = (bool, u64, u64, &'static str)> {
    let active = policy.enforced;
    let cap = policy.limits;
    let bound = policy.per_round_upper_bound;
    [
        (active.rounds, cap.rounds, bound.rounds, "rounds"),
        (active.agent_invocations, cap.agent_invocations, bound.agent_invocations, "agent invocations"),
        (active.tokens, cap.tokens, bound.tokens, "tokens"),
        (
            active.monetary_microunits,
            cap.monetary_microunits,
            bound.monetary_microunits,
            "monetary microunits",
        ),
        (active.duration_millis, cap.duration_millis, bound.duration_millis, "duration milliseconds"),
    ]
}

const fn absent_retry_policy() -> Option<RetryPolicy> {
    None
}
