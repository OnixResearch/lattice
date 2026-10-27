//! Pure autonomous continuation and quality-gate contracts.
//!
//! Hosts execute gate descriptors, read clocks, and calculate workspace
//! fingerprints. This module consumes typed observations only. It does not
//! execute commands, inspect files, read clocks, or persist state.

/// Autonomous-gates policy schema.
pub const AUTONOMOUS_GATES_SCHEMA: &str = "animus.autonomous-gates.v1";
const SCHEMA: &str = AUTONOMOUS_GATES_SCHEMA;
/// BLAKE3 domain for admitted file-set fingerprints.
pub const FINGERPRINT_DOMAIN: &str = "animus.autonomous-gates.fingerprint.v1";
/// BLAKE3 domain for autonomous receipt identities.
pub const RECEIPT_DOMAIN: &str = "animus.autonomous-gates.receipt.v1";
const POLICY_DOMAIN: &str = "animus.autonomous-gates.policy.v1";
const HASH_HEX_LENGTH: usize = 64;
const MAX_LABEL_LENGTH: usize = 128;
const MAX_NON_CLAIMS: usize = 32;
const MAX_OUTPUT_BYTES: usize = 1_048_576;
const MIN_OUTPUT_BYTES: usize = 1;
const OUTPUT_HALVES: usize = 2;
const FILE_FACT_FIELD_COUNT: usize = 2;
const DEFAULT_MAX_CONTINUATIONS: u32 = 8;
const DEFAULT_MAX_TURNS: u32 = 32;
const DEFAULT_MAX_TOKENS: u64 = 100_000;
const DEFAULT_MAX_WALL_CLOCK_MILLISECONDS: u64 = 3_600_000;
const DEFAULT_MAX_GATE_OUTPUT_BYTES: usize = 4_096;
const MAX_CONTINUATIONS: u32 = 1_000;
const MAX_TURNS: u32 = 10_000;
const MAX_TOKENS: u64 = 100_000_000;
const MAX_WALL_CLOCK_MILLISECONDS: u64 = 86_400_000;
const HEAD_TAIL_SEPARATOR: &str = "\n...[bounded gate output]...\n";
const HASH_SEPARATOR: &str = "\u{1f}";
const FIELD_POLICY: &str = "policy";
const FIELD_GATE: &str = "gate";
const FIELD_FINGERPRINT: &str = "fingerprint";
const FIELD_HASH: &str = "hash";
const NON_CLAIM_TASK_SUCCESS: &str = "not-task-success";
const NON_CLAIM_COMPLETION: &str = "not-objective-completion";
const NON_CLAIM_SCOPE: &str = "not-verification-outside-gate-scope";
const NON_CLAIM_COVERAGE: &str = "not-change-detection-outside-admitted-file-set";
const NON_CLAIM_SANDBOX: &str = "not-gate-sandboxing";
const NON_CLAIM_PRODUCTION: &str = "not-production-readiness";
const NON_CLAIM_RELEASE: &str = "not-release-eligibility";
const NON_CLAIM_AUTHORITY: &str = "not-authority-widening";
const REQUIRED_NON_CLAIMS: [&str; 8] = [
    NON_CLAIM_TASK_SUCCESS,
    NON_CLAIM_COMPLETION,
    NON_CLAIM_SCOPE,
    NON_CLAIM_COVERAGE,
    NON_CLAIM_SANDBOX,
    NON_CLAIM_PRODUCTION,
    NON_CLAIM_RELEASE,
    NON_CLAIM_AUTHORITY,
];
const LEAK_MARKERS: [&str; 11] = [
    "raw_gate_output:",
    "prompt:",
    "transcript:",
    "credential:",
    "api_key",
    "secret=",
    "provider_payload:",
    "environment:",
    "/home/",
    "/tmp/",
    "unbounded",
];

/// Stable classes of autonomous limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LimitClass {
    /// Maximum follow-up continuation count.
    Continuations,
    /// Maximum runtime turn count.
    Turns,
    /// Maximum admitted token count.
    Tokens,
    /// Maximum host-supplied elapsed wall-clock duration.
    WallClock,
}

