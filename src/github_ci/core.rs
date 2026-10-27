//! Provider-independent asynchronous orchestration decisions.
//!
//! Shells verify producer signatures and translate native wire contracts into these
//! facts. Identities here describe public inputs, never credentials or their hashes.
//! Times are injected Unix milliseconds; producer seconds must be checked-converted
//! by the adapter. No successful projection proves build correctness or deployment.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Identity(pub [u8; 32]);

impl Identity {
    pub fn hex(self) -> String {
        blake3::Hash::from_bytes(self.0).to_hex().to_string()
    }
}

fn frame(hasher: &mut blake3::Hasher, value: &[u8]) {
    hasher.update(&(value.len() as u64).to_le_bytes());
    hasher.update(value);
}

fn identity(domain: &[u8], fields: &[&[u8]]) -> Identity {
    let mut hasher = blake3::Hasher::new();
    frame(&mut hasher, domain);
    for field in fields {
        frame(&mut hasher, field);
    }
    Identity(*hasher.finalize().as_bytes())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("invalid named policy bounds")]
    Policy,
    #[error("matrix or public field exceeds bounds")]
    Bounds,
    #[error("invalid public identifier")]
    Identifier,
    #[error("arithmetic overflow")]
    Overflow,
    #[error("capability exceeds the exact plan or is expired")]
    Capability,
    #[error("event or status does not own the exact binding")]
    Binding,
    #[error("conflicting or stale event identity or cursor")]
    Conflict,
    #[error("transition is not allowed")]
    Transition,
    #[error("terminal state is immutable")]
    Terminal,
    #[error("state version or invariant is invalid")]
    State,
    #[error("public URL is not explicitly permitted")]
    Url,
    #[error("rollout prerequisite is missing or stale")]
    Prerequisite,
}

/// Named resource limits, supplied explicitly by the operator policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub matrix_axes: usize,
    pub matrix_values_per_axis: usize,
    pub matrix_jobs: usize,
    pub public_label_bytes: usize,
    pub event_history: usize,
    pub evidence: usize,
    pub annotations: usize,
    pub annotation_bytes: usize,
    pub title_bytes: usize,
    pub summary_bytes: usize,
    pub text_bytes: usize,
    pub url_bytes: usize,
    pub reconciliations: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub max_elapsed_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    pub bounds: Bounds,
    pub retry: RetryPolicy,
    pub capability_lifetime_ms: u64,
    pub public_urls: PublicUrlPolicy,
}

impl Policy {
    pub fn validate(&self) -> Result<(), Error> {
        let b = &self.bounds;
        if [
            b.matrix_axes,
            b.matrix_values_per_axis,
            b.matrix_jobs,
            b.public_label_bytes,
            b.event_history,
            b.evidence,
            b.annotation_bytes,
            b.title_bytes,
            b.summary_bytes,
            b.text_bytes,
            b.url_bytes,
        ]
        .contains(&0)
            || b.reconciliations == 0
            || self.capability_lifetime_ms == 0
            || self.retry.initial_delay_ms == 0
            || self.retry.max_delay_ms < self.retry.initial_delay_ms
            || self.retry.max_elapsed_ms < self.retry.max_delay_ms
        {
            return Err(Error::Policy);
        }
        Ok(())
    }
}

pub type Coordinates = BTreeMap<String, String>;

fn label(value: &str, bounds: &Bounds) -> Result<(), Error> {
    if value.is_empty()
        || value.len() > bounds.public_label_bytes
        || !value.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    {
        return Err(Error::Identifier);
    }
    Ok(())
}

/// Axis names and values are public labels, not arbitrary workflow parameters.
/// Validates the complete checked product before allocating any expanded jobs.
pub fn expand_matrix(axes: &BTreeMap<String, Vec<String>>, bounds: &Bounds) -> Result<Vec<Coordinates>, Error> {
    if axes.len() > bounds.matrix_axes || bounds.matrix_jobs == 0 {
        return Err(Error::Bounds);
    }
    let mut count = 1usize;
    for (name, values) in axes {
        label(name, bounds)?;
        if values.is_empty() || values.len() > bounds.matrix_values_per_axis {
            return Err(Error::Bounds);
        }
        for (index, value) in values.iter().enumerate() {
            label(value, bounds)?;
            if values[..index].contains(value) {
                return Err(Error::Conflict);
            }
        }
        count = count.checked_mul(values.len()).ok_or(Error::Overflow)?;
        if count > bounds.matrix_jobs {
            return Err(Error::Bounds);
        }
    }
    let mut jobs = Vec::with_capacity(count);
    // Mixed-radix enumeration avoids allocating intermediate Cartesian products.
    for ordinal in 0..count {
        let mut remaining = ordinal;
        let mut coordinates = Coordinates::new();
        for (name, values) in axes.iter().rev() {
            let index = remaining % values.len();
            remaining /= values.len();
            coordinates.insert(name.clone(), values[index].clone());
        }
        jobs.push(coordinates);
    }
    Ok(jobs)
}

/// Concrete artifact identities only; the adapter constructs the producer's own
/// request schema. There are deliberately no expressions, commands or env maps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConcreteRequest {
    pub source: Identity,
    pub derivation: Identity,
    pub input_artifacts: BTreeSet<Identity>,
}

