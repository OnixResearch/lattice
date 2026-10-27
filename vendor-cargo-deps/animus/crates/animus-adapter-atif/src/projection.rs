#![allow(missing_docs)]

use alloc::{collections::BTreeMap, string::String, vec, vec::Vec};

use crate::{
    ATIF_SCHEMA, Agent, AtifDiagnostic, AtifField, AtifIssueClass, AtifLocation, AtifObject,
    FieldDisposition, Message, Metrics, Observation, ObservationResult, Step, StepSource, ToolCall,
    Trajectory, ValidatedAtifPolicy, ValidatedTrajectory, hash_identity, validate_trajectory,
};

/// Explicit semantic loss class.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AtifLossClass {
    Reasoning,
    ToolArguments,
    ToolResults,
    ImageReference,
    TokenIds,
    Logprobs,
    Notes,
    ProviderModel,
    Extensions,
    CopiedContext,
    ExternalReference,
    UnsupportedField,
    UnavailableFact,
}

/// Safe disposition class reported independently from field content.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AtifDispositionClass {
    Omission,
    Redaction,
    HashRetention,
    Unresolved,
}

/// Projection status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AtifProjectionStatus {
    CompleteSubset,
    LossReported,
}

/// One neutral ledger draft.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtifLedgerDraft {
    pub entry_identity: String,
    pub parent_identity: Option<String>,
    pub body_class: &'static str,
    pub step_id: u32,
}

/// One deterministic batch draft.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtifBatchDraft {
    pub sequence: u32,
    pub step_identity: String,
}

/// One subagent-lineage draft.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtifLineageDraft {
    pub parent_identity: String,
    pub child_identity: String,
    pub step_id: u32,
}

/// Safe metrics draft.
#[derive(Clone, Debug, PartialEq)]
pub struct AtifMetricDraft {
    pub step_id: u32,
    pub prompt_tokens: Option<u64>,
    pub completion_tokens: Option<u64>,
    pub cached_tokens: Option<u64>,
    pub cost_usd: Option<f64>,
}

/// Dataset role draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AtifDatasetRole {
    System,
    User,
    Agent,
    ToolResult,
}

/// Dataset-owner draft. This value grants no promotion authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtifDatasetDraft {
    pub step_id: u32,
    pub role: AtifDatasetRole,
    pub content: Option<String>,
    pub eligible: bool,
    pub copied_context: bool,
}

/// Complete import projection.
#[derive(Clone, Debug, PartialEq)]
pub struct AtifImportProjection {
    pub semantic_identity: String,
    pub projection_identity: String,
    pub status: AtifProjectionStatus,
    pub ledger: Vec<AtifLedgerDraft>,
    pub batch: Vec<AtifBatchDraft>,
    pub lineage: Vec<AtifLineageDraft>,
    pub metrics: Vec<AtifMetricDraft>,
    pub hidden_reasoning_redactions: u32,
    pub dataset: Vec<AtifDatasetDraft>,
    pub losses: Vec<AtifLossClass>,
    pub dispositions: Vec<AtifDispositionClass>,
}

/// Admitted Animus tool-call snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimusToolCallSnapshot {
    pub call_identity: String,
    pub function_name: String,
    pub arguments: Option<AtifObject>,
}

/// Admitted Animus observation snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnimusObservationSnapshot {
    pub source_call_identity: Option<String>,
    pub content: Option<String>,
    pub subagent_trajectory_identity: Option<String>,
}

/// Admitted Animus step snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimusStepSnapshot {
    pub source: StepSource,
    pub message: Option<String>,
    pub model_name: Option<String>,
    pub tool_calls: Vec<AnimusToolCallSnapshot>,
    pub observations: Vec<AnimusObservationSnapshot>,
    pub metrics: Option<Metrics>,
    pub copied_context: Option<bool>,
    pub llm_call_count: Option<u32>,
}

/// Admitted Animus trajectory snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimusTrajectorySnapshot {
    pub trajectory_identity: String,
    pub session_identity: Option<String>,
    pub agent_name: String,
    pub agent_version: String,
    pub model_name: Option<String>,
    pub steps: Vec<AnimusStepSnapshot>,
}

/// Exported and revalidated ATIF document.
#[derive(Clone, Debug, PartialEq)]
pub struct AtifExportProjection {
    pub validated: ValidatedTrajectory,
    pub projection_identity: String,
    pub status: AtifProjectionStatus,
    pub losses: Vec<AtifLossClass>,
    pub dispositions: Vec<AtifDispositionClass>,
}