impl LimitClass {
    /// Returns the stable receipt label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Continuations => "continuations",
            Self::Turns => "turns",
            Self::Tokens => "tokens",
            Self::WallClock => "wall_clock",
        }
    }
}

/// Explicit autonomous execution bounds.
#[allow(
    clippy::struct_field_names,
    reason = "the max prefix makes each public policy ceiling explicit to callers"
)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AutonomousLimits {
    /// Maximum continuation count.
    pub max_continuations: u32,
    /// Maximum turn count.
    pub max_turns: u32,
    /// Maximum token count.
    pub max_tokens: u64,
    /// Maximum elapsed wall-clock milliseconds supplied by the host.
    pub max_wall_clock_milliseconds: u64,
}

/// Host-supplied counters at one autonomous decision point.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AutonomousCounters {
    /// Continuations already consumed.
    pub continuations: u32,
    /// Turns already consumed.
    pub turns: u32,
    /// Tokens already consumed.
    pub tokens: u64,
    /// Elapsed wall-clock milliseconds.
    pub wall_clock_milliseconds: u64,
}

/// Policy input exported from Nickel before Rust validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutonomousPolicyInput {
    /// Policy schema.
    pub schema: String,
    /// Safe policy identifier.
    pub policy_id: String,
    /// Explicit limits.
    pub limits: AutonomousLimits,
    /// Maximum failed-gate output bytes.
    pub max_gate_output_bytes: usize,
    /// Receipt field labels.
    pub receipt_fields: Vec<String>,
    /// Required non-claims.
    pub non_claims: Vec<String>,
    /// Source policy hash.
    pub policy_hash: String,
}

/// Strict autonomous policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutonomousPolicy {
    input: AutonomousPolicyInput,
    identity: String,
}

impl AutonomousPolicy {
    /// Returns the domain-separated policy identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }

    /// Returns the configured limits.
    #[must_use]
    pub const fn limits(&self) -> AutonomousLimits {
        self.input.limits
    }

    /// Returns the failed-gate output bound.
    #[must_use]
    pub const fn max_gate_output_bytes(&self) -> usize {
        self.input.max_gate_output_bytes
    }

    /// Returns the required receipt fields.
    #[must_use]
    pub fn receipt_fields(&self) -> &[String] {
        self.input.receipt_fields.as_slice()
    }

    /// Returns the required non-claims.
    #[must_use]
    pub fn non_claims(&self) -> &[String] {
        self.input.non_claims.as_slice()
    }
}

/// Builds and validates an autonomous policy.
///
/// # Errors
///
/// Returns [`AutonomousError`] when the exported policy is malformed or unsafe.
pub fn validate_policy(input: &AutonomousPolicyInput) -> Result<AutonomousPolicy, AutonomousError> {
    if input.schema != SCHEMA
        || input.limits.max_continuations == 0
        || u64::from(input.limits.max_continuations) > u64::from(MAX_CONTINUATIONS)
        || input.limits.max_turns == 0
        || input.limits.max_turns > MAX_TURNS
        || input.limits.max_tokens == 0
        || input.limits.max_tokens > MAX_TOKENS
        || input.limits.max_wall_clock_milliseconds == 0
        || input.limits.max_wall_clock_milliseconds > MAX_WALL_CLOCK_MILLISECONDS
        || input.max_gate_output_bytes < MIN_OUTPUT_BYTES
        || input.max_gate_output_bytes > MAX_OUTPUT_BYTES
    {
        return Err(AutonomousError::InvalidPolicy);
    }
    validate_label(FIELD_POLICY, input.policy_id.as_str())?;
    validate_hash(FIELD_POLICY, input.policy_hash.as_str())?;
    validate_required_labels(input.receipt_fields.as_slice(), FIELD_POLICY)?;
    validate_non_claims(input.non_claims.as_slice())?;
    let identity = hash_parts(
        POLICY_DOMAIN,
        &[
            input.schema.as_str(),
            input.policy_id.as_str(),
            input.limits.max_continuations.to_string().as_str(),
            input.limits.max_turns.to_string().as_str(),
            input.limits.max_tokens.to_string().as_str(),
            input
                .limits
                .max_wall_clock_milliseconds
                .to_string()
                .as_str(),
            input.max_gate_output_bytes.to_string().as_str(),
            input.policy_hash.as_str(),
        ],
    );
    Ok(AutonomousPolicy {
        input: input.clone(),
        identity,
    })
}

