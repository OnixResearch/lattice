use alloc::{collections::BTreeSet, format, string::String, vec, vec::Vec};

use crate::{
    ATIF_SCHEMA, AtifDiagnostic, AtifField, AtifIssueClass, AtifLocation, AtifNumber, AtifObject,
    AtifValue, ContentKind, ExternalReferencePolicy, FieldDisposition, FinalMetrics, Message,
    Metrics, Step, StepSource, Trajectory, ValidatedAtifPolicy, hash_identity,
};

const FIRST_STEP_ID: u32 = 1;
const ISO_DATE_TIME_MIN_BYTES: usize = 19;
const ISO_YEAR_END: usize = 4;
const ISO_MONTH_START: usize = 5;
const ISO_MONTH_END: usize = 7;
const ISO_DAY_START: usize = 8;
const ISO_DAY_END: usize = 10;
const ISO_HOUR_START: usize = 11;
const ISO_HOUR_END: usize = 13;
const ISO_MINUTE_START: usize = 14;
const ISO_MINUTE_END: usize = 16;
const ISO_SECOND_START: usize = 17;
const ISO_SECOND_END: usize = 19;
const MONTH_MIN: u32 = 1;
const MONTH_MAX: u32 = 12;
const FEBRUARY: u32 = 2;
const APRIL: u32 = 4;
const JUNE: u32 = 6;
const SEPTEMBER: u32 = 9;
const NOVEMBER: u32 = 11;
const DAY_MIN: u32 = 1;
const DAYS_SHORT_MONTH: u32 = 30;
const DAYS_LONG_MONTH: u32 = 31;
const DAYS_FEBRUARY: u32 = 28;
const DAYS_FEBRUARY_LEAP: u32 = 29;
const LEAP_DIVISOR: u32 = 4;
const CENTURY_DIVISOR: u32 = 100;
const LEAP_CENTURY_DIVISOR: u32 = 400;
const HOUR_MAX: u32 = 23;
const MINUTE_SECOND_MAX: u32 = 59;

/// Bounded structural counts safe for evidence.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AtifCounts {
    /// Trajectory documents.
    pub trajectories: u32,
    /// Steps.
    pub steps: u32,
    /// Content parts.
    pub content_parts: u32,
    /// Tool definitions.
    pub tool_definitions: u32,
    /// Tool calls.
    pub tool_calls: u32,
    /// Observation results.
    pub observation_results: u32,
    /// Subagent references.
    pub subagent_references: u32,
    /// External descriptors.
    pub external_references: u32,
    /// Metrics objects.
    pub metrics: u32,
    /// Extension nodes.
    pub extension_nodes: u32,
    /// Extension string bytes.
    pub extension_bytes: u64,
    /// Copied-context steps.
    pub copied_context_steps: u32,
    /// Deterministic no-LLM dispatch steps.
    pub no_llm_steps: u32,
}

/// External reference classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceDisposition {
    /// Embedded reference resolved by trajectory identity.
    Embedded,
    /// External image descriptor remains unresolved.
    UnresolvedImage,
    /// External continuation descriptor remains unresolved.
    UnresolvedContinuation,
    /// External subagent descriptor remains unresolved.
    UnresolvedSubagent,
}

/// Safe reference fact without a path or URL body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReferenceFact {
    /// Reference class.
    pub disposition: ReferenceDisposition,
    /// One-based step when applicable.
    pub step_id: Option<u32>,
}

/// Fully validated ATIF value.
#[derive(Clone, Debug, PartialEq)]
pub struct ValidatedTrajectory {
    document: Trajectory,
    counts: AtifCounts,
    references: Vec<ReferenceFact>,
    semantic_identity: String,
    policy_identity: String,
}

impl ValidatedTrajectory {
    /// Return the validated typed document.
    #[must_use]
    pub const fn document(&self) -> &Trajectory {
        &self.document
    }

    /// Return safe bounded counts.
    #[must_use]
    pub const fn counts(&self) -> AtifCounts {
        self.counts
    }

    /// Return safe reference classes.
    #[must_use]
    pub fn references(&self) -> &[ReferenceFact] {
        &self.references
    }

    /// Return the semantic typed identity.
    #[must_use]
    pub fn semantic_identity(&self) -> &str {
        self.semantic_identity.as_str()
    }

    /// Return the exact policy identity.
    #[must_use]
    pub fn policy_identity(&self) -> &str {
        self.policy_identity.as_str()
    }
}

/// One node in a caller-supplied resolved reference graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolvedTrajectoryNode<'a> {
    /// Exact node identity.
    pub trajectory_id: &'a str,
    /// Exact outgoing embedded reference identities.
    pub references: &'a [&'a str],
}

