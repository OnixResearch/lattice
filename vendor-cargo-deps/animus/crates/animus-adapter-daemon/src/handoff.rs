//! Versioned transactional daemon-handoff DTOs and implementation-free ports.
//!
//! These contracts carry bounded metadata only. Implementations own ingress,
//! storage, leases, local authentication, processes, deadlines, teardown, and
//! worker exit. The contracts do not grant those effects.

type String = alloc::string::String;

/// Transactional handoff DTO schema.
pub const HANDOFF_SCHEMA: &str = "animus.daemon-handoff.v1";
/// Supported local handoff protocol version.
pub const HANDOFF_PROTOCOL_VERSION: u16 = 1;
/// Maximum bytes in one handoff label.
pub const MAX_HANDOFF_LABEL_BYTES: usize = 256;
/// Minimum valid generation.
pub const MIN_HANDOFF_GENERATION: u64 = 1;

/// A bounded DTO validation issue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HandoffDtoIssue {
    /// Schema or protocol version is unsupported.
    UnsupportedVersion,
    /// A label is empty, unsafe, or overbound.
    InvalidLabel,
    /// A required generation is zero.
    InvalidGeneration,
    /// A process-tree count is inconsistent.
    InvalidProcessTree,
}

/// A bounded host service issue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HandoffServiceError {
    /// The required injected service is unavailable.
    Unavailable,
    /// Current host policy denied the request.
    Denied,
    /// The host effect failed.
    Failed,
    /// A bounded readiness or termination wait elapsed.
    TimedOut,
    /// The host returned malformed or unbound facts.
    Malformed,
}

/// Request to pause every ingress class for one handoff.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffIngressRequest {
    /// DTO schema.
    pub schema: String,
    /// Handoff identity.
    pub handoff: String,
    /// Old worker identity.
    pub worker: String,
    /// Session identity.
    pub session: String,
    /// Primary queue generation.
    pub primary_queue_generation: u64,
    /// Child queue generation.
    pub child_queue_generation: u64,
    /// Steering queue generation.
    pub steering_generation: u64,
}

/// Result of one exact handoff gate or host action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HandoffCheckState {
    /// The gate or action passed.
    Passed,
    /// The gate or action failed.
    Failed,
}

/// Observation after ingress pause was requested.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffIngressObservation {
    /// Handoff identity.
    pub handoff: String,
    /// Observed primary queue generation.
    pub primary_queue_generation: u64,
    /// Observed child queue generation.
    pub child_queue_generation: u64,
    /// Observed steering queue generation.
    pub steering_generation: u64,
    /// Prompt-ingress pause result.
    pub prompt_pause: HandoffCheckState,
    /// Steering-ingress pause result.
    pub steering_pause: HandoffCheckState,
    /// Follow-up-ingress pause result.
    pub follow_up_pause: HandoffCheckState,
    /// External-message-ingress pause result.
    pub external_pause: HandoffCheckState,
}

impl HandoffIngressObservation {
    /// Returns whether every required ingress class is paused.
    #[must_use]
    pub const fn all_paused(&self) -> bool {
        matches!(self.prompt_pause, HandoffCheckState::Passed)
            && matches!(self.steering_pause, HandoffCheckState::Passed)
            && matches!(self.follow_up_pause, HandoffCheckState::Passed)
            && matches!(self.external_pause, HandoffCheckState::Passed)
    }
}

/// Request to resume ingress after a later old-worker lease is restored.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffIngressResumeRequest {
    /// DTO schema.
    pub schema: String,
    /// Handoff identity.
    pub handoff: String,
    /// Old worker identity.
    pub worker: String,
    /// Restored lease generation.
    pub lease_generation: u64,
}

/// Observation after ingress resume was requested.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffIngressResumeObservation {
    /// Handoff identity.
    pub handoff: String,
    /// Restored lease generation.
    pub lease_generation: u64,
    /// Resume result.
    pub resumed: HandoffCheckState,
    /// Safe resume observation identity.
    pub observation: String,
}