/// Returns the checked default policy fixture used by focused tests.
///
/// # Panics
///
/// Panics only when repository-owned default constants drift from the policy contract.
#[must_use]
pub fn default_policy() -> AutonomousPolicy {
    validate_policy(&AutonomousPolicyInput {
        schema: SCHEMA.to_string(),
        policy_id: "autonomous-default".to_string(),
        limits: AutonomousLimits {
            max_continuations: DEFAULT_MAX_CONTINUATIONS,
            max_turns: DEFAULT_MAX_TURNS,
            max_tokens: DEFAULT_MAX_TOKENS,
            max_wall_clock_milliseconds: DEFAULT_MAX_WALL_CLOCK_MILLISECONDS,
        },
        max_gate_output_bytes: DEFAULT_MAX_GATE_OUTPUT_BYTES,
        receipt_fields: vec![
            "policy_identity".to_string(),
            "gate_identity".to_string(),
            "verified_scope".to_string(),
            "outcome_class".to_string(),
            "fingerprint_identity".to_string(),
            "limit_class".to_string(),
        ],
        non_claims: REQUIRED_NON_CLAIMS
            .iter()
            .map(ToString::to_string)
            .collect(),
        policy_hash: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
    })
    .expect("default autonomous policy is valid")
}

/// Gate command classes admitted by policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GateCommandClass {
    /// A static or dynamic analysis check.
    Check,
    /// A test command.
    Test,
    /// A lint command.
    Lint,
}

impl GateCommandClass {
    /// Returns the stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Check => "check",
            Self::Test => "test",
            Self::Lint => "lint",
        }
    }
}

/// Host-executed gate descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GateDescriptor {
    /// Opaque gate identity.
    pub gate_id: String,
    /// Host command class.
    pub command_class: GateCommandClass,
    /// Exact scope that the gate checks.
    pub verified_scope: String,
    /// Maximum output bytes admitted to the prompt.
    pub max_output_bytes: usize,
}

/// Validates a gate descriptor without executing it.
///
/// # Errors
///
/// Returns [`AutonomousError`] when the descriptor contains unsafe metadata.
pub fn validate_gate_descriptor(
    policy: &AutonomousPolicy,
    descriptor: &GateDescriptor,
) -> Result<(), AutonomousError> {
    validate_label(FIELD_GATE, descriptor.gate_id.as_str())?;
    validate_label("verified_scope", descriptor.verified_scope.as_str())?;
    if descriptor.max_output_bytes == 0
        || descriptor.max_output_bytes > policy.max_gate_output_bytes()
    {
        return Err(AutonomousError::InvalidGate);
    }
    Ok(())
}

/// Safe output classification for a failed gate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GateOutputClass {
    /// Output came from the gate's standard error stream.
    Stderr,
    /// Output represents a non-zero exit result.
    Exit,
    /// The host deadline elapsed.
    Timeout,
}

impl GateOutputClass {
    /// Returns the stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stderr => "stderr",
            Self::Exit => "exit",
            Self::Timeout => "timeout",
        }
    }
}

/// Bounded failed-gate output projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedGateOutput {
    /// Safe output class.
    pub class: GateOutputClass,
    /// Bounded text supplied to the next prompt.
    pub text: String,
    /// Original byte count, without retaining the original output.
    pub original_byte_count: usize,
    /// Whether the projection dropped bytes.
    pub truncated: bool,
}

