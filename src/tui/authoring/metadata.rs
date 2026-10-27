//! Typed descriptor metadata shared by editor rows and the portable capability matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Owner {
    Workflow,
    Node,
    Graph,
    Credential,
    Runner,
    Trigger,
    Operation,
    Identity,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Text,
    Boolean,
    UnsignedInteger,
    SignedInteger,
    Timestamp,
    Choice,
    Predicate,
    Path,
    TypedRow,
    AdmittedPlan,
    AllocatedIdentity,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Required,
    Optional,
    Repeated,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spec {
    pub owner: Owner,
    pub value_kind: ValueKind,
    pub shape: Shape,
    pub parser: &'static str,
    pub validator: &'static str,
    pub mutation_target: &'static str,
}

pub fn spec(id: &str) -> Spec {
    let (owner, parser, validator, target) = if id.starts_with("config.operation.") || id.starts_with("operation.") {
        (
            Owner::Operation,
            "operation::Builder::apply",
            "operation::planning::validate_plan_admission",
            "Workflow.configs / admitted InvocationPlan",
        )
    } else if id.starts_with("config.") {
        (
            Owner::Node,
            "node::apply",
            "NodeKindProvider::validate_config + aggregate admission",
            "Workflow.configs + reconciled graph ports",
        )
    } else if id.starts_with("workflow.") {
        (
            Owner::Workflow,
            "workflow::apply",
            "workflow::validate + metadata parser",
            "Workflow.meta + trigger workflow references",
        )
    } else if id.starts_with("graph.") {
        (Owner::Graph, "workflow::apply", "workflow::validate + checked graph admission", "Workflow.graph")
    } else if id.starts_with("credential.") {
        (
            Owner::Credential,
            "workflow::apply",
            "credentials::validate_secret_ref",
            "Workflow.credentials / opaque references only",
        )
    } else if id.starts_with("runner.") {
        (
            Owner::Runner,
            "workflow::apply",
            "engine::runner::runtime_bindings + workflow::validate",
            "Workflow.runner",
        )
    } else if id.starts_with("trigger.") {
        (
            Owner::Trigger,
            "workflow::apply",
            "trigger parsers + workflow::validate",
            "Workflow.triggers / no scheduler or listener",
        )
    } else {
        (
            Owner::Identity,
            "checked workflow allocation",
            "Workflow::validate_aggregate_ownership",
            "Workflow.graph + config owner identity",
        )
    };
    let leaf = id.rsplit('.').next().unwrap_or(id);
    let repeated = id.contains("[]")
        || id.contains("{}")
        || id.contains(".env.")
        || id.contains(".headers.")
        || id.contains(".branches.")
        || id.contains(".input_mappings.")
        || id.contains(".output_mappings.")
        || id.starts_with("graph.port.")
        || id.starts_with("graph.edge.")
        || id.starts_with("credential.env.")
        || id.starts_with("trigger.");
    let shape = if repeated {
        Shape::Repeated
    } else if id.starts_with("runner.")
        || [
            "cwd",
            "body",
            "content",
            "prompt_file",
            "model_hint",
            "session",
            "delegated_policy_reference",
            "tool_policy",
        ]
        .contains(&leaf)
    {
        Shape::Optional
    } else {
        Shape::Required
    };
    let value_kind = if owner == Owner::Operation {
        ValueKind::AdmittedPlan
    } else if owner == Owner::Identity {
        ValueKind::AllocatedIdentity
    } else if [
        "pipe_stdin",
        "overwrite",
        "enabled",
        "required",
        "identity_bound",
        "prohibit_self_approval",
        "approval_must_be_fresh_at_resume",
    ]
    .contains(&leaf)
    {
        ValueKind::Boolean
    } else if ["x", "y", "schedule_next_run_epoch_ms"].contains(&leaf) {
        ValueKind::SignedInteger
    } else if [
        "timeout_secs",
        "max_bytes",
        "retry_limit",
        "decision_ttl_millis",
        "resume_ttl_millis",
        "schedule_interval_secs",
        "move",
    ]
    .contains(&leaf)
        || id == "graph.port.id"
    {
        ValueKind::UnsignedInteger
    } else if ["created", "modified"].contains(&leaf) {
        ValueKind::Timestamp
    } else if leaf == "predicate" {
        ValueKind::Predicate
    } else if [
        "path",
        "cwd",
        "prompt_file",
        "catalog_path",
        "instances_path",
        "executable_path",
    ]
    .contains(&leaf)
    {
        ValueKind::Path
    } else if [
        "method",
        "operation",
        "target",
        "output_mode",
        "policy",
        "direction",
        "provider",
        "effect_policy",
        "cache_policy",
        "retry_policy",
        "artifact_policy",
        "kind",
        "source_kind",
        "overlap_policy",
        "idempotency_policy",
        "auth_kind",
        "clock_source",
        "on_rejection",
        "on_expiry",
        "schedule_missed_run_policy",
    ]
    .contains(&leaf)
    {
        ValueKind::Choice
    } else if repeated {
        ValueKind::TypedRow
    } else {
        ValueKind::Text
    };
    Spec {
        owner,
        value_kind,
        shape,
        parser,
        validator,
        mutation_target: target,
    }
}