/// Durable session-state request committed before lease release.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffCommitRequest {
    /// DTO schema.
    pub schema: String,
    /// Handoff identity.
    pub handoff: String,
    /// Session identity.
    pub session: String,
    /// Exact session-head identity.
    pub session_head: String,
    /// Composer-draft identity. No draft body is included.
    pub composer_draft: String,
    /// Pending-input vault generation.
    pub vault_generation: u64,
    /// Active-turn generation.
    pub active_turn_generation: u64,
    /// Authority-profile identity.
    pub authority_profile: String,
    /// Handoff-policy identity.
    pub policy: String,
}

/// Observation for one durable handoff-record commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffCommitObservation {
    /// Handoff identity.
    pub handoff: String,
    /// Durable commit identity.
    pub commit: String,
    /// Whether the exact record was committed.
    pub committed: bool,
}

/// Lease operation class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HandoffLeaseOperation {
    /// Release the old-worker lease.
    Release,
    /// Restore a later old-worker lease after cancellation.
    Restore,
}

/// Request for one exact lease operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffLeaseRequest {
    /// DTO schema.
    pub schema: String,
    /// Handoff identity.
    pub handoff: String,
    /// Session identity.
    pub session: String,
    /// Worker identity.
    pub worker: String,
    /// Lease operation.
    pub operation: HandoffLeaseOperation,
    /// Expected lease generation.
    pub generation: u64,
    /// Durable commit or cancellation identity authorizing the operation.
    pub authorization: String,
}

/// Observation for one lease operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffLeaseObservation {
    /// Handoff identity.
    pub handoff: String,
    /// Lease operation.
    pub operation: HandoffLeaseOperation,
    /// Observed generation.
    pub generation: u64,
    /// Whether the operation completed.
    pub completed: bool,
    /// Safe host observation identity.
    pub observation: String,
}

/// Request to start one owned replacement process tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffProcessStartRequest {
    /// DTO schema.
    pub schema: String,
    /// Handoff identity.
    pub handoff: String,
    /// Session identity.
    pub session: String,
    /// Released lease generation.
    pub released_generation: u64,
    /// Host-selected bounded deadline class.
    pub deadline_class: String,
}

/// Observation after replacement process start.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffProcessStartObservation {
    /// Handoff identity.
    pub handoff: String,
    /// Process-attempt identity.
    pub process_attempt: String,
    /// Replacement leader process identity.
    pub process: String,
    /// Authenticated local endpoint identity.
    pub endpoint: String,
    /// Whether process-tree ownership was established.
    pub started: bool,
}

/// Request to wait for and authenticate one replacement claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffClaimRequest {
    /// DTO schema.
    pub schema: String,
    /// Handoff identity.
    pub handoff: String,
    /// Session identity.
    pub session: String,
    /// Released lease generation.
    pub released_generation: u64,
    /// Process-attempt identity.
    pub process_attempt: String,
    /// Replacement process identity.
    pub process: String,
    /// Endpoint identity.
    pub endpoint: String,
    /// Supported protocol version.
    pub protocol_version: u16,
}

/// Authenticated replacement claim observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffClaimObservation {
    /// DTO schema.
    pub schema: String,
    /// Handoff identity.
    pub handoff: String,
    /// Session identity.
    pub session: String,
    /// Released lease generation.
    pub released_generation: u64,
    /// Process-attempt identity.
    pub process_attempt: String,
    /// Replacement process identity.
    pub process: String,
    /// Endpoint identity.
    pub endpoint: String,
    /// Claimant identity.
    pub claimant: String,
    /// Protocol version sent by the claimant.
    pub protocol_version: u16,
    /// Handoff-local freshness nonce.
    pub freshness_nonce: String,
    /// Whether local endpoint authentication passed.
    pub authenticated: bool,
    /// Whether authentication is current for this claim.
    pub current: bool,
    /// Safe authentication observation identity.
    pub authentication_observation: String,
}