/// Projects host output to the policy bound using head and tail preservation.
///
/// # Errors
///
/// Returns [`AutonomousError`] for an invalid output bound or unsafe output.
pub fn bound_gate_output(
    output_class: GateOutputClass,
    output: &str,
    max_bytes: usize,
) -> Result<BoundedGateOutput, AutonomousError> {
    if !(MIN_OUTPUT_BYTES..=MAX_OUTPUT_BYTES).contains(&max_bytes) {
        return Err(AutonomousError::InvalidOutput);
    }
    if contains_leak_marker(output) {
        return Err(AutonomousError::UnsafeOutput);
    }
    let original_byte_count = output.len();
    if original_byte_count <= max_bytes {
        return Ok(BoundedGateOutput {
            class: output_class,
            text: output.to_string(),
            original_byte_count,
            truncated: false,
        });
    }
    let separator_bytes = HEAD_TAIL_SEPARATOR.len();
    let available = max_bytes.saturating_sub(separator_bytes);
    let head_bytes = available / OUTPUT_HALVES;
    let tail_bytes = available.saturating_sub(head_bytes);
    let head = prefix_at_boundary(output, head_bytes);
    let tail_start = output.len().saturating_sub(tail_bytes);
    let tail = suffix_at_boundary(output, tail_start);
    let text = format!("{head}{HEAD_TAIL_SEPARATOR}{tail}");
    Ok(BoundedGateOutput {
        class: output_class,
        text,
        original_byte_count,
        truncated: true,
    })
}

/// Host result classes consumed by the pure contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GateOutcome {
    /// The host reported success.
    Passed,
    /// The host reported a bounded failure.
    Failed(BoundedGateOutput),
    /// The host deadline elapsed.
    TimedOut,
}

impl GateOutcome {
    /// Returns the stable outcome label.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed(_) => "failed",
            Self::TimedOut => "timeout",
        }
    }
}

/// Admitted logical file fact supplied by a host fingerprint service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedFileFact {
    /// Repository-relative logical path.
    pub logical_path: String,
    /// BLAKE3 content identity.
    pub content_hash: String,
}

/// Fingerprint over an explicit admitted file set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceFingerprint {
    /// BLAKE3 identity of the normalized admitted file facts.
    pub identity: String,
    /// BLAKE3 identity of the admitted input-set description.
    pub input_identity: String,
}

/// Computes a deterministic fingerprint over the supplied admitted file set.
///
/// Files outside `files` are not covered by this function.
///
/// # Errors
///
/// Returns [`AutonomousError`] for duplicate paths, unsafe paths, or malformed hashes.
pub fn fingerprint_admitted_file_set(
    mut files: Vec<AdmittedFileFact>,
) -> Result<WorkspaceFingerprint, AutonomousError> {
    files.sort_by(|left, right| left.logical_path.cmp(&right.logical_path));
    let mut parts = Vec::with_capacity(files.len() * FILE_FACT_FIELD_COUNT);
    for file in &files {
        validate_relative_path(file.logical_path.as_str())?;
        validate_hash(FIELD_HASH, file.content_hash.as_str())?;
        parts.push(file.logical_path.as_str());
        parts.push(file.content_hash.as_str());
    }
    for pair in files.windows(FILE_FACT_FIELD_COUNT) {
        if pair[0].logical_path == pair[1].logical_path {
            return Err(AutonomousError::DuplicatePath);
        }
    }
    let input_identity = hash_parts(FINGERPRINT_DOMAIN, &parts);
    let identity = hash_parts(
        FINGERPRINT_DOMAIN,
        &[input_identity.as_str(), "admitted-file-set"],
    );
    Ok(WorkspaceFingerprint {
        identity,
        input_identity,
    })
}

/// Recorded gate attempt used for deterministic rerun decisions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GateAttemptRecord {
    /// Gate identity.
    pub gate_id: String,
    /// Workspace fingerprint identity.
    pub fingerprint_identity: String,
    /// Recorded outcome.
    pub outcome: GateOutcome,
}

/// Decision for a gate rerun.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GateRerunDecision {
    /// The host can execute the gate.
    Allowed,
    /// A failed attempt exists for the unchanged workspace.
    DeniedUnchangedFailure,
}