/// Import a validated ATIF value into owner-specific drafts.
///
/// # Errors
/// Returns a bounded diagnostic without partial output.
pub fn import_validated_trajectory(
    validated: &ValidatedTrajectory,
    policy: &ValidatedAtifPolicy,
) -> Result<AtifImportProjection, AtifDiagnostic> {
    if validated.policy_identity() != policy.identity() {
        return Err(diag(AtifIssueClass::Policy, AtifField::Identity));
    }
    let mut output = AtifImportProjection {
        semantic_identity: validated.semantic_identity().into(),
        projection_identity: String::new(),
        status: AtifProjectionStatus::CompleteSubset,
        ledger: Vec::new(),
        batch: Vec::new(),
        lineage: Vec::new(),
        metrics: Vec::new(),
        hidden_reasoning_redactions: 0,
        dataset: Vec::new(),
        losses: Vec::new(),
        dispositions: Vec::new(),
    };
    import_document(
        validated.document(),
        validated.semantic_identity(),
        policy,
        &mut output,
    )?;
    finish_losses(&mut output.losses, policy)?;
    output.status = status(&output.losses);
    output.dispositions = disposition_classes(&output.losses, policy);
    output.projection_identity = projection_identity(
        "animus.harbor-atif-import.v1",
        validated.semantic_identity(),
        policy.identity(),
        &output.losses,
    );
    Ok(output)
}

/// Export an admitted snapshot without inventing unavailable ATIF facts.
///
/// # Errors
/// Returns a bounded diagnostic without partial bytes.
pub fn export_trajectory_snapshot(
    snapshot: &AnimusTrajectorySnapshot,
    policy: &ValidatedAtifPolicy,
) -> Result<AtifExportProjection, AtifDiagnostic> {
    if snapshot.steps.is_empty() {
        return Err(diag(AtifIssueClass::Bound, AtifField::Bound));
    }
    let mut losses = Vec::new();
    let mut steps = Vec::with_capacity(snapshot.steps.len());
    for (index, source) in snapshot.steps.iter().enumerate() {
        steps.push(export_step(source, index, policy, &mut losses)?);
    }
    let document = Trajectory {
        schema_version: ATIF_SCHEMA.into(),
        session_id: snapshot.session_identity.clone(),
        trajectory_id: Some(snapshot.trajectory_identity.clone()),
        agent: Agent {
            name: snapshot.agent_name.clone(),
            version: snapshot.agent_version.clone(),
            model_name: select_string(
                snapshot.model_name.as_ref(),
                policy.value().projection.provider_model,
                AtifLossClass::ProviderModel,
                &mut losses,
            )?,
            tool_definitions: None,
            extra: None,
        },
        steps,
        notes: None,
        final_metrics: None,
        continued_trajectory_ref: None,
        extra: None,
        subagent_trajectories: None,
    };
    let validated = validate_trajectory(document, policy)?;
    finish_losses(&mut losses, policy)?;
    let projection_identity = projection_identity(
        "animus.harbor-atif-export.v1",
        validated.semantic_identity(),
        policy.identity(),
        &losses,
    );
    let dispositions = disposition_classes(&losses, policy);
    Ok(AtifExportProjection {
        validated,
        projection_identity,
        status: status(&losses),
        losses,
        dispositions,
    })
}