/// Request to wait for exact replacement readiness.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffReadinessRequest {
    /// DTO schema.
    pub schema: String,
    /// Handoff identity.
    pub handoff: String,
    /// Claim identity.
    pub claim: String,
    /// Exact session head.
    pub session_head: String,
    /// Composer-draft identity.
    pub composer_draft: String,
    /// Pending-input vault generation.
    pub vault_generation: u64,
    /// Active-turn generation.
    pub active_turn_generation: u64,
    /// Authority-profile identity.
    pub authority_profile: String,
    /// Handoff-policy identity.
    pub policy: String,
}

/// Replacement readiness and durable-record observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffReadinessObservation {
    /// Handoff identity.
    pub handoff: String,
    /// Claim identity.
    pub claim: String,
    /// Loaded session-head identity.
    pub session_head: String,
    /// Visible composer-draft identity.
    pub composer_draft: String,
    /// Visible pending-input vault generation.
    pub vault_generation: u64,
    /// Admitted active-turn generation.
    pub active_turn_generation: u64,
    /// Admitted authority-profile identity.
    pub authority_profile: String,
    /// Admitted handoff-policy identity.
    pub policy: String,
    /// Service-readiness identity.
    pub services: String,
    /// Exact durable session-load result.
    pub session_load: HandoffCheckState,
    /// Pending-input vault-visibility result.
    pub vault_visibility: HandoffCheckState,
    /// Current authority-admission result.
    pub authority_admission: HandoffCheckState,
    /// Active-turn recovery-admission result.
    pub recovery_admission: HandoffCheckState,
    /// Required service-readiness result.
    pub service_readiness: HandoffCheckState,
    /// Matching readiness-record commit result.
    pub readiness_commit: HandoffCheckState,
}

/// Request to durably commit cancellation before termination.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffCancellationRequest {
    /// DTO schema.
    pub schema: String,
    /// Handoff identity.
    pub handoff: String,
    /// Prior transition identity.
    pub prior_transition: String,
    /// Optional process-attempt identity.
    pub process_attempt: Option<String>,
    /// Stable cancellation cause.
    pub cause: String,
    /// Canonical cancellation identity.
    pub cancellation: String,
}

/// Durable cancellation commit observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffCancellationObservation {
    /// Cancellation identity.
    pub cancellation: String,
    /// Whether cancelled state was committed.
    pub committed: bool,
    /// Safe commit observation identity.
    pub observation: String,
}

/// Request to terminate or observe one owned replacement process tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffTerminationRequest {
    /// DTO schema.
    pub schema: String,
    /// Handoff identity.
    pub handoff: String,
    /// Optional process-attempt identity. None means spawn never established ownership.
    pub process_attempt: Option<String>,
    /// Canonical cancellation identity.
    pub cancellation: String,
}

/// Complete process-tree termination observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffTerminationObservation {
    /// Optional process-attempt identity.
    pub process_attempt: Option<String>,
    /// Whether the leader is terminal.
    pub leader_terminal: bool,
    /// Whether every owned descendant is terminal.
    pub descendants_terminal: bool,
    /// Number of owned processes.
    pub owned_processes: u32,
    /// Number of terminal owned processes.
    pub terminal_processes: u32,
    /// Number of live owned descendants.
    pub live_descendants: u32,
}

/// Request to revalidate old-worker authority and durable store state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffRollbackRequest {
    /// DTO schema.
    pub schema: String,
    /// Handoff identity.
    pub handoff: String,
    /// Old worker identity.
    pub old_worker: String,
    /// Expected authority-profile identity.
    pub authority_profile: String,
    /// Expected session-head identity.
    pub session_head: String,
    /// Expected next lease generation.
    pub next_generation: u64,
}