/// Denies unchanged failed-gate reruns.
///
/// # Errors
///
/// Returns [`AutonomousError`] for an invalid descriptor or fingerprint.
pub fn gate_rerun_decision(
    policy: &AutonomousPolicy,
    descriptor: &GateDescriptor,
    fingerprint: &WorkspaceFingerprint,
    history: &[GateAttemptRecord],
) -> Result<GateRerunDecision, AutonomousError> {
    validate_gate_descriptor(policy, descriptor)?;
    validate_hash(FIELD_FINGERPRINT, fingerprint.identity.as_str())?;
    for attempt in history {
        if attempt.gate_id == descriptor.gate_id
            && attempt.fingerprint_identity == fingerprint.identity
            && matches!(
                attempt.outcome,
                GateOutcome::Failed(_) | GateOutcome::TimedOut
            )
        {
            return Ok(GateRerunDecision::DeniedUnchangedFailure);
        }
    }
    Ok(GateRerunDecision::Allowed)
}

/// Autonomous continuation decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AutonomousOutcome {
    /// Continue because gates still need work or a failed gate may rerun.
    Continue,
    /// All admitted gates passed, without a task-success claim.
    GatesPassed,
    /// A named limit stopped continuation, without a task-success claim.
    LimitReached(LimitClass),
}

impl AutonomousOutcome {
    /// Returns the stable outcome label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Continue => "continue",
            Self::GatesPassed => "gates_passed",
            Self::LimitReached(_) => "limit_reached",
        }
    }
}

/// Decides whether autonomous continuation can proceed.
#[must_use]
pub fn decide_continuation(
    limits: AutonomousLimits,
    counters: AutonomousCounters,
    all_gates_passed: bool,
) -> AutonomousOutcome {
    if counters.continuations >= limits.max_continuations {
        return AutonomousOutcome::LimitReached(LimitClass::Continuations);
    }
    if counters.turns >= limits.max_turns {
        return AutonomousOutcome::LimitReached(LimitClass::Turns);
    }
    if counters.tokens >= limits.max_tokens {
        return AutonomousOutcome::LimitReached(LimitClass::Tokens);
    }
    if counters.wall_clock_milliseconds >= limits.max_wall_clock_milliseconds {
        return AutonomousOutcome::LimitReached(LimitClass::WallClock);
    }
    if all_gates_passed {
        AutonomousOutcome::GatesPassed
    } else {
        AutonomousOutcome::Continue
    }
}

/// Safe metadata-only autonomous receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutonomousReceipt {
    /// Policy identity.
    pub policy_identity: String,
    /// Gate identity.
    pub gate_identity: String,
    /// Verified gate scope.
    pub verified_scope: String,
    /// Workspace fingerprint identity.
    pub fingerprint_identity: String,
    /// Outcome class.
    pub outcome_class: String,
    /// Optional limit class.
    pub limit_class: Option<String>,
    /// Required non-claims.
    pub non_claims: Vec<String>,
    /// Receipt identity.
    pub receipt_identity: String,
}

/// Projects a gate or continuation observation into a safe receipt.
///
/// # Errors
///
/// Returns [`AutonomousError`] when the receipt would contain a leak or an
/// invalid success projection.
pub fn project_receipt(
    policy: &AutonomousPolicy,
    descriptor: &GateDescriptor,
    fingerprint: &WorkspaceFingerprint,
    outcome: AutonomousOutcome,
) -> Result<AutonomousReceipt, AutonomousError> {
    validate_gate_descriptor(policy, descriptor)?;
    validate_hash(FIELD_FINGERPRINT, fingerprint.identity.as_str())?;
    if matches!(outcome, AutonomousOutcome::Continue) && descriptor.verified_scope.is_empty() {
        return Err(AutonomousError::InvalidReceipt);
    }
    let limit_class = match outcome {
        AutonomousOutcome::LimitReached(limit) => Some(limit.as_str().to_string()),
        AutonomousOutcome::Continue | AutonomousOutcome::GatesPassed => None,
    };
    let outcome_class = outcome.as_str().to_string();
    let receipt_identity = hash_parts(
        RECEIPT_DOMAIN,
        &[
            policy.identity(),
            descriptor.gate_id.as_str(),
            descriptor.verified_scope.as_str(),
            fingerprint.identity.as_str(),
            outcome_class.as_str(),
            limit_class.as_deref().unwrap_or("none"),
        ],
    );
    Ok(AutonomousReceipt {
        policy_identity: policy.identity().to_string(),
        gate_identity: descriptor.gate_id.clone(),
        verified_scope: descriptor.verified_scope.clone(),
        fingerprint_identity: fingerprint.identity.clone(),
        outcome_class,
        limit_class,
        non_claims: policy.non_claims().to_vec(),
        receipt_identity,
    })
}