fn import_document(
    document: &Trajectory,
    parent: &str,
    policy: &ValidatedAtifPolicy,
    output: &mut AtifImportProjection,
) -> Result<(), AtifDiagnostic> {
    if document.agent.model_name.is_some() {
        apply_disposition(
            policy.value().projection.provider_model,
            AtifLossClass::ProviderModel,
            &mut output.losses,
        )?;
    }
    if document.agent.tool_definitions.is_some() {
        push_loss(&mut output.losses, AtifLossClass::UnsupportedField);
    }
    let mut prior = None;
    for step in &document.steps {
        let id = hash_identity(
            "animus.harbor-atif-step.v1",
            &[parent.as_bytes(), &step.step_id.to_le_bytes()],
        );
        output.ledger.push(AtifLedgerDraft {
            entry_identity: id.clone(),
            parent_identity: prior.clone(),
            body_class: source_label(step.source),
            step_id: step.step_id,
        });
        output.batch.push(AtifBatchDraft {
            sequence: step.step_id,
            step_identity: id.clone(),
        });
        prior = Some(id);
        import_step(step, parent, policy, output)?;
    }
    if document.notes.is_some() {
        apply_disposition(
            policy.value().projection.notes,
            AtifLossClass::Notes,
            &mut output.losses,
        )?;
    }
    if document.extra.is_some() || document.agent.extra.is_some() {
        apply_disposition(
            policy.value().projection.extensions,
            AtifLossClass::Extensions,
            &mut output.losses,
        )?;
    }
    if document.continued_trajectory_ref.is_some() {
        push_loss(&mut output.losses, AtifLossClass::ExternalReference);
    }
    if let Some(metrics) = &document.final_metrics {
        output.metrics.push(AtifMetricDraft {
            step_id: 0,
            prompt_tokens: metrics.total_prompt_tokens,
            completion_tokens: metrics.total_completion_tokens,
            cached_tokens: metrics.total_cached_tokens,
            cost_usd: metrics.total_cost_usd,
        });
        if metrics.total_steps.is_some() {
            push_loss(&mut output.losses, AtifLossClass::UnsupportedField);
        }
        if metrics.extra.is_some() {
            push_loss(&mut output.losses, AtifLossClass::Extensions);
        }
    }
    for child in document
        .subagent_trajectories
        .as_deref()
        .unwrap_or_default()
    {
        let child_id = child
            .trajectory_id
            .as_deref()
            .ok_or_else(|| diag(AtifIssueClass::Reference, AtifField::Reference))?;
        import_document(child, child_id, policy, output)?;
    }
    Ok(())
}

fn import_step(
    step: &Step,
    parent: &str,
    policy: &ValidatedAtifPolicy,
    output: &mut AtifImportProjection,
) -> Result<(), AtifDiagnostic> {
    let p = &policy.value().projection;
    if step.timestamp.is_some() {
        push_loss(&mut output.losses, AtifLossClass::UnsupportedField);
    }
    if step.reasoning_content.is_some() || step.reasoning_effort.is_some() {
        apply_disposition(p.reasoning, AtifLossClass::Reasoning, &mut output.losses)?;
        if p.reasoning != FieldDisposition::Include {
            output.hidden_reasoning_redactions = output
                .hidden_reasoning_redactions
                .checked_add(1)
                .ok_or_else(|| diag(AtifIssueClass::Bound, AtifField::Bound))?;
        }
    }
    if step.model_name.is_some() {
        apply_disposition(
            p.provider_model,
            AtifLossClass::ProviderModel,
            &mut output.losses,
        )?;
    }
    let copied = step.is_copied_context == Some(true);
    if copied {
        push_loss(&mut output.losses, AtifLossClass::CopiedContext);
    }
    output.dataset.push(AtifDatasetDraft {
        step_id: step.step_id,
        role: dataset_role(step.source),
        content: project_message(
            &step.message,
            p.messages,
            AtifLossClass::UnsupportedField,
            &mut output.losses,
        )?,
        eligible: !copied && step.llm_call_count != Some(0),
        copied_context: copied,
    });
    import_tool_calls(step, policy, output)?;
    import_observation(step, parent, policy, output)?;
    import_metrics(step, policy, output)?;
    if step.extra.is_some() {
        push_loss(&mut output.losses, AtifLossClass::Extensions);
    }
    Ok(())
}

fn import_tool_calls(
    step: &Step,
    policy: &ValidatedAtifPolicy,
    output: &mut AtifImportProjection,
) -> Result<(), AtifDiagnostic> {
    for call in step.tool_calls.as_deref().unwrap_or_default() {
        if !call.arguments.is_empty() {
            apply_disposition(
                policy.value().projection.tool_arguments,
                AtifLossClass::ToolArguments,
                &mut output.losses,
            )?;
        }
        if call.extra.is_some() {
            push_loss(&mut output.losses, AtifLossClass::Extensions);
        }
    }
    Ok(())
}