/// Current rollback revalidation observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffRollbackObservation {
    /// Handoff identity.
    pub handoff: String,
    /// Old worker identity.
    pub old_worker: String,
    /// Current authority-profile identity.
    pub authority_profile: String,
    /// Current session-head identity.
    pub session_head: String,
    /// Expected next lease generation.
    pub next_generation: u64,
    /// Whether authority is current.
    pub authority_current: bool,
    /// Whether the durable store head is current.
    pub store_head_current: bool,
    /// Safe revalidation observation identity.
    pub observation: String,
}

/// Request to exit the old worker after readiness commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffExitRequest {
    /// DTO schema.
    pub schema: String,
    /// Handoff identity.
    pub handoff: String,
    /// Old worker identity.
    pub old_worker: String,
    /// Replacement readiness identity.
    pub readiness: String,
}

/// Old-worker exit observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffExitObservation {
    /// Old worker identity.
    pub old_worker: String,
    /// Replacement readiness identity.
    pub readiness: String,
    /// Whether the old worker exited.
    pub exited: bool,
    /// Safe exit observation identity.
    pub observation: String,
}

/// Injected ingress-pause port.
pub trait HandoffIngressPort {
    /// Pauses prompt, steer, follow-up, and external-message ingress.
    ///
    /// # Errors
    ///
    /// Returns a bounded service issue without guessing pause state.
    fn pause_ingress(
        &mut self,
        request: &HandoffIngressRequest,
    ) -> Result<HandoffIngressObservation, HandoffServiceError>;

    /// Resumes ingress after a later old-worker lease is restored.
    ///
    /// # Errors
    ///
    /// Returns a bounded service issue without claiming ingress availability.
    fn resume_ingress(
        &mut self,
        request: &HandoffIngressResumeRequest,
    ) -> Result<HandoffIngressResumeObservation, HandoffServiceError>;
}

/// Injected durable handoff-record port.
pub trait HandoffStatePort {
    /// Commits exact session input state before lease release.
    ///
    /// # Errors
    ///
    /// Returns a bounded service issue without claiming durability.
    fn commit_handoff(
        &mut self,
        request: &HandoffCommitRequest,
    ) -> Result<HandoffCommitObservation, HandoffServiceError>;

    /// Commits cancellation before process termination.
    ///
    /// # Errors
    ///
    /// Returns a bounded service issue without issuing termination.
    fn commit_cancellation(
        &mut self,
        request: &HandoffCancellationRequest,
    ) -> Result<HandoffCancellationObservation, HandoffServiceError>;

    /// Revalidates old-worker authority and durable session state before rollback.
    ///
    /// # Errors
    ///
    /// Returns a bounded service issue without restoring a lease.
    fn revalidate_rollback(
        &mut self,
        request: &HandoffRollbackRequest,
    ) -> Result<HandoffRollbackObservation, HandoffServiceError>;
}

/// Injected session-lease port.
pub trait HandoffLeasePort {
    /// Releases or restores one exact lease generation.
    ///
    /// # Errors
    ///
    /// Returns a bounded service issue without inventing ownership.
    fn update_lease(
        &mut self,
        request: &HandoffLeaseRequest,
    ) -> Result<HandoffLeaseObservation, HandoffServiceError>;
}

/// Injected replacement process-tree port.
pub trait HandoffProcessPort {
    /// Starts one replacement process tree.
    ///
    /// # Errors
    ///
    /// Returns a bounded service issue without claiming tree ownership.
    fn start_replacement(
        &mut self,
        request: &HandoffProcessStartRequest,
    ) -> Result<HandoffProcessStartObservation, HandoffServiceError>;

    /// Requests owned process-tree termination after cancellation commit.
    ///
    /// # Errors
    ///
    /// Returns a bounded service issue while lease restoration remains blocked.
    fn terminate_replacement(
        &mut self,
        request: &HandoffTerminationRequest,
    ) -> Result<(), HandoffServiceError>;

    /// Observes complete owned process-tree termination.
    ///
    /// # Errors
    ///
    /// Returns a bounded service issue without inferring descendant exit.
    fn observe_termination(
        &mut self,
        request: &HandoffTerminationRequest,
    ) -> Result<HandoffTerminationObservation, HandoffServiceError>;
}