impl ConcreteRequest {
    pub fn identity(&self) -> Identity {
        let mut hasher = blake3::Hasher::new();
        frame(&mut hasher, b"lattice.github-ci.request.v1");
        frame(&mut hasher, &self.source.0);
        frame(&mut hasher, &self.derivation.0);
        frame(&mut hasher, &(self.input_artifacts.len() as u64).to_le_bytes());
        for artifact in &self.input_artifacts {
            frame(&mut hasher, &artifact.0);
        }
        Identity(*hasher.finalize().as_bytes())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkflowJobInput {
    pub workflow: Identity,
    pub node: Identity,
    pub generation: u32,
    pub coordinates: Coordinates,
    pub subject: Identity,
    pub audience: String,
    pub project: Identity,
    pub request: ConcreteRequest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Operation {
    Submit,
    Status,
    Events,
    Log,
    Result,
    OwnedCancel,
}

impl Operation {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Submit => "mantle/build/submit",
            Self::Status => "mantle/status/read",
            Self::Events => "mantle/event/read",
            Self::Log => "mantle/log/read",
            Self::Result => "mantle/result/read",
            Self::OwnedCancel => "mantle/attempt/cancel-owned",
        }
    }
}

/// One operation per plan: submission does not confer read or cancel authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapabilityPlan {
    pub audience: String,
    pub project: Identity,
    pub resource: Identity,
    pub operation: Operation,
    pub not_before_ms: u64,
    pub expires_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapabilityFacts {
    pub audience: String,
    pub project: Identity,
    pub resource: Identity,
    pub operations: BTreeSet<Operation>,
    pub not_before_ms: u64,
    pub expires_ms: u64,
}

impl CapabilityPlan {
    /// Public facts must already have been authenticated by the exchange shell.
    pub fn admit(&self, facts: &CapabilityFacts, now_ms: u64) -> Result<(), Error> {
        if facts.audience != self.audience
            || facts.project != self.project
            || facts.resource != self.resource
            || facts.operations.len() != 1
            || !facts.operations.contains(&self.operation)
            || facts.not_before_ms < self.not_before_ms
            || facts.not_before_ms > now_ms
            || facts.expires_ms <= now_ms
            || facts.expires_ms > self.expires_ms
            || self.expires_ms <= self.not_before_ms
        {
            return Err(Error::Capability);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobPlan {
    pub workflow: Identity,
    /// Exact public Basalt verified subject identity, distinct from holder DID.
    pub subject: Identity,
    pub job: Identity,
    pub generation: u32,
    pub request: ConcreteRequest,
    pub request_id: Identity,
    pub idempotency: Identity,
    pub submission: CapabilityPlan,
}

fn submission_idempotency(
    job: Identity,
    subject: Identity,
    project: Identity,
    request: Identity,
    audience: &str,
) -> Identity {
    identity(b"lattice.github-ci.submit.v1", &[&job.0, &subject.0, &project.0, &request.0, audience.as_bytes()])
}

impl JobPlan {
    /// Recheck public values before admitting them to a concrete shell. A caller
    /// cannot mutate subject or request while retaining a former submission key.
    pub fn validate(&self) -> Result<(), Error> {
        if self.request_id != self.request.identity()
            || self.submission.resource != self.request_id
            || self.submission.operation != Operation::Submit
            || self.idempotency
                != submission_idempotency(
                    self.job,
                    self.subject,
                    self.submission.project,
                    self.request_id,
                    &self.submission.audience,
                )
        {
            return Err(Error::Binding);
        }
        Ok(())
    }
}

pub fn plan_job(input: &WorkflowJobInput, policy: &Policy, now_ms: u64) -> Result<JobPlan, Error> {
    policy.validate()?;
    if input.coordinates.len() > policy.bounds.matrix_axes
        || input.request.input_artifacts.len() > policy.bounds.evidence
        || input.audience.is_empty()
        || input.audience.len() > policy.bounds.public_label_bytes
        || input.audience.chars().any(char::is_control)
    {
        return Err(Error::Bounds);
    }
    let mut hasher = blake3::Hasher::new();
    frame(&mut hasher, b"lattice.github-ci.job.v1");
    frame(&mut hasher, &input.workflow.0);
    frame(&mut hasher, &input.node.0);
    frame(&mut hasher, &input.generation.to_le_bytes());
    frame(&mut hasher, &(input.coordinates.len() as u64).to_le_bytes());
    for (name, value) in &input.coordinates {
        label(name, &policy.bounds)?;
        label(value, &policy.bounds)?;
        frame(&mut hasher, name.as_bytes());
        frame(&mut hasher, value.as_bytes());
    }
    let job = Identity(*hasher.finalize().as_bytes());
    let request_id = input.request.identity();
    let idempotency = submission_idempotency(job, input.subject, input.project, request_id, &input.audience);
    Ok(JobPlan {
        workflow: input.workflow,
        subject: input.subject,
        job,
        generation: input.generation,
        request: input.request.clone(),
        request_id,
        idempotency,
        submission: CapabilityPlan {
            audience: input.audience.clone(),
            project: input.project,
            resource: request_id,
            operation: Operation::Submit,
            not_before_ms: now_ms,
            expires_ms: now_ms.checked_add(policy.capability_lifetime_ms).ok_or(Error::Overflow)?,
        },
    })
}

pub fn attempt_capability(
    submission: &CapabilityPlan,
    attempt: Identity,
    operation: Operation,
    policy: &Policy,
    now_ms: u64,
) -> Result<CapabilityPlan, Error> {
    policy.validate()?;
    if operation == Operation::Submit {
        return Err(Error::Capability);
    }
    Ok(CapabilityPlan {
        audience: submission.audience.clone(),
        project: submission.project,
        resource: attempt,
        operation,
        not_before_ms: now_ms,
        expires_ms: now_ms.checked_add(policy.capability_lifetime_ms).ok_or(Error::Overflow)?,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Admitted,
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    Infrastructure,
}

impl Phase {
    pub const fn terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled | Self::Infrastructure)
    }
}

fn transition(from: Phase, to: Phase, reconciliation: bool) -> Result<(), Error> {
    if from.terminal() {
        return Err(Error::Terminal);
    }
    let allowed = from == to
        || matches!((from, to), (Phase::Admitted, Phase::Queued) | (Phase::Queued, Phase::Running))
        || to.terminal()
        || (reconciliation && from == Phase::Admitted && to == Phase::Running);
    if allowed { Ok(()) } else { Err(Error::Transition) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttemptBinding {
    pub request: Identity,
    pub attempt: Identity,
    pub fence: u64,
    pub producer: Identity,
}

/// All fields are public facts. No open-ended metadata, tokens, text, URL or
/// value-derived secret hashes belong in this durable representation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DurableState {
    pub schema_version: u32,
    pub workflow: Identity,
    pub job: Identity,
    pub generation: u32,
    pub idempotency: Identity,
    pub binding: AttemptBinding,
    pub check_id: Option<u64>,
    pub cursor: u64,
    pub version: u64,
    pub phase: Phase,
    pub history: BTreeMap<u64, EventFacts>,
    pub evidence: BTreeSet<Identity>,
    pub reconciliations: u32,
    pub awaiting_reconciliation: bool,
    pub cancellation_requested: bool,
}

pub const STATE_SCHEMA_VERSION: u32 = 1;

impl DurableState {
    pub fn new(plan: &JobPlan, binding: AttemptBinding) -> Result<Self, Error> {
        plan.validate()?;
        if binding.request != plan.request_id {
            return Err(Error::Binding);
        }
        Ok(Self {
            schema_version: STATE_SCHEMA_VERSION,
            workflow: plan.workflow,
            job: plan.job,
            generation: plan.generation,
            idempotency: plan.idempotency,
            binding,
            check_id: None,
            cursor: 0,
            version: 0,
            phase: Phase::Admitted,
            history: BTreeMap::new(),
            evidence: BTreeSet::new(),
            reconciliations: 0,
            awaiting_reconciliation: false,
            cancellation_requested: false,
        })
    }

    /// Recovery must additionally bind structurally valid public state to the
    /// exact reviewed job; a valid foreign record is not this job's authority.
    pub fn validate_for(&self, plan: &JobPlan, policy: &Policy) -> Result<(), Error> {
        plan.validate()?;
        self.validate(policy)?;
        if self.workflow != plan.workflow
            || self.job != plan.job
            || self.generation != plan.generation
            || self.idempotency != plan.idempotency
            || self.binding.request != plan.request_id
        {
            return Err(Error::Binding);
        }
        Ok(())
    }

    pub fn validate(&self, policy: &Policy) -> Result<(), Error> {
        policy.validate()?;
        if self.schema_version != STATE_SCHEMA_VERSION
            || self.check_id == Some(0)
            || self.history.len() > policy.bounds.event_history
            || self.evidence.len() > policy.bounds.evidence
            || self.reconciliations > policy.bounds.reconciliations
            || (self.phase.terminal() && self.awaiting_reconciliation)
            || self.history.keys().next_back().copied().unwrap_or(0) != self.cursor
        {
            return Err(Error::State);
        }
        let mut ids = BTreeSet::new();
        let mut evidence = BTreeSet::new();
        let mut previous_phase = Phase::Admitted;
        let mut previous_sequence = 0u64;
        let mut gaps = 0u32;
        for (sequence, event) in &self.history {
            if *sequence == 0
                || *sequence != event.sequence
                || event.binding != self.binding
                || !ids.insert(event.event)
                || event.evidence.len() > policy.bounds.evidence
            {
                return Err(Error::State);
            }
            if *sequence != previous_sequence.checked_add(1).ok_or(Error::Overflow)?
                || (previous_phase == Phase::Admitted && event.phase == Phase::Running)
            {
                gaps = gaps.checked_add(1).ok_or(Error::Overflow)?;
            }
            transition(previous_phase, event.phase, true).map_err(|_| Error::State)?;
            previous_phase = event.phase;
            previous_sequence = *sequence;
            evidence.extend(&event.evidence);
        }
        if evidence != self.evidence || gaps > self.reconciliations || self.version < self.history.len() as u64 {
            return Err(Error::State);
        }
        if let Some(last) = self.history.values().next_back() {
            if last.phase != self.phase {
                return Err(Error::State);
            }
        } else if self.phase != Phase::Admitted {
            return Err(Error::State);
        }
        Ok(())
    }
}

/// Neutral facts, not a Mantle wire schema. An event identity is producer-owned;
/// exact duplicate equality includes its entire admitted public fact record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventFacts {
    pub binding: AttemptBinding,
    pub event: Identity,
    pub sequence: u64,
    pub phase: Phase,
    pub evidence: BTreeSet<Identity>,
}

/// Cannot be deserialized or constructed by untrusted consumers. Only the sibling
/// transport shell may assert that its configured producer verification passed.
#[derive(Clone, Debug)]
pub struct VerifiedEvent(EventFacts);

impl VerifiedEvent {
    pub(super) fn after_verification(facts: EventFacts) -> Self {
        Self(facts)
    }
}

/// A status observation authenticates a latest event, not just a status string.
#[derive(Clone, Debug)]
pub struct VerifiedStatus(EventFacts);

impl VerifiedStatus {
    pub(super) fn after_verification(facts: EventFacts) -> Self {
        Self(facts)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Disposition {
    Applied,
    Duplicate,
    Reconcile,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reduction {
    pub state: DurableState,
    pub disposition: Disposition,
}

fn admit_facts(state: &DurableState, facts: &EventFacts, policy: &Policy) -> Result<(), Error> {
    state.validate(policy)?;
    if state.binding != facts.binding {
        return Err(Error::Binding);
    }
    if facts.sequence == 0 {
        return Err(Error::Conflict);
    }
    if facts.evidence.len() > policy.bounds.evidence {
        return Err(Error::Bounds);
    }
    if state.history.values().any(|old| old.event == facts.event && old != facts) {
        return Err(Error::Conflict);
    }
    Ok(())
}

fn apply(state: &DurableState, facts: &EventFacts, policy: &Policy) -> Result<Reduction, Error> {
    if state.history.len() >= policy.bounds.event_history {
        return Err(Error::Bounds);
    }
    let added = facts.evidence.difference(&state.evidence).count();
    if state.evidence.len().checked_add(added).ok_or(Error::Overflow)? > policy.bounds.evidence {
        return Err(Error::Bounds);
    }
    let mut next = state.clone();
    next.version = next.version.checked_add(1).ok_or(Error::Overflow)?;
    next.cursor = facts.sequence;
    next.phase = facts.phase;
    next.history.insert(facts.sequence, facts.clone());
    next.evidence.extend(&facts.evidence);
    next.awaiting_reconciliation = false;
    Ok(Reduction {
        state: next,
        disposition: Disposition::Applied,
    })
}

pub fn reduce(state: &DurableState, event: &VerifiedEvent, policy: &Policy) -> Result<Reduction, Error> {
    let facts = &event.0;
    admit_facts(state, facts, policy)?;
    if let Some(old) = state.history.get(&facts.sequence) {
        return if old == facts {
            Ok(Reduction {
                state: state.clone(),
                disposition: Disposition::Duplicate,
            })
        } else {
            Err(Error::Conflict)
        };
    }
    if state.phase.terminal() {
        return Err(Error::Terminal);
    }
    if facts.sequence <= state.cursor {
        return Err(Error::Conflict);
    }
    if state.awaiting_reconciliation || facts.sequence != state.cursor.checked_add(1).ok_or(Error::Overflow)? {
        if state.reconciliations >= policy.bounds.reconciliations {
            return Err(Error::Bounds);
        }
        let mut next = state.clone();
        if !next.awaiting_reconciliation {
            next.version = next.version.checked_add(1).ok_or(Error::Overflow)?;
            next.awaiting_reconciliation = true;
        }
        return Ok(Reduction {
            state: next,
            disposition: Disposition::Reconcile,
        });
    }
    transition(state.phase, facts.phase, false)?;
    apply(state, facts, policy)
}

pub fn reconcile(state: &DurableState, status: &VerifiedStatus, policy: &Policy) -> Result<Reduction, Error> {
    let facts = &status.0;
    admit_facts(state, facts, policy)?;
    if let Some(old) = state.history.get(&facts.sequence) {
        if old != facts {
            return Err(Error::Conflict);
        }
        // A same-head observation cannot fill an already observed sequence gap.
        if state.awaiting_reconciliation {
            return Err(Error::Conflict);
        }
        return Ok(Reduction {
            state: state.clone(),
            disposition: Disposition::Duplicate,
        });
    }
    if facts.sequence <= state.cursor {
        return Err(Error::Conflict);
    }
    if state.reconciliations >= policy.bounds.reconciliations {
        return Err(Error::Bounds);
    }
    transition(state.phase, facts.phase, true)?;
    let mut reduction = apply(state, facts, policy)?;
    reduction.state.reconciliations += 1;
    Ok(reduction)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObservedError {
    RateLimited,
    AbuseLimited,
    Transient,
    Timeout,
    Disconnected,
    Unauthorized,
    PermissionDenied,
    MissingRepository,
    Malformed,
    DuplicateCreate,
    StaleVersion,
    Rejected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetryDecision {
    RetryAt(u64),
    Reconcile,
    Stop,
}

/// Lost responses never authorize a fresh submission: retry means the exact same
/// request/idempotency key. Duplicate creates and stale updates need read-back.
pub fn retry_decision(
    error: ObservedError,
    retries: u32,
    started_ms: u64,
    now_ms: u64,
    retry_after_ms: Option<u64>,
    policy: &Policy,
) -> Result<RetryDecision, Error> {
    policy.validate()?;
    let elapsed = now_ms.checked_sub(started_ms).ok_or(Error::State)?;
    if matches!(error, ObservedError::DuplicateCreate | ObservedError::StaleVersion) {
        return Ok(RetryDecision::Reconcile);
    }
    if !matches!(
        error,
        ObservedError::RateLimited
            | ObservedError::AbuseLimited
            | ObservedError::Transient
            | ObservedError::Timeout
            | ObservedError::Disconnected
    ) || retries >= policy.retry.max_retries
        || elapsed >= policy.retry.max_elapsed_ms
    {
        return Ok(RetryDecision::Stop);
    }
    let multiplier = 1u64.checked_shl(retries).unwrap_or(u64::MAX);
    let backoff = policy.retry.initial_delay_ms.saturating_mul(multiplier).min(policy.retry.max_delay_ms);
    let delay = backoff.max(retry_after_ms.unwrap_or(0));
    // Never retry earlier than an excessive server delay; stop instead of capping it.
    if delay > policy.retry.max_delay_ms
        || elapsed.checked_add(delay).ok_or(Error::Overflow)? > policy.retry.max_elapsed_ms
    {
        return Ok(RetryDecision::Stop);
    }
    Ok(RetryDecision::RetryAt(now_ms.checked_add(delay).ok_or(Error::Overflow)?))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CancelReason {
    User,
    Superseded,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CancelIntent {
    pub binding: AttemptBinding,
    pub expected_version: u64,
    pub reason: CancelReason,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cancellation {
    pub state: DurableState,
    pub request: Option<AttemptBinding>,
}

pub fn cancel(
    state: &DurableState,
    intent: &CancelIntent,
    capability: &CapabilityPlan,
    facts: &CapabilityFacts,
    policy: &Policy,
    now_ms: u64,
) -> Result<Cancellation, Error> {
    state.validate(policy)?;
    if intent.binding != state.binding || intent.expected_version != state.version {
        return Err(Error::Binding);
    }
    if capability.resource != state.binding.attempt || capability.operation != Operation::OwnedCancel {
        return Err(Error::Capability);
    }
    capability.admit(facts, now_ms)?;
    if state.phase.terminal() || state.cancellation_requested {
        return Ok(Cancellation {
            state: state.clone(),
            request: None,
        });
    }
    let mut next = state.clone();
    next.version = next.version.checked_add(1).ok_or(Error::Overflow)?;
    next.cancellation_requested = true;
    // Intent does not claim that the provider cancelled the execution.
    Ok(Cancellation {
        state: next,
        request: Some(state.binding),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestartDecision {
    TerminalProjection,
    ReacquireAndReconcile(AttemptBinding),
}

pub fn restart(state: &DurableState, policy: &Policy) -> Result<RestartDecision, Error> {
    state.validate(policy)?;
    if state.phase.terminal() {
        return Ok(RestartDecision::TerminalProjection);
    }
    if state.reconciliations >= policy.bounds.reconciliations {
        return Err(Error::Bounds);
    }
    Ok(RestartDecision::ReacquireAndReconcile(state.binding))
}

/// Exact public HTTPS origins and path prefixes. No host suffix matching, query,
/// fragment, userinfo, redirects, or arbitrary credential-bearing links.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicUrlPolicy {
    pub allowed: BTreeSet<PublicUrlPrefix>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicUrlPrefix {
    pub origin: String,
    pub path_prefix: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicUrl(String);

impl PublicUrl {
    pub fn parse(value: &str, policy: &PublicUrlPolicy, bounds: &Bounds) -> Result<Self, Error> {
        if value.len() > bounds.url_bytes
            || value.chars().any(char::is_control)
            || value.contains('\\')
            || value.contains('%')
            || value.contains('@')
        {
            return Err(Error::Url);
        }
        let url = url::Url::parse(value).map_err(|_| Error::Url)?;
        if url.scheme() != "https"
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || url.host_str().is_none()
        {
            return Err(Error::Url);
        }
        let origin = url.origin().ascii_serialization();
        if !policy.allowed.iter().any(|prefix| {
            prefix.origin == origin
                && prefix.path_prefix.starts_with('/')
                && (url.path() == prefix.path_prefix
                    || (prefix.path_prefix.ends_with('/') && url.path().starts_with(&prefix.path_prefix)))
        }) {
            return Err(Error::Url);
        }
        Ok(Self(url.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Annotation {
    pub path: String,
    pub line: u32,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckStatus {
    Queued,
    InProgress,
    Completed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Conclusion {
    Success,
    Failure,
    Cancelled,
    ActionRequired,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckProjection {
    pub check_id: Option<u64>,
    pub external_id: Identity,
    pub update_id: Identity,
    pub state_version: u64,
    pub status: CheckStatus,
    pub conclusion: Option<Conclusion>,
    pub title: String,
    pub summary: String,
    pub text: String,
    pub annotations: Vec<Annotation>,
    pub links: Vec<PublicUrl>,
}

/// Projection accepts public, already-redacted content, never raw HTTP bodies or
/// logs. Bounds reject rather than silently changing consumer-visible evidence.
pub fn project_check(
    state: &DurableState,
    title: &str,
    summary: &str,
    text: &str,
    annotations: &[Annotation],
    links: &[PublicUrl],
    policy: &Policy,
) -> Result<CheckProjection, Error> {
    state.validate(policy)?;
    let b = &policy.bounds;
    if title.len() > b.title_bytes
        || summary.len() > b.summary_bytes
        || text.len() > b.text_bytes
        || annotations.len() > b.annotations
        || links.len() > b.evidence
    {
        return Err(Error::Bounds);
    }
    for annotation in annotations {
        if annotation.line == 0
            || annotation.path.is_empty()
            || annotation.path.starts_with('/')
            || annotation.path.contains('\\')
            || annotation.path.split('/').any(|p| p == ".." || p.is_empty())
            || annotation.path.len() > b.public_label_bytes
            || annotation.message.len() > b.annotation_bytes
            || annotation.path.chars().any(char::is_control)
        {
            return Err(Error::Bounds);
        }
    }
    for link in links {
        PublicUrl::parse(link.as_str(), &policy.public_urls, b)?;
    }
    let (status, conclusion) = match state.phase {
        Phase::Admitted | Phase::Queued => (CheckStatus::Queued, None),
        Phase::Running => (CheckStatus::InProgress, None),
        Phase::Succeeded => (CheckStatus::Completed, Some(Conclusion::Success)),
        Phase::Failed => (CheckStatus::Completed, Some(Conclusion::Failure)),
        Phase::Cancelled => (CheckStatus::Completed, Some(Conclusion::Cancelled)),
        Phase::Infrastructure => (CheckStatus::Completed, Some(Conclusion::ActionRequired)),
    };
    // Include payload identity: equal update keys must never describe unequal content.
    let mut hasher = blake3::Hasher::new();
    frame(&mut hasher, b"lattice.github-ci.check-update.v1");
    frame(&mut hasher, &state.job.0);
    frame(&mut hasher, &state.version.to_le_bytes());
    frame(&mut hasher, &[state.phase as u8]);
    for value in [title, summary, text] {
        frame(&mut hasher, value.as_bytes());
    }
    frame(&mut hasher, &(annotations.len() as u64).to_le_bytes());
    for annotation in annotations {
        frame(&mut hasher, annotation.path.as_bytes());
        frame(&mut hasher, &annotation.line.to_le_bytes());
        frame(&mut hasher, annotation.message.as_bytes());
    }
    frame(&mut hasher, &(links.len() as u64).to_le_bytes());
    for link in links {
        frame(&mut hasher, link.as_str().as_bytes());
    }
    Ok(CheckProjection {
        check_id: state.check_id,
        external_id: state.job,
        update_id: Identity(*hasher.finalize().as_bytes()),
        state_version: state.version,
        status,
        conclusion,
        title: title.to_owned(),
        summary: summary.to_owned(),
        text: text.to_owned(),
        annotations: annotations.to_vec(),
        links: links.to_vec(),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    Offline,
    Private,
    ReportOnly,
    Required,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RolloutEvidence {
    pub candidate: Identity,
    pub producer_prerequisites: BTreeSet<Identity>,
    pub offline: bool,
    pub private: bool,
    pub report_only: bool,
    pub recovery: bool,
    pub duplicate_events: bool,
    pub cancellation: bool,
    pub credential_redaction: bool,
    pub cairn_gate: Option<Identity>,
}

pub fn admit_stage(
    stage: Stage,
    candidate: Identity,
    required_producers: &BTreeSet<Identity>,
    evidence: &RolloutEvidence,
) -> Result<(), Error> {
    if stage == Stage::Offline {
        return Ok(());
    }
    if evidence.candidate != candidate || !evidence.offline {
        return Err(Error::Prerequisite);
    }
    if stage >= Stage::ReportOnly && !evidence.private {
        return Err(Error::Prerequisite);
    }
    if stage == Stage::Required
        && (required_producers.is_empty()
            || !required_producers.is_subset(&evidence.producer_prerequisites)
            || !evidence.report_only
            || !evidence.recovery
            || !evidence.duplicate_events
            || !evidence.cancellation
            || !evidence.credential_redaction
            || evidence.cairn_gate.is_none())
    {
        return Err(Error::Prerequisite);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u8) -> Identity {
        Identity([n; 32])
    }

    fn policy() -> Policy {
        Policy {
            bounds: Bounds {
                matrix_axes: 8,
                matrix_values_per_axis: 8,
                matrix_jobs: 16,
                public_label_bytes: 128,
                event_history: 32,
                evidence: 8,
                annotations: 2,
                annotation_bytes: 128,
                title_bytes: 64,
                summary_bytes: 256,
                text_bytes: 512,
                url_bytes: 512,
                reconciliations: 3,
            },
            retry: RetryPolicy {
                max_retries: 3,
                initial_delay_ms: 100,
                max_delay_ms: 800,
                max_elapsed_ms: 2000,
            },
            capability_lifetime_ms: 1000,
            public_urls: PublicUrlPolicy {
                allowed: BTreeSet::from([PublicUrlPrefix {
                    origin: "https://evidence.example".into(),
                    path_prefix: "/public/".into(),
                }]),
            },
        }
    }

    fn input() -> WorkflowJobInput {
        WorkflowJobInput {
            workflow: id(1),
            node: id(2),
            generation: 0,
            coordinates: BTreeMap::from([("system".into(), "linux".into())]),
            subject: id(3),
            audience: "mantle.example".into(),
            project: id(4),
            request: ConcreteRequest {
                source: id(5),
                derivation: id(6),
                input_artifacts: BTreeSet::from([id(7)]),
            },
        }
    }

    fn initial() -> DurableState {
        let plan = plan_job(&input(), &policy(), 10).unwrap();
        DurableState::new(&plan, AttemptBinding {
            request: plan.request_id,
            attempt: id(8),
            fence: 9,
            producer: id(10),
        })
        .unwrap()
    }

    fn event(state: &DurableState, sequence: u64, phase: Phase) -> VerifiedEvent {
        VerifiedEvent::after_verification(EventFacts {
            binding: state.binding,
            event: identity(b"test.public.event", &[&sequence.to_le_bytes()]),
            sequence,
            phase,
            evidence: BTreeSet::new(),
        })
    }

    fn facts(plan: &CapabilityPlan) -> CapabilityFacts {
        CapabilityFacts {
            audience: plan.audience.clone(),
            project: plan.project,
            resource: plan.resource,
            operations: BTreeSet::from([plan.operation]),
            not_before_ms: plan.not_before_ms,
            expires_ms: plan.expires_ms,
        }
    }

    #[test]
    fn matrix_is_canonical_bounded_and_rejects_duplicate_coordinates() {
        let mut axes = BTreeMap::from([
            ("system".into(), vec!["linux".into(), "darwin".into()]),
            ("mode".into(), vec!["debug".into(), "release".into()]),
        ]);
        let jobs = expand_matrix(&axes, &policy().bounds).unwrap();
        assert_eq!(jobs, vec![
            BTreeMap::from([("mode".into(), "debug".into()), ("system".into(), "linux".into())]),
            BTreeMap::from([("mode".into(), "debug".into()), ("system".into(), "darwin".into())]),
            BTreeMap::from([("mode".into(), "release".into()), ("system".into(), "linux".into())]),
            BTreeMap::from([("mode".into(), "release".into()), ("system".into(), "darwin".into())]),
        ]);
        let mut bounds = policy().bounds;
        bounds.matrix_jobs = 3;
        assert_eq!(expand_matrix(&axes, &bounds), Err(Error::Bounds));
        axes.insert("system".into(), vec!["linux".into(), "linux".into()]);
        assert_eq!(expand_matrix(&axes, &policy().bounds), Err(Error::Conflict));
        assert_eq!(expand_matrix(&BTreeMap::new(), &bounds), Ok(vec![BTreeMap::new()]));
        axes.insert("system".into(), Vec::new());
        assert_eq!(expand_matrix(&axes, &bounds), Err(Error::Bounds));
    }

    #[test]
    fn job_and_submission_bind_every_identity_without_renewal_churn() {
        let p = policy();
        let base = plan_job(&input(), &p, 10).unwrap();
        let renewed = plan_job(&input(), &p, 20).unwrap();
        assert_eq!(base.job, renewed.job);
        assert_eq!(base.idempotency, renewed.idempotency);
        for field in 0..7 {
            let mut changed = input();
            match field {
                0 => changed.workflow = id(11),
                1 => changed.node = id(11),
                2 => changed.generation += 1,
                3 => {
                    changed.coordinates.insert("system".into(), "darwin".into());
                }
                4 => changed.subject = id(11),
                5 => changed.project = id(11),
                _ => changed.request.derivation = id(11),
            }
            let other = plan_job(&changed, &p, 10).unwrap();
            assert_ne!(base.idempotency, other.idempotency, "field {field}");
            if field < 4 {
                assert_ne!(base.job, other.job);
            }
        }
        assert_eq!(plan_job(&input(), &p, u64::MAX), Err(Error::Overflow));
        let mut foreign_input = input();
        foreign_input.audience = "other-service".into();
        let foreign = plan_job(&foreign_input, &p, 10).unwrap();
        assert_ne!(base.idempotency, foreign.idempotency);
        assert_eq!(initial().validate_for(&foreign, &p), Err(Error::Binding));
        assert_eq!(initial().validate_for(&base, &p), Ok(()));
    }

    #[test]
    fn mutated_plan_cannot_reuse_submission_identity_for_another_subject_or_request() {
        let plan = plan_job(&input(), &policy(), 10).unwrap();
        plan.validate().unwrap();
        for mutation in 0..7 {
            let mut changed = plan.clone();
            match mutation {
                0 => changed.subject = id(99),
                1 => changed.request.derivation = id(99),
                2 => changed.request_id = id(99),
                3 => changed.submission.resource = id(99),
                4 => changed.submission.project = id(99),
                5 => changed.submission.audience = "foreign-service".into(),
                _ => changed.idempotency = id(99),
            }
            assert_eq!(changed.validate(), Err(Error::Binding), "mutation {mutation}");
            assert_eq!(DurableState::new(&changed, initial().binding), Err(Error::Binding));
        }
    }

    #[test]
    fn matrix_product_overflow_is_checked_before_expansion() {
        let mut b = policy().bounds;
        b.matrix_axes = usize::BITS as usize;
        b.matrix_jobs = usize::MAX;
        let axes = (0..usize::BITS).map(|i| (format!("axis{i}"), vec!["a".into(), "b".into()])).collect();
        assert_eq!(expand_matrix(&axes, &b), Err(Error::Overflow));
    }

    #[test]
    fn capabilities_reject_each_broadening_and_expiry_boundary() {
        let plan = plan_job(&input(), &policy(), 10).unwrap().submission;
        assert_eq!(plan.admit(&facts(&plan), 10), Ok(()));
        assert_eq!(plan.admit(&facts(&plan), plan.expires_ms), Err(Error::Capability));
        for field in 0..7 {
            let mut f = facts(&plan);
            match field {
                0 => f.audience.push('x'),
                1 => f.project = id(99),
                2 => f.resource = id(99),
                3 => {
                    f.operations.insert(Operation::OwnedCancel);
                }
                4 => f.expires_ms += 1,
                5 => f.not_before_ms -= 1,
                _ => f.not_before_ms += 1,
            }
            assert_eq!(plan.admit(&f, 10), Err(Error::Capability), "field {field}");
        }
    }

    #[test]
    fn reducer_replay_is_deterministic_and_every_old_duplicate_is_noop() {
        let p = policy();
        // Exhaustive bounded lifecycle samples: replay and all earlier duplicates.
        for terminal in [Phase::Succeeded, Phase::Failed, Phase::Cancelled, Phase::Infrastructure] {
            for running_progress in 0..12 {
                let mut left = initial();
                let mut right = left.clone();
                let mut delivered = Vec::new();
                let phases = [Phase::Queued, Phase::Running]
                    .into_iter()
                    .chain(std::iter::repeat_n(Phase::Running, running_progress))
                    .chain([terminal]);
                for (index, phase) in phases.enumerate() {
                    let e = event(&left, index as u64 + 1, phase);
                    left = reduce(&left, &e, &p).unwrap().state;
                    right = reduce(&right, &e, &p).unwrap().state;
                    assert_eq!(left, right);
                    left.validate(&p).unwrap();
                    delivered.push(e);
                    for old in &delivered {
                        let duplicate = reduce(&left, old, &p).unwrap();
                        assert_eq!(duplicate.disposition, Disposition::Duplicate);
                        assert_eq!(duplicate.state, left);
                    }
                }
                assert_eq!(reduce(&left, &event(&left, left.cursor + 1, Phase::Running), &p), Err(Error::Terminal));
            }
        }
    }

    #[test]
    fn binding_event_conflict_regression_and_transition_rejection() {
        let p = policy();
        let state = initial();
        let good = event(&state, 1, Phase::Queued);
        for field in 0..4 {
            let mut bad = good.clone();
            match field {
                0 => bad.0.binding.request = id(91),
                1 => bad.0.binding.attempt = id(91),
                2 => bad.0.binding.fence += 1,
                _ => bad.0.binding.producer = id(91),
            }
            assert_eq!(reduce(&state, &bad, &p), Err(Error::Binding));
        }
        assert_eq!(reduce(&state, &event(&state, 1, Phase::Running), &p), Err(Error::Transition));
        let queued = reduce(&state, &good, &p).unwrap().state;
        let mut conflict = good.clone();
        conflict.0.phase = Phase::Failed;
        assert_eq!(reduce(&queued, &conflict, &p), Err(Error::Conflict));
        conflict.0.sequence = 2;
        assert_eq!(reduce(&queued, &conflict, &p), Err(Error::Conflict));
        let running = reduce(&queued, &event(&queued, 2, Phase::Running), &p).unwrap().state;
        assert_eq!(reduce(&running, &event(&running, 3, Phase::Queued), &p), Err(Error::Transition));
    }

    #[test]
    fn gaps_remain_uncertain_until_bounded_authenticated_status_reconciliation() {
        let p = policy();
        let state = initial();
        let gap = event(&state, 3, Phase::Running);
        let pending = reduce(&state, &gap, &p).unwrap();
        assert_eq!(pending.disposition, Disposition::Reconcile);
        assert_eq!(pending.state.phase, Phase::Admitted);
        assert_eq!(pending.state.cursor, 0);
        assert_eq!(
            reduce(&pending.state, &event(&state, 1, Phase::Queued), &p).unwrap().disposition,
            Disposition::Reconcile
        );
        let reconciled = reconcile(&pending.state, &VerifiedStatus::after_verification(gap.0), &p).unwrap().state;
        assert_eq!(reconciled.cursor, 3);
        assert_eq!(reconciled.phase, Phase::Running);
        assert!(!reconciled.awaiting_reconciliation);
        reconciled.validate(&p).unwrap();
        let mut exhausted = reconciled.clone();
        exhausted.reconciliations = p.bounds.reconciliations;
        assert_eq!(reduce(&exhausted, &event(&exhausted, 5, Phase::Succeeded), &p), Err(Error::Bounds));
        assert_eq!(restart(&exhausted, &p), Err(Error::Bounds));
    }

    #[test]
    fn persisted_state_rejects_unknown_fields_and_corrupt_public_invariants() {
        let p = policy();
        let state = initial();
        let mut encoded = serde_json::to_value(&state).unwrap();
        encoded["provider_value"] = serde_json::json!("not-admitted");
        assert!(serde_json::from_value::<DurableState>(encoded).is_err());
        for field in 0..5 {
            let mut invalid = state.clone();
            match field {
                0 => invalid.schema_version += 1,
                1 => invalid.cursor = 1,
                2 => invalid.phase = Phase::Succeeded,
                3 => invalid.check_id = Some(0),
                _ => {
                    invalid.evidence.insert(id(99));
                }
            }
            assert_eq!(invalid.validate(&p), Err(Error::State));
        }
        let mut limited = p.clone();
        limited.bounds.event_history = 1;
        let queued = reduce(&state, &event(&state, 1, Phase::Queued), &limited).unwrap().state;
        assert_eq!(reduce(&queued, &event(&queued, 2, Phase::Running), &limited), Err(Error::Bounds));
    }

    #[test]
    fn retry_respects_server_delay_elapsed_bounds_and_closed_denials() {
        let p = policy();
        assert_eq!(
            retry_decision(ObservedError::RateLimited, 0, 0, 100, Some(500), &p),
            Ok(RetryDecision::RetryAt(600))
        );
        assert_eq!(retry_decision(ObservedError::RateLimited, 0, 0, 100, Some(801), &p), Ok(RetryDecision::Stop));
        assert_eq!(retry_decision(ObservedError::Transient, 3, 0, 100, None, &p), Ok(RetryDecision::Stop));
        assert_eq!(retry_decision(ObservedError::Transient, 0, 0, 1950, None, &p), Ok(RetryDecision::Stop));
        for error in [
            ObservedError::Unauthorized,
            ObservedError::PermissionDenied,
            ObservedError::MissingRepository,
            ObservedError::Malformed,
        ] {
            assert_eq!(retry_decision(error, 0, 0, 1, None, &p), Ok(RetryDecision::Stop));
        }
        assert_eq!(retry_decision(ObservedError::StaleVersion, 0, 0, 1, None, &p), Ok(RetryDecision::Reconcile));
    }

    #[test]
    fn owned_cancel_is_intent_not_terminal_truth_and_restart_requires_reconciliation() {
        let p = policy();
        let state = initial();
        let submission = plan_job(&input(), &p, 10).unwrap().submission;
        let capability =
            attempt_capability(&submission, state.binding.attempt, Operation::OwnedCancel, &p, 10).unwrap();
        let intent = CancelIntent {
            binding: state.binding,
            expected_version: 0,
            reason: CancelReason::Superseded,
        };
        let cancelled = cancel(&state, &intent, &capability, &facts(&capability), &p, 10).unwrap();
        assert_eq!(cancelled.request, Some(state.binding));
        assert_eq!(cancelled.state.phase, Phase::Admitted);
        assert_eq!(restart(&cancelled.state, &p), Ok(RestartDecision::ReacquireAndReconcile(state.binding)));
        let terminal = reduce(&cancelled.state, &event(&state, 1, Phase::Succeeded), &p).unwrap().state;
        assert_eq!(restart(&terminal, &p), Ok(RestartDecision::TerminalProjection));
        assert_eq!(cancel(&state, &intent, &submission, &facts(&submission), &p, 10), Err(Error::Capability));
        let mut stale = intent;
        stale.expected_version = 99;
        assert_eq!(cancel(&state, &stale, &capability, &facts(&capability), &p, 10), Err(Error::Binding));
    }

    #[test]
    fn projection_bounds_exact_public_links_and_payload_idempotency() {
        let p = policy();
        let state = initial();
        let link = PublicUrl::parse("https://evidence.example/public/job-1", &p.public_urls, &p.bounds).unwrap();
        for value in [
            "http://evidence.example/public/a",
            "https://evidence.example.evil/public/a",
            "https://user:secret@evidence.example/public/a",
            "https://evidence.example/public/a?token=x",
            "https://evidence.example/private/a",
            "https://evidence.example/public/a#token",
            "https://evidence.example/public/%2e%2e/private",
            "https://evidence.example/publicity/a",
        ] {
            assert_eq!(PublicUrl::parse(value, &p.public_urls, &p.bounds), Err(Error::Url), "{value}");
        }
        let terminal = reduce(&state, &event(&state, 1, Phase::Succeeded), &p).unwrap().state;
        let projection =
            project_check(&terminal, "Build", "Observed success", "Public evidence", &[], &[link], &p).unwrap();
        assert_eq!(projection.status, CheckStatus::Completed);
        assert_eq!(projection.conclusion, Some(Conclusion::Success));
        let mut bound = terminal.clone();
        bound.check_id = Some(42);
        let after_create =
            project_check(&bound, "Build", "Observed success", "Public evidence", &[], &projection.links, &p).unwrap();
        assert_eq!(after_create.update_id, projection.update_id);
        assert_eq!(after_create.check_id, Some(42));
        let changed = project_check(&terminal, "Build", "Other observation", "Public evidence", &[], &[], &p).unwrap();
        assert_ne!(projection.update_id, changed.update_id);
        assert_eq!(
            project_check(&terminal, &"x".repeat(p.bounds.title_bytes + 1), "", "", &[], &[], &p),
            Err(Error::Bounds)
        );
        let annotation = Annotation {
            path: "../secret".into(),
            line: 1,
            message: "bad".into(),
        };
        assert_eq!(project_check(&terminal, "", "", "", &[annotation], &[], &p), Err(Error::Bounds));
        let exact_title = "x".repeat(p.bounds.title_bytes);
        assert_eq!(project_check(&terminal, &exact_title, "", "", &[], &[], &p).unwrap().title, exact_title);
    }

    #[test]
    fn required_stage_fails_closed_for_every_prerequisite_and_moved_candidate() {
        let required = BTreeSet::from([id(2), id(3), id(4), id(5)]);
        let complete = RolloutEvidence {
            candidate: id(1),
            producer_prerequisites: required.clone(),
            offline: true,
            private: true,
            report_only: true,
            recovery: true,
            duplicate_events: true,
            cancellation: true,
            credential_redaction: true,
            cairn_gate: Some(id(6)),
        };
        assert_eq!(admit_stage(Stage::Required, id(1), &required, &complete), Ok(()));
        assert_eq!(admit_stage(Stage::Required, id(99), &required, &complete), Err(Error::Prerequisite));
        for field in 0..9 {
            let mut missing = complete.clone();
            match field {
                0 => missing.offline = false,
                1 => missing.private = false,
                2 => missing.report_only = false,
                3 => missing.recovery = false,
                4 => missing.duplicate_events = false,
                5 => missing.cancellation = false,
                6 => missing.credential_redaction = false,
                7 => missing.cairn_gate = None,
                _ => {
                    missing.producer_prerequisites.remove(&id(3));
                }
            }
            assert_eq!(admit_stage(Stage::Required, id(1), &required, &missing), Err(Error::Prerequisite));
        }
    }
}