/// Validate a typed ATIF document with pure deterministic logic.
///
/// # Errors
/// Returns one bounded issue before any projection or receipt is produced.
pub fn validate_trajectory(
    document: Trajectory,
    policy: &ValidatedAtifPolicy,
) -> Result<ValidatedTrajectory, AtifDiagnostic> {
    let mut validator = Validator::new(policy);
    validator.validate_document(&document, 1, 0, &mut Vec::new())?;
    let material = semantic_material(&document);
    let parts = material.iter().map(String::as_bytes).collect::<Vec<_>>();
    let semantic_identity = hash_identity("animus.harbor-atif-semantic.v1", &parts);
    Ok(ValidatedTrajectory {
        document,
        counts: validator.counts,
        references: validator.references,
        semantic_identity,
        policy_identity: policy.identity().into(),
    })
}

/// Validate a caller-supplied resolved graph without resolving any descriptors.
///
/// # Errors
/// Returns identity, reference, bound, or cycle diagnostics.
pub fn validate_resolved_graph(
    nodes: &[ResolvedTrajectoryNode<'_>],
    policy: &ValidatedAtifPolicy,
) -> Result<(), AtifDiagnostic> {
    let bounds = &policy.value().bounds;
    if nodes.is_empty() || nodes.len() > bounds.max_trajectories {
        return Err(diagnostic(
            AtifIssueClass::Bound,
            AtifLocation::Graph,
            AtifField::Bound,
        ));
    }
    let mut identities = BTreeSet::new();
    for node in nodes {
        if !valid_identity(node.trajectory_id, bounds.max_string_bytes)
            || !identities.insert(node.trajectory_id)
        {
            return Err(diagnostic(
                AtifIssueClass::Identity,
                AtifLocation::Graph,
                AtifField::Identity,
            ));
        }
    }
    for node in nodes {
        let targets = node.references.iter().copied().collect::<BTreeSet<_>>();
        if node.references.len() > bounds.max_subagent_refs
            || targets.len() != node.references.len()
            || node
                .references
                .iter()
                .any(|target| !identities.contains(target))
        {
            return Err(diagnostic(
                AtifIssueClass::Reference,
                AtifLocation::Graph,
                AtifField::Reference,
            ));
        }
    }
    for node in nodes {
        let mut path = Vec::new();
        visit_graph(node.trajectory_id, nodes, bounds.max_depth, &mut path)?;
    }
    Ok(())
}

fn visit_graph(
    current: &str,
    nodes: &[ResolvedTrajectoryNode<'_>],
    max_depth: usize,
    path: &mut Vec<String>,
) -> Result<(), AtifDiagnostic> {
    if path.len() >= max_depth {
        return Err(diagnostic(
            AtifIssueClass::Bound,
            AtifLocation::Graph,
            AtifField::Bound,
        ));
    }
    if path.iter().any(|value| value == current) {
        return Err(diagnostic(
            AtifIssueClass::Cycle,
            AtifLocation::Graph,
            AtifField::Reference,
        ));
    }
    path.push(current.into());
    let node = nodes
        .iter()
        .find(|node| node.trajectory_id == current)
        .ok_or_else(|| {
            diagnostic(
                AtifIssueClass::Reference,
                AtifLocation::Graph,
                AtifField::Reference,
            )
        })?;
    for target in node.references {
        visit_graph(target, nodes, max_depth, path)?;
    }
    let removed = path.pop();
    debug_assert!(removed.is_some());
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum JsonObjectClass {
    Payload,
    Extension,
}

struct Validator<'a> {
    policy: &'a ValidatedAtifPolicy,
    counts: AtifCounts,
    references: Vec<ReferenceFact>,
    json_nodes: usize,
    extension_nodes: usize,
    extension_bytes: usize,
}

impl<'a> Validator<'a> {
    fn new(policy: &'a ValidatedAtifPolicy) -> Self {
        Self {
            policy,
            counts: AtifCounts::default(),
            references: Vec::new(),
            json_nodes: 0,
            extension_nodes: 0,
            extension_bytes: 0,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn validate_document(
        &mut self,
        document: &Trajectory,
        depth: usize,
        sibling_index: usize,
        ancestors: &mut Vec<String>,
    ) -> Result<(), AtifDiagnostic> {
        let bounds = &self.policy.value().bounds;
        if depth > bounds.max_depth {
            return Err(self.at(
                AtifIssueClass::Bound,
                AtifLocation::Subagent {
                    depth: bounded_u32(depth),
                    index: bounded_u32(sibling_index),
                },
                AtifField::Bound,
            ));
        }
        add_count(&mut self.counts.trajectories, 1)?;
        if usize::try_from(self.counts.trajectories).unwrap_or(usize::MAX) > bounds.max_trajectories
        {
            return Err(self.at(AtifIssueClass::Bound, AtifLocation::Root, AtifField::Bound));
        }
        if document.schema_version != ATIF_SCHEMA {
            return Err(self.at(
                AtifIssueClass::Version,
                AtifLocation::Root,
                AtifField::Schema,
            ));
        }
        self.validate_root_identities(document, depth)?;
        self.validate_agent(document)?;
        if document.steps.is_empty() || document.steps.len() > bounds.max_steps {
            return Err(self.at(AtifIssueClass::Bound, AtifLocation::Root, AtifField::Bound));
        }
        let embedded_ids = embedded_id_set(document, bounds.max_string_bytes)?;
        for (index, step) in document.steps.iter().enumerate() {
            self.validate_step(step, index, &embedded_ids)?;
        }
        if let Some(final_metrics) = &document.final_metrics {
            self.validate_final_metrics(
                final_metrics,
                document.steps.len(),
                document.notes.is_some(),
            )?;
        }
        if let Some(notes) = &document.notes {
            self.validate_string(notes, AtifLocation::Root, AtifField::Content)?;
        }
        if document.continued_trajectory_ref.is_some() {
            self.classify_external(ReferenceDisposition::UnresolvedContinuation, None)?;
        }
        self.validate_extra(document.extra.as_ref())?;
        if let Some(identifier) = &document.trajectory_id {
            if ancestors.iter().any(|ancestor| ancestor == identifier) {
                return Err(self.at(
                    AtifIssueClass::Cycle,
                    AtifLocation::Root,
                    AtifField::Reference,
                ));
            }
            ancestors.push(identifier.clone());
        }
        if let Some(subagents) = &document.subagent_trajectories {
            for (index, subagent) in subagents.iter().enumerate() {
                self.validate_document(subagent, depth.saturating_add(1), index, ancestors)?;
            }
        }
        if document.trajectory_id.is_some() {
            let removed = ancestors.pop();
            debug_assert!(removed.is_some());
        }
        Ok(())
    }

    fn validate_root_identities(
        &self,
        document: &Trajectory,
        depth: usize,
    ) -> Result<(), AtifDiagnostic> {
        let max = self.policy.value().bounds.max_string_bytes;
        if document
            .session_id
            .as_ref()
            .is_some_and(|value| !valid_identity(value, max))
            || document
                .trajectory_id
                .as_ref()
                .is_some_and(|value| !valid_identity(value, max))
            || (depth > 1 && document.trajectory_id.is_none())
        {
            return Err(self.at(
                AtifIssueClass::Identity,
                AtifLocation::Root,
                AtifField::Identity,
            ));
        }
        Ok(())
    }

    fn validate_agent(&mut self, document: &Trajectory) -> Result<(), AtifDiagnostic> {
        let max = self.policy.value().bounds.max_string_bytes;
        if !valid_identity(&document.agent.name, max)
            || !valid_identity(&document.agent.version, max)
            || document
                .agent
                .model_name
                .as_ref()
                .is_some_and(|value| !valid_string(value, max))
        {
            return Err(self.at(
                AtifIssueClass::Identity,
                AtifLocation::Agent,
                AtifField::Identity,
            ));
        }
        if let Some(definitions) = &document.agent.tool_definitions {
            if definitions.len() > self.policy.value().bounds.max_tool_definitions {
                return Err(self.at(AtifIssueClass::Bound, AtifLocation::Agent, AtifField::Bound));
            }
            add_count(&mut self.counts.tool_definitions, definitions.len())?;
            for definition in definitions {
                self.validate_json_object(definition, 1, JsonObjectClass::Payload)?;
            }
        }
        self.validate_extra(document.agent.extra.as_ref())
    }

    fn validate_step(
        &mut self,
        step: &Step,
        index: usize,
        embedded_ids: &BTreeSet<&str>,
    ) -> Result<(), AtifDiagnostic> {
        let expected = u32::try_from(index)
            .ok()
            .and_then(|value| value.checked_add(FIRST_STEP_ID))
            .ok_or_else(|| self.at(AtifIssueClass::Bound, AtifLocation::Root, AtifField::Bound))?;
        let location = AtifLocation::Step(expected);
        if step.step_id != expected {
            return Err(self.at(AtifIssueClass::StepSequence, location, AtifField::StepId));
        }
        add_count(&mut self.counts.steps, 1)?;
        if usize::try_from(self.counts.steps).unwrap_or(usize::MAX)
            > self.policy.value().bounds.max_aggregate_steps
        {
            return Err(self.at(AtifIssueClass::Bound, location, AtifField::Bound));
        }
        if step
            .timestamp
            .as_ref()
            .is_some_and(|value| !valid_timestamp(value))
        {
            return Err(self.at(AtifIssueClass::Timestamp, location, AtifField::Timestamp));
        }
        self.validate_source_fields(step, location)?;
        self.validate_message(&step.message, step.step_id)?;
        self.validate_optional_step_strings(step, location)?;
        self.validate_tool_observation(step, embedded_ids)?;
        if let Some(metrics) = &step.metrics {
            self.validate_metrics(metrics, step.step_id)?;
        }
        if step.llm_call_count == Some(0)
            && step.source == StepSource::Agent
            && (step.metrics.is_some() || step.reasoning_content.is_some())
        {
            return Err(self.at(AtifIssueClass::NoLlm, location, AtifField::LlmCallCount));
        }
        if step.llm_call_count == Some(0) {
            add_count(&mut self.counts.no_llm_steps, 1)?;
        }
        if step.is_copied_context == Some(true) {
            add_count(&mut self.counts.copied_context_steps, 1)?;
        }
        self.validate_extra(step.extra.as_ref())
    }

    fn validate_source_fields(
        &self,
        step: &Step,
        location: AtifLocation,
    ) -> Result<(), AtifDiagnostic> {
        if step.source != StepSource::Agent
            && (step.model_name.is_some()
                || step.reasoning_effort.is_some()
                || step.reasoning_content.is_some()
                || step.tool_calls.is_some()
                || step.metrics.is_some())
        {
            return Err(self.at(AtifIssueClass::SourceField, location, AtifField::Source));
        }
        if step.reasoning_effort.as_ref().is_some_and(
            |effort| matches!(effort, crate::ReasoningEffort::Number(value) if !value.is_finite()),
        ) {
            return Err(self.at(AtifIssueClass::Metric, location, AtifField::Metrics));
        }
        Ok(())
    }

    fn validate_optional_step_strings(
        &self,
        step: &Step,
        location: AtifLocation,
    ) -> Result<(), AtifDiagnostic> {
        for value in [step.model_name.as_ref(), step.reasoning_content.as_ref()]
            .into_iter()
            .flatten()
        {
            self.validate_string(value, location, AtifField::Content)?;
        }
        if let Some(crate::ReasoningEffort::Text(value)) = &step.reasoning_effort {
            self.validate_string(value, location, AtifField::Content)?;
        }
        Ok(())
    }

    fn validate_message(&mut self, message: &Message, step_id: u32) -> Result<(), AtifDiagnostic> {
        match message {
            Message::Text(value) => {
                self.validate_string(value, AtifLocation::Step(step_id), AtifField::Content)
            }
            Message::Parts(parts) => {
                if parts.is_empty() || parts.len() > self.policy.value().bounds.max_content_parts {
                    return Err(self.at(
                        AtifIssueClass::Bound,
                        AtifLocation::Step(step_id),
                        AtifField::Bound,
                    ));
                }
                add_count(&mut self.counts.content_parts, parts.len())?;
                for (index, part) in parts.iter().enumerate() {
                    let location = AtifLocation::Content {
                        step: step_id,
                        part: bounded_u32(index),
                    };
                    match part.kind {
                        ContentKind::Text if part.text.is_some() && part.source.is_none() => {
                            self.validate_string(
                                part.text.as_deref().unwrap_or_default(),
                                location,
                                AtifField::Content,
                            )?;
                        }
                        ContentKind::Image if part.source.is_some() && part.text.is_none() => {
                            let source = part.source.as_ref().expect("guarded image source");
                            self.validate_string(&source.path, location, AtifField::Reference)?;
                            self.classify_external(
                                ReferenceDisposition::UnresolvedImage,
                                Some(step_id),
                            )?;
                        }
                        _ => {
                            return Err(self.at(
                                AtifIssueClass::Content,
                                location,
                                AtifField::Content,
                            ));
                        }
                    }
                }
                Ok(())
            }
        }
    }

    fn validate_tool_observation(
        &mut self,
        step: &Step,
        embedded_ids: &BTreeSet<&str>,
    ) -> Result<(), AtifDiagnostic> {
        let calls = step.tool_calls.as_deref().unwrap_or_default();
        if calls.len() > self.policy.value().bounds.max_tool_calls {
            return Err(self.at(
                AtifIssueClass::Bound,
                AtifLocation::Step(step.step_id),
                AtifField::Bound,
            ));
        }
        add_count(&mut self.counts.tool_calls, calls.len())?;
        let mut call_ids = BTreeSet::new();
        for (index, call) in calls.iter().enumerate() {
            let location = AtifLocation::ToolCall {
                step: step.step_id,
                call: bounded_u32(index),
            };
            if !valid_identity(
                &call.tool_call_id,
                self.policy.value().bounds.max_string_bytes,
            ) || !valid_identity(
                &call.function_name,
                self.policy.value().bounds.max_string_bytes,
            ) || !call_ids.insert(call.tool_call_id.as_str())
            {
                return Err(self.at(AtifIssueClass::ToolCorrelation, location, AtifField::Tool));
            }
            self.validate_json_object(&call.arguments, 1, JsonObjectClass::Payload)?;
            self.validate_extra(call.extra.as_ref())?;
        }
        if let Some(observation) = &step.observation {
            if observation.results.len() > self.policy.value().bounds.max_observation_results {
                return Err(self.at(
                    AtifIssueClass::Bound,
                    AtifLocation::Step(step.step_id),
                    AtifField::Bound,
                ));
            }
            add_count(
                &mut self.counts.observation_results,
                observation.results.len(),
            )?;
            for (index, result) in observation.results.iter().enumerate() {
                let location = AtifLocation::Observation {
                    step: step.step_id,
                    result: bounded_u32(index),
                };
                if result
                    .source_call_id
                    .as_ref()
                    .is_some_and(|identity| !call_ids.contains(identity.as_str()))
                {
                    return Err(self.at(
                        AtifIssueClass::ToolCorrelation,
                        location,
                        AtifField::Observation,
                    ));
                }
                if let Some(content) = &result.content {
                    self.validate_message(content, step.step_id)?;
                }
                self.validate_subagent_refs(
                    result
                        .subagent_trajectory_ref
                        .as_deref()
                        .unwrap_or_default(),
                    embedded_ids,
                    step.step_id,
                )?;
                self.validate_extra(result.extra.as_ref())?;
            }
        }
        Ok(())
    }

    fn validate_subagent_refs(
        &mut self,
        references: &[crate::SubagentTrajectoryRef],
        embedded_ids: &BTreeSet<&str>,
        step_id: u32,
    ) -> Result<(), AtifDiagnostic> {
        if references.len() > self.policy.value().bounds.max_subagent_refs {
            return Err(self.at(
                AtifIssueClass::Bound,
                AtifLocation::Step(step_id),
                AtifField::Bound,
            ));
        }
        add_count(&mut self.counts.subagent_references, references.len())?;
        let mut embedded_reference_ids = BTreeSet::new();
        for reference in references {
            let embedded = reference.trajectory_id.as_deref();
            let external = reference.trajectory_path.as_deref();
            if embedded.is_none() && external.is_none() {
                return Err(self.at(
                    AtifIssueClass::Reference,
                    AtifLocation::Step(step_id),
                    AtifField::Reference,
                ));
            }
            if reference.session_id.as_ref().is_some_and(|value| {
                !valid_identity(value, self.policy.value().bounds.max_string_bytes)
            }) {
                return Err(self.at(
                    AtifIssueClass::Identity,
                    AtifLocation::Step(step_id),
                    AtifField::Identity,
                ));
            }
            if let Some(identity) = embedded {
                if !embedded_ids.contains(identity) || !embedded_reference_ids.insert(identity) {
                    return Err(self.at(
                        AtifIssueClass::Reference,
                        AtifLocation::Step(step_id),
                        AtifField::Reference,
                    ));
                }
                self.references.push(ReferenceFact {
                    disposition: ReferenceDisposition::Embedded,
                    step_id: Some(step_id),
                });
            }
            if let Some(path) = external {
                self.validate_string(path, AtifLocation::Step(step_id), AtifField::Reference)?;
                self.classify_external(ReferenceDisposition::UnresolvedSubagent, Some(step_id))?;
            }
            self.validate_extra(reference.extra.as_ref())?;
        }
        Ok(())
    }

    fn validate_metrics(&mut self, metrics: &Metrics, step_id: u32) -> Result<(), AtifDiagnostic> {
        add_count(&mut self.counts.metrics, 1)?;
        let location = AtifLocation::Metrics(step_id);
        if metrics_entry_count(metrics) > self.policy.value().bounds.max_metrics_entries
            || metrics
                .cost_usd
                .is_some_and(|value| !valid_non_negative(value))
            || metrics
                .cached_tokens
                .zip(metrics.prompt_tokens)
                .is_some_and(|(cached, prompt)| cached > prompt)
            || !length_matches(metrics.prompt_token_ids.as_deref(), metrics.prompt_tokens)
            || !length_matches(
                metrics.completion_token_ids.as_deref(),
                metrics.completion_tokens,
            )
            || !length_matches(metrics.logprobs.as_deref(), metrics.completion_tokens)
            || metrics.logprobs.as_ref().is_some_and(|values| {
                values.len() > self.policy.value().bounds.max_token_items
                    || values.iter().any(|value| !value.is_finite())
            })
            || metrics
                .prompt_token_ids
                .as_ref()
                .is_some_and(|values| values.len() > self.policy.value().bounds.max_token_items)
            || metrics
                .completion_token_ids
                .as_ref()
                .is_some_and(|values| values.len() > self.policy.value().bounds.max_token_items)
        {
            return Err(self.at(AtifIssueClass::Metric, location, AtifField::Metrics));
        }
        self.validate_extra(metrics.extra.as_ref())
    }

    fn validate_final_metrics(
        &mut self,
        metrics: &FinalMetrics,
        root_steps: usize,
        notes_present: bool,
    ) -> Result<(), AtifDiagnostic> {
        add_count(&mut self.counts.metrics, 1)?;
        let undocumented_step_difference = metrics
            .total_steps
            .is_some_and(|declared| u64::try_from(root_steps) != Ok(declared) && !notes_present);
        if final_metric_entry_count(metrics) > self.policy.value().bounds.max_metrics_entries
            || undocumented_step_difference
            || metrics
                .total_cost_usd
                .is_some_and(|value| !valid_non_negative(value))
            || metrics
                .total_cached_tokens
                .zip(metrics.total_prompt_tokens)
                .is_some_and(|(cached, prompt)| cached > prompt)
        {
            return Err(self.at(
                AtifIssueClass::Metric,
                AtifLocation::Root,
                AtifField::Metrics,
            ));
        }
        self.validate_extra(metrics.extra.as_ref())
    }

    fn classify_external(
        &mut self,
        disposition: ReferenceDisposition,
        step_id: Option<u32>,
    ) -> Result<(), AtifDiagnostic> {
        if self.policy.value().external_references == ExternalReferencePolicy::Reject {
            return Err(self.at(
                AtifIssueClass::Reference,
                step_id.map_or(AtifLocation::Root, AtifLocation::Step),
                AtifField::Reference,
            ));
        }
        add_count(&mut self.counts.external_references, 1)?;
        self.references.push(ReferenceFact {
            disposition,
            step_id,
        });
        Ok(())
    }

    fn validate_extra(&mut self, object: Option<&AtifObject>) -> Result<(), AtifDiagnostic> {
        if let Some(object) = object {
            let disposition = self.policy.value().projection.extensions;
            if disposition == FieldDisposition::Reject
                || disposition == FieldDisposition::Hash
                    && !hashable_extension(object, &self.policy.value().extension_scalar_allowlist)
            {
                return Err(self.at(
                    AtifIssueClass::Extension,
                    AtifLocation::Extension,
                    AtifField::Extension,
                ));
            }
            self.validate_json_object(object, 1, JsonObjectClass::Extension)?;
        }
        Ok(())
    }

    fn validate_json_object(
        &mut self,
        object: &AtifObject,
        depth: usize,
        class: JsonObjectClass,
    ) -> Result<(), AtifDiagnostic> {
        if depth > self.policy.value().bounds.max_extension_depth {
            return Err(self.at(
                AtifIssueClass::Bound,
                AtifLocation::Extension,
                AtifField::Bound,
            ));
        }
        for (key, value) in object.entries() {
            if !valid_string(key, self.policy.value().bounds.max_string_bytes)
                || class == JsonObjectClass::Extension && unsafe_extension_key(key)
            {
                return Err(self.at(
                    AtifIssueClass::Extension,
                    AtifLocation::Extension,
                    AtifField::Extension,
                ));
            }
            self.json_nodes = self.json_nodes.saturating_add(1);
            if class == JsonObjectClass::Extension {
                self.extension_nodes = self.extension_nodes.saturating_add(1);
                self.extension_bytes = self.extension_bytes.saturating_add(key.len());
            }
            self.validate_json_value(value, depth, class)?;
            if self.json_nodes > self.policy.value().bounds.max_extension_nodes
                || self.extension_nodes > self.policy.value().bounds.max_extension_nodes
                || self.extension_bytes > self.policy.value().bounds.max_extension_bytes
            {
                return Err(self.at(
                    AtifIssueClass::Bound,
                    AtifLocation::Extension,
                    AtifField::Bound,
                ));
            }
        }
        self.counts.extension_nodes = bounded_u32(self.extension_nodes);
        self.counts.extension_bytes = u64::try_from(self.extension_bytes).unwrap_or(u64::MAX);
        Ok(())
    }

    fn validate_json_value(
        &mut self,
        value: &AtifValue,
        depth: usize,
        class: JsonObjectClass,
    ) -> Result<(), AtifDiagnostic> {
        self.json_nodes = self.json_nodes.saturating_add(1);
        if class == JsonObjectClass::Extension {
            self.extension_nodes = self.extension_nodes.saturating_add(1);
        }
        match value {
            AtifValue::Null
            | AtifValue::Bool(_)
            | AtifValue::Number(AtifNumber::Signed(_) | AtifNumber::Unsigned(_)) => {}
            AtifValue::Number(AtifNumber::Float(value)) if value.is_finite() => {}
            AtifValue::Number(AtifNumber::Float(_)) => {
                return Err(self.at(
                    AtifIssueClass::Extension,
                    AtifLocation::Extension,
                    AtifField::Extension,
                ));
            }
            AtifValue::String(value) => {
                if class == JsonObjectClass::Extension {
                    self.extension_bytes = self.extension_bytes.saturating_add(value.len());
                }
                if !valid_string(value, self.policy.value().bounds.max_string_bytes) {
                    return Err(self.at(
                        AtifIssueClass::Bound,
                        AtifLocation::Extension,
                        AtifField::Bound,
                    ));
                }
            }
            AtifValue::Array(values) => {
                if values.len() > self.policy.value().bounds.max_extension_nodes {
                    return Err(self.at(
                        AtifIssueClass::Bound,
                        AtifLocation::Extension,
                        AtifField::Bound,
                    ));
                }
                for value in values {
                    self.validate_json_value(value, depth.saturating_add(1), class)?;
                }
            }
            AtifValue::Object(object) => {
                self.validate_json_object(object, depth.saturating_add(1), class)?;
            }
        }
        Ok(())
    }

    fn validate_string(
        &self,
        value: &str,
        location: AtifLocation,
        field: AtifField,
    ) -> Result<(), AtifDiagnostic> {
        if !valid_string(value, self.policy.value().bounds.max_string_bytes) {
            return Err(self.at(AtifIssueClass::Bound, location, field));
        }
        Ok(())
    }

    #[allow(clippy::unused_self)]
    const fn at(
        &self,
        class: AtifIssueClass,
        location: AtifLocation,
        field: AtifField,
    ) -> AtifDiagnostic {
        AtifDiagnostic::new(class, location, field)
    }
}

fn embedded_id_set(
    document: &Trajectory,
    max_bytes: usize,
) -> Result<BTreeSet<&str>, AtifDiagnostic> {
    let mut identities = BTreeSet::new();
    if let Some(subagents) = &document.subagent_trajectories {
        for subagent in subagents {
            let identity = subagent.trajectory_id.as_deref().ok_or_else(|| {
                diagnostic(
                    AtifIssueClass::Identity,
                    AtifLocation::Root,
                    AtifField::Identity,
                )
            })?;
            if !valid_identity(identity, max_bytes) || !identities.insert(identity) {
                return Err(diagnostic(
                    AtifIssueClass::Reference,
                    AtifLocation::Root,
                    AtifField::Reference,
                ));
            }
        }
    }
    Ok(identities)
}

fn semantic_material(document: &Trajectory) -> Vec<String> {
    // The derived typed representation is deterministic because every map is a
    // BTreeMap. The identity domain binds this representation to this codec cohort.
    vec![format!("{document:?}")]
}

fn valid_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() < ISO_DATE_TIME_MIN_BYTES
        || bytes.get(ISO_YEAR_END) != Some(&b'-')
        || bytes.get(ISO_MONTH_END) != Some(&b'-')
        || bytes.get(ISO_DAY_END) != Some(&b'T')
        || bytes.get(ISO_HOUR_END) != Some(&b':')
        || bytes.get(ISO_MINUTE_END) != Some(&b':')
    {
        return false;
    }
    let ranges = [
        (0, ISO_YEAR_END),
        (ISO_MONTH_START, ISO_MONTH_END),
        (ISO_DAY_START, ISO_DAY_END),
        (ISO_HOUR_START, ISO_HOUR_END),
        (ISO_MINUTE_START, ISO_MINUTE_END),
        (ISO_SECOND_START, ISO_SECOND_END),
    ];
    if ranges
        .iter()
        .any(|(start, end)| !bytes[*start..*end].iter().all(u8::is_ascii_digit))
    {
        return false;
    }
    let year = parse_digits(&bytes[..ISO_YEAR_END]);
    let month = parse_digits(&bytes[ISO_MONTH_START..ISO_MONTH_END]);
    let day = parse_digits(&bytes[ISO_DAY_START..ISO_DAY_END]);
    let hour = parse_digits(&bytes[ISO_HOUR_START..ISO_HOUR_END]);
    let minute = parse_digits(&bytes[ISO_MINUTE_START..ISO_MINUTE_END]);
    let second = parse_digits(&bytes[ISO_SECOND_START..ISO_SECOND_END]);
    year.zip(month)
        .zip(day)
        .is_some_and(|((year, month), day)| {
            (MONTH_MIN..=MONTH_MAX).contains(&month)
                && (DAY_MIN..=days_in_month(year, month)).contains(&day)
        })
        && hour.is_some_and(|value| value <= HOUR_MAX)
        && minute.is_some_and(|value| value <= MINUTE_SECOND_MAX)
        && second.is_some_and(|value| value <= MINUTE_SECOND_MAX)
        && valid_time_suffix(&bytes[ISO_SECOND_END..])
}

fn valid_time_suffix(suffix: &[u8]) -> bool {
    if suffix.is_empty() || suffix == b"Z" {
        return true;
    }
    if suffix.first() == Some(&b'.') {
        let fraction_end = suffix[1..]
            .iter()
            .position(|byte| !byte.is_ascii_digit())
            .map_or(suffix.len(), |index| index.saturating_add(1));
        if fraction_end == 1 {
            return false;
        }
        return valid_zone_suffix(&suffix[fraction_end..]);
    }
    valid_zone_suffix(suffix)
}

fn valid_zone_suffix(suffix: &[u8]) -> bool {
    const OFFSET_BYTES: usize = 6;
    const OFFSET_COLON_INDEX: usize = 3;
    const OFFSET_HOUR_END: usize = 3;
    const OFFSET_MINUTE_START: usize = 4;
    suffix == b"Z"
        || suffix.len() == OFFSET_BYTES
            && suffix
                .first()
                .is_some_and(|value| matches!(value, b'+' | b'-'))
            && suffix.get(OFFSET_COLON_INDEX) == Some(&b':')
            && suffix[1..OFFSET_HOUR_END].iter().all(u8::is_ascii_digit)
            && suffix[OFFSET_MINUTE_START..].iter().all(u8::is_ascii_digit)
            && parse_digits(&suffix[1..OFFSET_HOUR_END]).is_some_and(|value| value <= HOUR_MAX)
            && parse_digits(&suffix[OFFSET_MINUTE_START..])
                .is_some_and(|value| value <= MINUTE_SECOND_MAX)
}

fn parse_digits(bytes: &[u8]) -> Option<u32> {
    bytes.iter().try_fold(0_u32, |value, byte| {
        value
            .checked_mul(10)
            .and_then(|value| value.checked_add(u32::from(byte.saturating_sub(b'0'))))
    })
}

fn length_matches<T>(values: Option<&[T]>, declared: Option<u64>) -> bool {
    match (values, declared) {
        (Some(values), Some(declared)) => u64::try_from(values.len()) == Ok(declared),
        (None, _) | (Some(_), None) => true,
    }
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        FEBRUARY if is_leap_year(year) => DAYS_FEBRUARY_LEAP,
        FEBRUARY => DAYS_FEBRUARY,
        APRIL | JUNE | SEPTEMBER | NOVEMBER => DAYS_SHORT_MONTH,
        _ => DAYS_LONG_MONTH,
    }
}

fn is_leap_year(year: u32) -> bool {
    year % LEAP_DIVISOR == 0 && (year % CENTURY_DIVISOR != 0 || year % LEAP_CENTURY_DIVISOR == 0)
}

fn metrics_entry_count(metrics: &Metrics) -> usize {
    [
        metrics.prompt_tokens.is_some(),
        metrics.completion_tokens.is_some(),
        metrics.cached_tokens.is_some(),
        metrics.cost_usd.is_some(),
        metrics.prompt_token_ids.is_some(),
        metrics.completion_token_ids.is_some(),
        metrics.logprobs.is_some(),
        metrics.extra.is_some(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count()
}

fn final_metric_entry_count(metrics: &FinalMetrics) -> usize {
    [
        metrics.total_prompt_tokens.is_some(),
        metrics.total_completion_tokens.is_some(),
        metrics.total_cached_tokens.is_some(),
        metrics.total_cost_usd.is_some(),
        metrics.total_steps.is_some(),
        metrics.extra.is_some(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count()
}

fn valid_non_negative(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

fn valid_identity(value: &str, max_bytes: usize) -> bool {
    valid_string(value, max_bytes) && !value.is_empty() && !value.chars().any(char::is_control)
}

fn valid_string(value: &str, max_bytes: usize) -> bool {
    value.len() <= max_bytes
}

fn hashable_extension(object: &AtifObject, allowlist: &[String]) -> bool {
    object.entries().iter().all(|(key, value)| {
        allowlist.contains(key)
            && matches!(
                value,
                AtifValue::Null | AtifValue::Bool(_) | AtifValue::Number(_) | AtifValue::String(_)
            )
    })
}

fn unsafe_extension_key(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "credential",
        "secret",
        "password",
        "private_key",
        "api_key",
        "provider_payload",
        "raw_prompt",
        "hidden_reasoning",
        "environment_value",
        "command",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn add_count(target: &mut u32, amount: usize) -> Result<(), AtifDiagnostic> {
    let amount = u32::try_from(amount)
        .map_err(|_| diagnostic(AtifIssueClass::Bound, AtifLocation::Root, AtifField::Bound))?;
    *target = target
        .checked_add(amount)
        .ok_or_else(|| diagnostic(AtifIssueClass::Bound, AtifLocation::Root, AtifField::Bound))?;
    Ok(())
}

fn bounded_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

const fn diagnostic(
    class: AtifIssueClass,
    location: AtifLocation,
    field: AtifField,
) -> AtifDiagnostic {
    AtifDiagnostic::new(class, location, field)
}