/// Injected local endpoint and authentication port.
pub trait HandoffEndpointPort {
    /// Waits for one claim and returns current local authentication facts.
    ///
    /// # Errors
    ///
    /// Returns a bounded service issue without admitting the claimant.
    fn authenticate_claim(
        &mut self,
        request: &HandoffClaimRequest,
    ) -> Result<HandoffClaimObservation, HandoffServiceError>;
}

/// Injected replacement-readiness port.
pub trait HandoffReadinessPort {
    /// Waits for exact durable session, vault, authority, recovery, and service readiness.
    ///
    /// # Errors
    ///
    /// Returns a bounded service issue without authorizing old-worker exit.
    fn wait_readiness(
        &mut self,
        request: &HandoffReadinessRequest,
    ) -> Result<HandoffReadinessObservation, HandoffServiceError>;
}

/// Injected old-worker exit port.
pub trait HandoffExitPort {
    /// Exits the old worker after exact readiness commit.
    ///
    /// # Errors
    ///
    /// Returns a bounded service issue without claiming completion.
    fn exit_old_worker(
        &mut self,
        request: &HandoffExitRequest,
    ) -> Result<HandoffExitObservation, HandoffServiceError>;
}

/// Composite service boundary for one transactional daemon handoff.
pub trait HandoffHostServices:
    HandoffIngressPort
    + HandoffStatePort
    + HandoffLeasePort
    + HandoffProcessPort
    + HandoffEndpointPort
    + HandoffReadinessPort
    + HandoffExitPort
{
}

impl<T> HandoffHostServices for T where
    T: HandoffIngressPort
        + HandoffStatePort
        + HandoffLeasePort
        + HandoffProcessPort
        + HandoffEndpointPort
        + HandoffReadinessPort
        + HandoffExitPort
{
}

/// Validates a replacement claim observation after decoding.
///
/// # Errors
///
/// Returns a bounded version, label, or generation issue.
pub fn validate_handoff_claim_observation(
    observation: &HandoffClaimObservation,
) -> Result<(), HandoffDtoIssue> {
    validate_schema(&observation.schema)?;
    validate_generation(observation.released_generation)?;
    for value in [
        &observation.handoff,
        &observation.session,
        &observation.process_attempt,
        &observation.process,
        &observation.endpoint,
        &observation.claimant,
        &observation.freshness_nonce,
        &observation.authentication_observation,
    ] {
        validate_label(value)?;
    }
    if observation.protocol_version != HANDOFF_PROTOCOL_VERSION {
        return Err(HandoffDtoIssue::UnsupportedVersion);
    }
    Ok(())
}

/// Validates a readiness observation after decoding.
///
/// # Errors
///
/// Returns a bounded label or generation issue.
pub fn validate_handoff_readiness_observation(
    observation: &HandoffReadinessObservation,
) -> Result<(), HandoffDtoIssue> {
    validate_generation(observation.vault_generation)?;
    validate_generation(observation.active_turn_generation)?;
    for value in [
        &observation.handoff,
        &observation.claim,
        &observation.session_head,
        &observation.composer_draft,
        &observation.authority_profile,
        &observation.policy,
        &observation.services,
    ] {
        validate_label(value)?;
    }
    Ok(())
}

/// Validates process-tree termination counts after decoding.
///
/// # Errors
///
/// Returns a bounded label or inconsistent-count issue.
pub fn validate_handoff_termination_observation(
    observation: &HandoffTerminationObservation,
) -> Result<(), HandoffDtoIssue> {
    if let Some(attempt) = &observation.process_attempt {
        validate_label(attempt)?;
    }
    if observation.terminal_processes > observation.owned_processes
        || observation.live_descendants > observation.owned_processes
    {
        return Err(HandoffDtoIssue::InvalidProcessTree);
    }
    Ok(())
}