fn import_observation(
    step: &Step,
    parent: &str,
    policy: &ValidatedAtifPolicy,
    output: &mut AtifImportProjection,
) -> Result<(), AtifDiagnostic> {
    let Some(observation) = &step.observation else {
        return Ok(());
    };
    let copied = step.is_copied_context == Some(true);
    for result in &observation.results {
        if let Some(content) = &result.content {
            output.dataset.push(AtifDatasetDraft {
                step_id: step.step_id,
                role: AtifDatasetRole::ToolResult,
                content: project_message(
                    content,
                    policy.value().projection.tool_results,
                    AtifLossClass::ToolResults,
                    &mut output.losses,
                )?,
                eligible: false,
                copied_context: copied,
            });
        }
        for reference in result
            .subagent_trajectory_ref
            .as_deref()
            .unwrap_or_default()
        {
            if let Some(child) = &reference.trajectory_id {
                output.lineage.push(AtifLineageDraft {
                    parent_identity: parent.into(),
                    child_identity: child.clone(),
                    step_id: step.step_id,
                });
            }
            if reference.trajectory_path.is_some() {
                push_loss(&mut output.losses, AtifLossClass::ExternalReference);
            }
            if reference.extra.is_some() {
                push_loss(&mut output.losses, AtifLossClass::Extensions);
            }
        }
        if result.extra.is_some() {
            push_loss(&mut output.losses, AtifLossClass::Extensions);
        }
    }
    Ok(())
}

fn import_metrics(
    step: &Step,
    policy: &ValidatedAtifPolicy,
    output: &mut AtifImportProjection,
) -> Result<(), AtifDiagnostic> {
    let Some(metrics) = &step.metrics else {
        return Ok(());
    };
    output.metrics.push(AtifMetricDraft {
        step_id: step.step_id,
        prompt_tokens: metrics.prompt_tokens,
        completion_tokens: metrics.completion_tokens,
        cached_tokens: metrics.cached_tokens,
        cost_usd: metrics.cost_usd,
    });
    let projection = &policy.value().projection;
    if metrics.prompt_token_ids.is_some() || metrics.completion_token_ids.is_some() {
        apply_disposition(
            projection.token_ids,
            AtifLossClass::TokenIds,
            &mut output.losses,
        )?;
    }
    if metrics.logprobs.is_some() {
        apply_disposition(
            projection.logprobs,
            AtifLossClass::Logprobs,
            &mut output.losses,
        )?;
    }
    if metrics.extra.is_some() {
        push_loss(&mut output.losses, AtifLossClass::Extensions);
    }
    Ok(())
}

fn export_step(
    source: &AnimusStepSnapshot,
    index: usize,
    policy: &ValidatedAtifPolicy,
    losses: &mut Vec<AtifLossClass>,
) -> Result<Step, AtifDiagnostic> {
    let step_id = u32::try_from(index)
        .ok()
        .and_then(|value| value.checked_add(1))
        .ok_or_else(|| diag(AtifIssueClass::Bound, AtifField::Bound))?;
    let message = select_string(
        source.message.as_ref(),
        policy.value().projection.messages,
        AtifLossClass::UnsupportedField,
        losses,
    )?
    .map_or_else(|| Message::Text(String::new()), Message::Text);
    let mut calls = Vec::new();
    for call in &source.tool_calls {
        let arguments = match (&call.arguments, policy.value().projection.tool_arguments) {
            (Some(value), FieldDisposition::Include) => value.clone(),
            (Some(_), FieldDisposition::Reject) => {
                return Err(diag(AtifIssueClass::Authority, AtifField::Projection));
            }
            (Some(_), _) => {
                push_loss(losses, AtifLossClass::ToolArguments);
                AtifObject::new(BTreeMap::new())
            }
            (None, _) => {
                push_loss(losses, AtifLossClass::UnavailableFact);
                AtifObject::new(BTreeMap::new())
            }
        };
        calls.push(ToolCall {
            tool_call_id: call.call_identity.clone(),
            function_name: call.function_name.clone(),
            arguments,
            extra: None,
        });
    }
    let mut results = Vec::new();
    for result in &source.observations {
        results.push(ObservationResult {
            source_call_id: result.source_call_identity.clone(),
            content: select_string(
                result.content.as_ref(),
                policy.value().projection.tool_results,
                AtifLossClass::ToolResults,
                losses,
            )?
            .map(Message::Text),
            subagent_trajectory_ref: result.subagent_trajectory_identity.as_ref().map(|id| {
                vec![crate::SubagentTrajectoryRef {
                    trajectory_id: Some(id.clone()),
                    session_id: None,
                    trajectory_path: None,
                    extra: None,
                }]
            }),
            extra: None,
        });
    }
    let metrics = sanitize_metrics(source.metrics.as_ref(), policy, losses)?;
    Ok(Step {
        step_id,
        timestamp: None,
        source: source.source,
        model_name: select_string(
            source.model_name.as_ref(),
            policy.value().projection.provider_model,
            AtifLossClass::ProviderModel,
            losses,
        )?,
        reasoning_effort: None,
        message,
        reasoning_content: None,
        tool_calls: (!calls.is_empty()).then_some(calls),
        observation: (!results.is_empty()).then_some(Observation { results }),
        metrics,
        is_copied_context: source.copied_context,
        llm_call_count: source.llm_call_count,
        extra: None,
    })
}