/// Errors returned by autonomous pure-core operations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AutonomousError {
    /// Policy input is invalid.
    InvalidPolicy,
    /// Gate descriptor is invalid.
    InvalidGate,
    /// Output is invalid or exceeds the requested bound.
    InvalidOutput,
    /// Output contains a prohibited material marker.
    UnsafeOutput,
    /// A label is empty, too long, or unsafe.
    UnsafeLabel,
    /// A hash is not lowercase BLAKE3 hexadecimal.
    InvalidHash,
    /// A path is not a repository-relative logical path.
    UnsafePath,
    /// A path occurs more than once.
    DuplicatePath,
    /// A receipt input is invalid.
    InvalidReceipt,
}

fn validate_label(_field: &str, value: &str) -> Result<(), AutonomousError> {
    if value.is_empty()
        || value.len() > MAX_LABEL_LENGTH
        || contains_leak_marker(value)
        || value.chars().any(char::is_control)
    {
        return Err(AutonomousError::UnsafeLabel);
    }
    Ok(())
}

fn validate_required_labels(values: &[String], field: &str) -> Result<(), AutonomousError> {
    if values.is_empty()
        || values
            .iter()
            .any(|value| validate_label(field, value.as_str()).is_err())
    {
        return Err(AutonomousError::InvalidPolicy);
    }
    Ok(())
}

fn validate_non_claims(values: &[String]) -> Result<(), AutonomousError> {
    if values.len() > MAX_NON_CLAIMS
        || REQUIRED_NON_CLAIMS
            .iter()
            .any(|required| !values.iter().any(|value| value == required))
    {
        return Err(AutonomousError::InvalidPolicy);
    }
    validate_required_labels(values, "non_claims")
}

fn validate_hash(_field: &str, value: &str) -> Result<(), AutonomousError> {
    if value.len() != HASH_HEX_LENGTH
        || !value
            .chars()
            .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase())
    {
        return Err(AutonomousError::InvalidHash);
    }
    Ok(())
}

fn validate_relative_path(value: &str) -> Result<(), AutonomousError> {
    if value.is_empty()
        || value.starts_with('/')
        || value.starts_with("../")
        || value.contains("/../")
        || value == ".."
        || value.contains('\0')
    {
        return Err(AutonomousError::UnsafePath);
    }
    Ok(())
}

fn contains_leak_marker(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    LEAK_MARKERS.iter().any(|marker| lowered.contains(marker))
}

fn prefix_at_boundary(value: &str, requested_bytes: usize) -> &str {
    let mut end = requested_bytes.min(value.len());
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    &value[..end]
}

fn suffix_at_boundary(value: &str, requested_start: usize) -> &str {
    let mut start = requested_start.min(value.len());
    while start < value.len() && !value.is_char_boundary(start) {
        start += 1;
    }
    &value[start..]
}