/// Validates one handoff DTO label.
///
/// # Errors
///
/// Returns `InvalidLabel` for empty, unsafe, or overbound text.
pub fn validate_handoff_label(value: &str) -> Result<(), HandoffDtoIssue> {
    validate_label(value)
}

fn validate_schema(schema: &str) -> Result<(), HandoffDtoIssue> {
    if schema == HANDOFF_SCHEMA {
        return Ok(());
    }
    Err(HandoffDtoIssue::UnsupportedVersion)
}

fn validate_generation(value: u64) -> Result<(), HandoffDtoIssue> {
    if value >= MIN_HANDOFF_GENERATION {
        return Ok(());
    }
    Err(HandoffDtoIssue::InvalidGeneration)
}

fn validate_label(value: &str) -> Result<(), HandoffDtoIssue> {
    let valid = !value.is_empty()
        && value.len() <= MAX_HANDOFF_LABEL_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':'));
    if valid {
        return Ok(());
    }
    Err(HandoffDtoIssue::InvalidLabel)
}

#[cfg(test)]
mod tests {
    use super::*;

    const GENERATION: u64 = 7;
    const LABEL_OVERFLOW: usize = MAX_HANDOFF_LABEL_BYTES.saturating_add(1);
    const ONE_PROCESS: u32 = 1;

    fn claim() -> HandoffClaimObservation {
        HandoffClaimObservation {
            schema: HANDOFF_SCHEMA.into(),
            handoff: "handoff.identity".into(),
            session: "session.identity".into(),
            released_generation: GENERATION,
            process_attempt: "process.attempt".into(),
            process: "process.identity".into(),
            endpoint: "endpoint.identity".into(),
            claimant: "claimant.identity".into(),
            protocol_version: HANDOFF_PROTOCOL_VERSION,
            freshness_nonce: "nonce.identity".into(),
            authenticated: true,
            current: true,
            authentication_observation: "authentication.identity".into(),
        }
    }

    #[test]
    fn current_claim_and_complete_tree_validate() {
        let claim = claim();
        validate_handoff_claim_observation(&claim).expect("claim DTO");
        let termination = HandoffTerminationObservation {
            process_attempt: Some("process.attempt".into()),
            leader_terminal: true,
            descendants_terminal: true,
            owned_processes: ONE_PROCESS,
            terminal_processes: ONE_PROCESS,
            live_descendants: 0,
        };
        validate_handoff_termination_observation(&termination).expect("termination DTO");
    }

    #[test]
    fn stale_protocol_unsafe_label_and_bad_counts_fail_closed() {
        let mut stale = claim();
        stale.protocol_version = HANDOFF_PROTOCOL_VERSION.saturating_add(1);
        assert_eq!(
            validate_handoff_claim_observation(&stale),
            Err(HandoffDtoIssue::UnsupportedVersion)
        );
        let mut unsafe_claim = claim();
        unsafe_claim.endpoint = "/private/socket".into();
        assert_eq!(
            validate_handoff_claim_observation(&unsafe_claim),
            Err(HandoffDtoIssue::InvalidLabel)
        );
        let mut oversized_claim = claim();
        oversized_claim.freshness_nonce = "x".repeat(LABEL_OVERFLOW);
        assert_eq!(
            validate_handoff_claim_observation(&oversized_claim),
            Err(HandoffDtoIssue::InvalidLabel)
        );
        let mut wrong_schema = claim();
        wrong_schema.schema = "animus.daemon-handoff.v2".into();
        assert_eq!(
            validate_handoff_claim_observation(&wrong_schema),
            Err(HandoffDtoIssue::UnsupportedVersion)
        );
        let malformed = HandoffTerminationObservation {
            process_attempt: Some("process.attempt".into()),
            leader_terminal: true,
            descendants_terminal: false,
            owned_processes: 0,
            terminal_processes: ONE_PROCESS,
            live_descendants: ONE_PROCESS,
        };
        assert_eq!(
            validate_handoff_termination_observation(&malformed),
            Err(HandoffDtoIssue::InvalidProcessTree)
        );
    }
}