fn sanitize_metrics(
    metrics: Option<&Metrics>,
    policy: &ValidatedAtifPolicy,
    losses: &mut Vec<AtifLossClass>,
) -> Result<Option<Metrics>, AtifDiagnostic> {
    let Some(metrics) = metrics else {
        return Ok(None);
    };
    let p = &policy.value().projection;
    if metrics.prompt_token_ids.is_some() || metrics.completion_token_ids.is_some() {
        apply_disposition(p.token_ids, AtifLossClass::TokenIds, losses)?;
    }
    if metrics.logprobs.is_some() {
        apply_disposition(p.logprobs, AtifLossClass::Logprobs, losses)?;
    }
    if metrics.extra.is_some() {
        apply_disposition(p.extensions, AtifLossClass::Extensions, losses)?;
    }
    Ok(Some(Metrics {
        prompt_tokens: metrics.prompt_tokens,
        completion_tokens: metrics.completion_tokens,
        cached_tokens: metrics.cached_tokens,
        cost_usd: metrics.cost_usd,
        prompt_token_ids: include_value(metrics.prompt_token_ids.as_ref(), p.token_ids),
        completion_token_ids: include_value(metrics.completion_token_ids.as_ref(), p.token_ids),
        logprobs: include_value(metrics.logprobs.as_ref(), p.logprobs),
        extra: include_value(metrics.extra.as_ref(), p.extensions),
    }))
}

fn project_message(
    message: &Message,
    disposition: FieldDisposition,
    loss: AtifLossClass,
    losses: &mut Vec<AtifLossClass>,
) -> Result<Option<String>, AtifDiagnostic> {
    let has_image =
        matches!(message, Message::Parts(parts) if parts.iter().any(|p| p.source.is_some()));
    if has_image {
        push_loss(losses, AtifLossClass::ImageReference);
    }
    let text = match message {
        Message::Text(value) => Some(value.clone()),
        Message::Parts(parts) => {
            let values = parts
                .iter()
                .filter_map(|part| part.text.as_ref())
                .cloned()
                .collect::<Vec<_>>();
            (!values.is_empty()).then(|| values.join("\n"))
        }
    };
    select_string(text.as_ref(), disposition, loss, losses)
}

fn select_string(
    value: Option<&String>,
    disposition: FieldDisposition,
    loss: AtifLossClass,
    losses: &mut Vec<AtifLossClass>,
) -> Result<Option<String>, AtifDiagnostic> {
    let Some(value) = value else { return Ok(None) };
    match disposition {
        FieldDisposition::Include => Ok(Some(value.clone())),
        FieldDisposition::Redact => {
            push_loss(losses, loss);
            Ok(Some("[redacted]".into()))
        }
        FieldDisposition::Hash => {
            push_loss(losses, loss);
            Ok(Some(hash_identity(
                "animus.harbor-atif-field.v1",
                &[value.as_bytes()],
            )))
        }
        FieldDisposition::Omit | FieldDisposition::Unresolved => {
            push_loss(losses, loss);
            Ok(None)
        }
        FieldDisposition::Reject => Err(diag(AtifIssueClass::Authority, AtifField::Projection)),
    }
}

fn apply_disposition(
    disposition: FieldDisposition,
    loss: AtifLossClass,
    losses: &mut Vec<AtifLossClass>,
) -> Result<(), AtifDiagnostic> {
    if disposition == FieldDisposition::Reject {
        return Err(diag(AtifIssueClass::Authority, AtifField::Projection));
    }
    if disposition != FieldDisposition::Include {
        push_loss(losses, loss);
    }
    Ok(())
}

fn include_value<T: Clone>(value: Option<&T>, disposition: FieldDisposition) -> Option<T> {
    (disposition == FieldDisposition::Include)
        .then(|| value.cloned())
        .flatten()
}

fn finish_losses(
    losses: &mut Vec<AtifLossClass>,
    policy: &ValidatedAtifPolicy,
) -> Result<(), AtifDiagnostic> {
    losses.sort_unstable();
    losses.dedup();
    if losses.len() > policy.value().bounds.max_loss_classes {
        return Err(diag(AtifIssueClass::Bound, AtifField::Bound));
    }
    Ok(())
}