fn hash_parts(domain: &str, parts: &[&str]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    for part in parts {
        hasher.update(HASH_SEPARATOR.as_bytes());
        hasher.update(part.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_HASH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const SMALL_OUTPUT_BOUND: usize = 32;
    const LARGE_OUTPUT_BOUND: usize = 128;
    const ONE_CONTINUATION: u32 = 1;
    const ONE_TURN: u32 = 1;
    const ONE_TOKEN: u64 = 1;
    const ONE_MILLISECOND: u64 = 1;
    const TWO: u32 = 2;
    const TWO_U64: u64 = 2;
    const THREE: u64 = 3;
    const FOUR: u64 = 4;

    fn descriptor() -> GateDescriptor {
        GateDescriptor {
            gate_id: "workspace-tests".to_string(),
            command_class: GateCommandClass::Test,
            verified_scope: "workspace-tests".to_string(),
            max_output_bytes: LARGE_OUTPUT_BOUND,
        }
    }

    fn fingerprint() -> WorkspaceFingerprint {
        fingerprint_admitted_file_set(vec![AdmittedFileFact {
            logical_path: "crates/animus-runtime/src/lib.rs".to_string(),
            content_hash: VALID_HASH.to_string(),
        }])
        .expect("fingerprint")
    }

    #[test]
    fn policy_and_gate_are_valid() {
        let policy = default_policy();
        assert!(validate_gate_descriptor(&policy, &descriptor()).is_ok());
        assert_eq!(policy.non_claims().len(), REQUIRED_NON_CLAIMS.len());
    }

    #[test]
    fn failed_output_preserves_bounds_and_classes() {
        let output = "0123456789abcdefghijklmnopqrstuvwxyz";
        let bounded = bound_gate_output(GateOutputClass::Exit, output, SMALL_OUTPUT_BOUND)
            .expect("bounded output");
        assert!(bounded.truncated);
        assert!(bounded.text.len() <= SMALL_OUTPUT_BOUND);
        assert_eq!(bounded.class, GateOutputClass::Exit);
        assert_eq!(bounded.original_byte_count, output.len());
    }

    #[test]
    fn changed_workspace_allows_failed_gate_rerun() {
        let policy = default_policy();
        let first = fingerprint();
        let second = fingerprint_admitted_file_set(vec![AdmittedFileFact {
            logical_path: "crates/animus-runtime/src/lib.rs".to_string(),
            content_hash: "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                .to_string(),
        }])
        .expect("changed fingerprint");
        let history = vec![GateAttemptRecord {
            gate_id: descriptor().gate_id.clone(),
            fingerprint_identity: first.identity,
            outcome: GateOutcome::TimedOut,
        }];
        assert_eq!(
            gate_rerun_decision(&policy, &descriptor(), &second, &history).expect("rerun decision"),
            GateRerunDecision::Allowed
        );
    }

    #[test]
    fn unchanged_failed_gate_is_denied() {
        let policy = default_policy();
        let current = fingerprint();
        let history = vec![GateAttemptRecord {
            gate_id: descriptor().gate_id.clone(),
            fingerprint_identity: current.identity.clone(),
            outcome: GateOutcome::Failed(
                bound_gate_output(GateOutputClass::Stderr, "failed", LARGE_OUTPUT_BOUND)
                    .expect("bounded failure"),
            ),
        }];
        assert_eq!(
            gate_rerun_decision(&policy, &descriptor(), &current, &history)
                .expect("rerun decision"),
            GateRerunDecision::DeniedUnchangedFailure
        );
    }

    #[test]
    fn limits_are_not_success() {
        let limits = AutonomousLimits {
            max_continuations: ONE_CONTINUATION,
            max_turns: TWO,
            max_tokens: THREE,
            max_wall_clock_milliseconds: FOUR,
        };
        let outcome = decide_continuation(
            limits,
            AutonomousCounters {
                continuations: ONE_CONTINUATION,
                turns: 0,
                tokens: 0,
                wall_clock_milliseconds: 0,
            },
            true,
        );
        assert_eq!(
            outcome,
            AutonomousOutcome::LimitReached(LimitClass::Continuations)
        );
        assert_ne!(outcome, AutonomousOutcome::GatesPassed);
    }

    #[test]
    fn gates_passed_is_scoped_and_not_success() {
        let outcome = decide_continuation(
            AutonomousLimits {
                max_continuations: TWO,
                max_turns: TWO,
                max_tokens: TWO_U64,
                max_wall_clock_milliseconds: TWO_U64,
            },
            AutonomousCounters {
                continuations: 0,
                turns: 0,
                tokens: 0,
                wall_clock_milliseconds: 0,
            },
            true,
        );
        assert_eq!(outcome, AutonomousOutcome::GatesPassed);
        let receipt = project_receipt(&default_policy(), &descriptor(), &fingerprint(), outcome)
            .expect("receipt");
        assert_eq!(receipt.outcome_class, "gates_passed");
        assert!(
            receipt
                .non_claims
                .contains(&NON_CLAIM_TASK_SUCCESS.to_string())
        );
    }

    #[test]
    fn unsafe_inputs_are_rejected() {
        let policy = default_policy();
        assert_eq!(
            bound_gate_output(
                GateOutputClass::Stderr,
                "raw_gate_output: secret",
                LARGE_OUTPUT_BOUND
            )
            .expect_err("leak must fail"),
            AutonomousError::UnsafeOutput
        );
        assert_eq!(
            fingerprint_admitted_file_set(vec![AdmittedFileFact {
                logical_path: "../outside".to_string(),
                content_hash: VALID_HASH.to_string(),
            }])
            .expect_err("path must fail"),
            AutonomousError::UnsafePath
        );
        assert_eq!(
            validate_gate_descriptor(
                &policy,
                &GateDescriptor {
                    max_output_bytes: policy.max_gate_output_bytes() + 1,
                    ..descriptor()
                },
            )
            .expect_err("output bound must fail"),
            AutonomousError::InvalidGate
        );
    }

    #[test]
    fn fingerprint_is_order_independent_and_duplicate_paths_fail() {
        let first = fingerprint_admitted_file_set(vec![
            AdmittedFileFact {
                logical_path: "b".to_string(),
                content_hash: VALID_HASH.to_string(),
            },
            AdmittedFileFact {
                logical_path: "a".to_string(),
                content_hash: VALID_HASH.to_string(),
            },
        ])
        .expect("first fingerprint");
        let second = fingerprint_admitted_file_set(vec![
            AdmittedFileFact {
                logical_path: "a".to_string(),
                content_hash: VALID_HASH.to_string(),
            },
            AdmittedFileFact {
                logical_path: "b".to_string(),
                content_hash: VALID_HASH.to_string(),
            },
        ])
        .expect("second fingerprint");
        assert_eq!(first, second);
        assert_eq!(
            fingerprint_admitted_file_set(vec![
                AdmittedFileFact {
                    logical_path: "a".to_string(),
                    content_hash: VALID_HASH.to_string(),
                },
                AdmittedFileFact {
                    logical_path: "a".to_string(),
                    content_hash: VALID_HASH.to_string(),
                },
            ])
            .expect_err("duplicate path must fail"),
            AutonomousError::DuplicatePath
        );
    }

    #[test]
    fn receipt_is_metadata_only() {
        let receipt = project_receipt(
            &default_policy(),
            &descriptor(),
            &fingerprint(),
            AutonomousOutcome::LimitReached(LimitClass::WallClock),
        )
        .expect("receipt");
        let debug = format!("{receipt:?}");
        assert!(!debug.contains("crates/animus-runtime/src/lib.rs"));
        assert!(!debug.contains("raw_gate_output"));
        assert_eq!(receipt.limit_class.as_deref(), Some("wall_clock"));
    }

    #[test]
    fn generated_nickel_policy_matches_rust_defaults() {
        let value: serde_json::Value =
            serde_json::from_str(GENERATED_POLICY_JSON).expect("generated policy JSON");
        assert_eq!(value["schema"], AUTONOMOUS_GATES_SCHEMA);
        assert_eq!(value["policy_id"], "autonomous-default");
        assert_eq!(
            value["limits"]["max_continuations"],
            DEFAULT_MAX_CONTINUATIONS
        );
        assert_eq!(value["limits"]["max_turns"], DEFAULT_MAX_TURNS);
        assert_eq!(value["limits"]["max_tokens"], DEFAULT_MAX_TOKENS);
        assert_eq!(
            value["limits"]["max_wall_clock_milliseconds"],
            DEFAULT_MAX_WALL_CLOCK_MILLISECONDS
        );
        assert_eq!(
            value["max_gate_output_bytes"],
            DEFAULT_MAX_GATE_OUTPUT_BYTES
        );
        assert_eq!(
            value["non_claims"].as_array().map(Vec::len),
            Some(REQUIRED_NON_CLAIMS.len())
        );
    }

    const GENERATED_POLICY_JSON: &str =
        include_str!("../../../../config/generated/autonomous-gates-policy.json");

    #[allow(dead_code)]
    const _BOUNDARY_COUNTERS: AutonomousCounters = AutonomousCounters {
        continuations: ONE_CONTINUATION,
        turns: ONE_TURN,
        tokens: ONE_TOKEN,
        wall_clock_milliseconds: ONE_MILLISECOND,
    };
}