fn push_loss(losses: &mut Vec<AtifLossClass>, loss: AtifLossClass) {
    if !losses.contains(&loss) {
        losses.push(loss);
    }
}

fn projection_identity(
    domain: &str,
    source: &str,
    policy: &str,
    losses: &[AtifLossClass],
) -> String {
    let loss_text = losses
        .iter()
        .map(|loss| loss_label(*loss))
        .collect::<Vec<_>>()
        .join(":");
    hash_identity(
        domain,
        &[source.as_bytes(), policy.as_bytes(), loss_text.as_bytes()],
    )
}

const fn status(losses: &[AtifLossClass]) -> AtifProjectionStatus {
    if losses.is_empty() {
        AtifProjectionStatus::CompleteSubset
    } else {
        AtifProjectionStatus::LossReported
    }
}

const fn dataset_role(source: StepSource) -> AtifDatasetRole {
    match source {
        StepSource::System => AtifDatasetRole::System,
        StepSource::User => AtifDatasetRole::User,
        StepSource::Agent => AtifDatasetRole::Agent,
    }
}

const fn source_label(source: StepSource) -> &'static str {
    match source {
        StepSource::System => "system_message",
        StepSource::User => "user_message",
        StepSource::Agent => "agent_message",
    }
}

fn disposition_classes(
    losses: &[AtifLossClass],
    policy: &ValidatedAtifPolicy,
) -> Vec<AtifDispositionClass> {
    let projection = &policy.value().projection;
    let mut classes = losses
        .iter()
        .map(|loss| match loss {
            AtifLossClass::Reasoning => disposition_class(projection.reasoning),
            AtifLossClass::ToolArguments => disposition_class(projection.tool_arguments),
            AtifLossClass::ToolResults => disposition_class(projection.tool_results),
            AtifLossClass::TokenIds => disposition_class(projection.token_ids),
            AtifLossClass::Logprobs => disposition_class(projection.logprobs),
            AtifLossClass::Notes => disposition_class(projection.notes),
            AtifLossClass::ProviderModel => disposition_class(projection.provider_model),
            AtifLossClass::Extensions => disposition_class(projection.extensions),
            AtifLossClass::ImageReference | AtifLossClass::ExternalReference => {
                AtifDispositionClass::Unresolved
            }
            AtifLossClass::CopiedContext
            | AtifLossClass::UnsupportedField
            | AtifLossClass::UnavailableFact => AtifDispositionClass::Omission,
        })
        .collect::<Vec<_>>();
    classes.sort_unstable();
    classes.dedup();
    classes
}

const fn disposition_class(disposition: FieldDisposition) -> AtifDispositionClass {
    match disposition {
        FieldDisposition::Redact => AtifDispositionClass::Redaction,
        FieldDisposition::Hash => AtifDispositionClass::HashRetention,
        FieldDisposition::Unresolved => AtifDispositionClass::Unresolved,
        FieldDisposition::Include | FieldDisposition::Omit | FieldDisposition::Reject => {
            AtifDispositionClass::Omission
        }
    }
}

pub(crate) const fn disposition_class_label(value: AtifDispositionClass) -> &'static str {
    match value {
        AtifDispositionClass::Omission => "omission",
        AtifDispositionClass::Redaction => "redaction",
        AtifDispositionClass::HashRetention => "hash_retention",
        AtifDispositionClass::Unresolved => "unresolved",
    }
}

pub(crate) const fn loss_label(loss: AtifLossClass) -> &'static str {
    match loss {
        AtifLossClass::Reasoning => "reasoning",
        AtifLossClass::ToolArguments => "tool_arguments",
        AtifLossClass::ToolResults => "tool_results",
        AtifLossClass::ImageReference => "image_reference",
        AtifLossClass::TokenIds => "token_ids",
        AtifLossClass::Logprobs => "logprobs",
        AtifLossClass::Notes => "notes",
        AtifLossClass::ProviderModel => "provider_model",
        AtifLossClass::Extensions => "extensions",
        AtifLossClass::CopiedContext => "copied_context",
        AtifLossClass::ExternalReference => "external_reference",
        AtifLossClass::UnsupportedField => "unsupported_field",
        AtifLossClass::UnavailableFact => "unavailable_fact",
    }
}

const fn diag(class: AtifIssueClass, field: AtifField) -> AtifDiagnostic {
    AtifDiagnostic::new(class, AtifLocation::Root, field)
}
