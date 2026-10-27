use std::collections::BTreeSet;

use animus_adapters::pijul::{
    PIJUL_CHANNEL_OUTPUT_FORMAT, PIJUL_CLI_LICENSE, PIJUL_CLI_RELEASE, PIJUL_COHORT_IDENTITY,
    PIJUL_CRATE_CHECKSUM_SHA256, PIJUL_DIFF_OUTPUT_FORMAT, PIJUL_LOG_OUTPUT_FORMAT,
    PIJUL_SOURCE_CHANGE, PIJUL_SOURCE_STATE, PijulOperation,
};
use serde::Deserialize;

use super::{
    CHANGE_HASH_CHARS, PROFILE_SCHEMA, PROJECTION_TARGETS, PijulValidationError,
    PijulValidationIssue, REQUIRED_NON_CLAIMS, STATE_ID_CHARS,
};

const QUERY_COUNT: usize = 3;
const MUTATION_COUNT: usize = 6;
const UNSUPPORTED_COUNT: usize = 1;
const SIGNATURE_STATUS_COUNT: usize = 4;
const RELEASE_SOURCE_REPOSITORY: &str = "https://nest.pijul.com/pijul/pijul";
const NIX_SOURCE_HASH: &str = "sha256-vU41JiuxB6Bsi88st/tkt02054oN3HEN52pnLu5hMA4=";
const NIX_OUTPUT_BASENAME: &str = "pijul-1.0.0-beta.18";
const CHANGE_HASH_FORMAT: &str = "blake3-base32-nopad-uppercase";
const STATE_ID_FORMAT: &str = "merkle-blake3-base32-nopad-uppercase";

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileInput {
    schema: String,
    enabled_by_default: bool,
    cohort: CohortInput,
    operations: OperationsInput,
    bounds: BoundsInput,
    channels: ChannelsInput,
    identity: IdentityInput,
    projections: Vec<String>,
    receipt: ReceiptInput,
    evidence: EvidenceInput,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CohortInput {
    release: String,
    source_repository: String,
    source_change: String,
    source_state: String,
    crate_checksum_sha256: String,
    nix_source_hash: String,
    nix_output_basename: String,
    change_hash_format: String,
    change_hash_chars: u64,
    state_id_format: String,
    state_id_chars: u64,
    log_output_format: String,
    diff_output_format: String,
    channel_output_format: String,
    license: String,
    cohort_identity: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OperationsInput {
    query_allowlist: Vec<String>,
    protected_mutations: Vec<String>,
    unsupported: Vec<String>,
    confirmation_required: bool,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BoundsInput {
    max_log_entries: u64,
    max_diff_records: u64,
    max_record_records: u64,
    max_path_bytes: u64,
    max_channel_bytes: u64,
    max_diagnostic_bytes: u64,
    max_stdout_bytes: u64,
    max_stderr_bytes: u64,
    max_stdin_bytes: u64,
    max_memory_bytes: u64,
    max_cpu_milliseconds: u64,
    deadline_milliseconds: u64,
    minimum_hash_prefix_chars: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ChannelsInput {
    allowlist: Vec<String>,
    allow_unlisted: bool,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(transparent)]
struct PolicyFlag(bool);

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct IdentityInput {
    load_local_identity_files: PolicyFlag,
    report_author_class: PolicyFlag,
    report_descriptive_timestamp: PolicyFlag,
    signature_statuses: Vec<String>,
    signature_authenticity_claim: PolicyFlag,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReceiptInput {
    repository_class: String,
    status: String,
    redaction_class: String,
    omission_class: String,
    truncation_class: String,
    projection_targets: Vec<String>,
    non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceInput {
    metadata_only: PolicyFlag,
    raw_diffs_allowed: PolicyFlag,
    file_bodies_allowed: PolicyFlag,
    change_descriptions_allowed: PolicyFlag,
    author_details_allowed: PolicyFlag,
    key_material_allowed: PolicyFlag,
    private_paths_allowed: PolicyFlag,
    credentials_allowed: PolicyFlag,
    unrestricted_diagnostics_allowed: PolicyFlag,
    non_claims: Vec<String>,
}

/// Named finite Pijul adapter bounds after validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PijulBounds {
    /// Maximum log records.
    pub max_log_entries: u64,
    /// Maximum diff status records.
    pub max_diff_records: u64,
    /// Maximum records described by a protected record plan.
    pub max_record_records: u64,
    /// Maximum bytes in one repository-relative path.
    pub max_path_bytes: u64,
    /// Maximum bytes in one channel name.
    pub max_channel_bytes: u64,
    /// Maximum bytes in one diagnostic field.
    pub max_diagnostic_bytes: u64,
    /// Maximum retained standard output.
    pub max_stdout_bytes: u64,
    /// Maximum retained standard error.
    pub max_stderr_bytes: u64,
    /// Maximum standard input.
    pub max_stdin_bytes: u64,
    /// Maximum resident memory bytes.
    pub max_memory_bytes: u64,
    /// Maximum CPU milliseconds.
    pub max_cpu_milliseconds: u64,
    /// Executor deadline in milliseconds.
    pub deadline_milliseconds: u64,
    /// Minimum admitted hash-prefix characters.
    pub minimum_hash_prefix_chars: u64,
}

/// Validated exact-cohort Pijul adapter profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PijulProfile {
    cohort_identity: String,
    policy_identity: String,
    query_allowlist: Vec<PijulOperation>,
    protected_mutations: Vec<PijulOperation>,
    unsupported: Vec<PijulOperation>,
    bounds: PijulBounds,
    channel_allowlist: Vec<String>,
    projections: Vec<String>,
}

impl PijulProfile {
    /// Returns the exact cohort identity.
    #[must_use]
    pub fn cohort_identity(&self) -> &str {
        self.cohort_identity.as_str()
    }

    /// Returns the BLAKE3 identity of the exact generated policy bytes.
    #[must_use]
    pub fn policy_identity(&self) -> &str {
        self.policy_identity.as_str()
    }

    /// Returns named finite bounds.
    #[must_use]
    pub const fn bounds(&self) -> PijulBounds {
        self.bounds
    }

    /// Returns admitted read-only operations.
    #[must_use]
    pub fn query_allowlist(&self) -> &[PijulOperation] {
        self.query_allowlist.as_slice()
    }

    /// Returns classified protected mutations. This adapter does not dispatch them.
    #[must_use]
    pub fn protected_mutations(&self) -> &[PijulOperation] {
        self.protected_mutations.as_slice()
    }

    /// Returns policy-declared unsupported operations.
    #[must_use]
    pub fn unsupported(&self) -> &[PijulOperation] {
        self.unsupported.as_slice()
    }

    /// Returns allowed channel names.
    #[must_use]
    pub fn channel_allowlist(&self) -> &[String] {
        self.channel_allowlist.as_slice()
    }

    /// Returns existing owner projection labels.
    #[must_use]
    pub fn projections(&self) -> &[String] {
        self.projections.as_slice()
    }
}

/// Parses generated JSON and revalidates the exact profile.
///
/// # Errors
///
/// Returns a safe error for malformed JSON, cohort drift, unknown operations,
/// missing bounds, widened content policy, or weakened non-claims.
pub fn parse_pijul_profile_json(input: &str) -> Result<PijulProfile, PijulValidationError> {
    let profile: ProfileInput = serde_json::from_str(input).map_err(|_error| {
        PijulValidationError::new(PijulValidationIssue::MalformedProfile, "profile")
    })?;
    let policy_identity = blake3::hash(input.as_bytes()).to_hex().to_string();
    validate_profile(profile, policy_identity)
}

fn validate_profile(
    input: ProfileInput,
    policy_identity: String,
) -> Result<PijulProfile, PijulValidationError> {
    validate_cohort(&input)?;
    let bounds = validate_bounds(input.bounds)?;
    let query_allowlist = parse_operations(&input.operations.query_allowlist)?;
    let protected_mutations = parse_operations(&input.operations.protected_mutations)?;
    let unsupported = parse_operations(&input.operations.unsupported)?;
    validate_operations(
        &input.operations,
        &query_allowlist,
        &protected_mutations,
        &unsupported,
    )?;
    validate_channels(&input.channels, bounds)?;
    validate_identity(&input.identity)?;
    validate_evidence(&input.evidence)?;
    validate_projections(&input.projections)?;
    validate_receipt(&input.receipt)?;
    Ok(PijulProfile {
        cohort_identity: input.cohort.cohort_identity,
        policy_identity,
        query_allowlist,
        protected_mutations,
        unsupported,
        bounds,
        channel_allowlist: input.channels.allowlist,
        projections: input.projections,
    })
}

fn validate_cohort(input: &ProfileInput) -> Result<(), PijulValidationError> {
    let cohort = &input.cohort;
    let change_chars = u64::try_from(CHANGE_HASH_CHARS).unwrap_or(u64::MAX);
    let state_chars = u64::try_from(STATE_ID_CHARS).unwrap_or(u64::MAX);
    let exact = input.schema == PROFILE_SCHEMA
        && !input.enabled_by_default
        && cohort.release == PIJUL_CLI_RELEASE
        && cohort.source_repository == RELEASE_SOURCE_REPOSITORY
        && cohort.source_change == PIJUL_SOURCE_CHANGE
        && cohort.source_state == PIJUL_SOURCE_STATE
        && cohort.crate_checksum_sha256 == PIJUL_CRATE_CHECKSUM_SHA256
        && cohort.nix_source_hash == NIX_SOURCE_HASH
        && cohort.nix_output_basename == NIX_OUTPUT_BASENAME
        && cohort.change_hash_format == CHANGE_HASH_FORMAT
        && cohort.change_hash_chars == change_chars
        && cohort.state_id_format == STATE_ID_FORMAT
        && cohort.state_id_chars == state_chars
        && cohort.log_output_format == PIJUL_LOG_OUTPUT_FORMAT
        && cohort.diff_output_format == PIJUL_DIFF_OUTPUT_FORMAT
        && cohort.channel_output_format == PIJUL_CHANNEL_OUTPUT_FORMAT
        && cohort.license == PIJUL_CLI_LICENSE
        && cohort.cohort_identity == PIJUL_COHORT_IDENTITY;
    if exact {
        Ok(())
    } else {
        Err(PijulValidationError::new(
            PijulValidationIssue::CohortMismatch,
            "cohort",
        ))
    }
}

fn validate_bounds(input: BoundsInput) -> Result<PijulBounds, PijulValidationError> {
    let values = [
        input.max_log_entries,
        input.max_diff_records,
        input.max_record_records,
        input.max_path_bytes,
        input.max_channel_bytes,
        input.max_diagnostic_bytes,
        input.max_stdout_bytes,
        input.max_stderr_bytes,
        input.max_stdin_bytes,
        input.max_memory_bytes,
        input.max_cpu_milliseconds,
        input.deadline_milliseconds,
        input.minimum_hash_prefix_chars,
    ];
    let full_hash_chars = u64::try_from(CHANGE_HASH_CHARS).unwrap_or(u64::MAX);
    if values.contains(&0)
        || input.minimum_hash_prefix_chars >= full_hash_chars
        || input.max_stderr_bytes > input.max_stdout_bytes
    {
        return Err(PijulValidationError::new(
            PijulValidationIssue::InvalidBound,
            "bounds",
        ));
    }
    Ok(PijulBounds {
        max_log_entries: input.max_log_entries,
        max_diff_records: input.max_diff_records,
        max_record_records: input.max_record_records,
        max_path_bytes: input.max_path_bytes,
        max_channel_bytes: input.max_channel_bytes,
        max_diagnostic_bytes: input.max_diagnostic_bytes,
        max_stdout_bytes: input.max_stdout_bytes,
        max_stderr_bytes: input.max_stderr_bytes,
        max_stdin_bytes: input.max_stdin_bytes,
        max_memory_bytes: input.max_memory_bytes,
        max_cpu_milliseconds: input.max_cpu_milliseconds,
        deadline_milliseconds: input.deadline_milliseconds,
        minimum_hash_prefix_chars: input.minimum_hash_prefix_chars,
    })
}

fn parse_operations(values: &[String]) -> Result<Vec<PijulOperation>, PijulValidationError> {
    let mut result = Vec::with_capacity(values.len());
    let mut unique = BTreeSet::new();
    for value in values {
        let operation = match value.as_str() {
            "log" => PijulOperation::Log,
            "diff" => PijulOperation::Diff,
            "channel_list" => PijulOperation::ChannelList,
            "change" => PijulOperation::Change,
            "record" => PijulOperation::Record,
            "unrecord" => PijulOperation::Unrecord,
            "apply" => PijulOperation::Apply,
            "fork" => PijulOperation::Fork,
            "reset" => PijulOperation::Reset,
            "channel_mutation" => PijulOperation::ChannelMutation,
            _ => {
                return Err(PijulValidationError::new(
                    PijulValidationIssue::UnsupportedOperation,
                    "operation",
                ));
            }
        };
        if !unique.insert(operation.as_str()) {
            return Err(PijulValidationError::new(
                PijulValidationIssue::UnsupportedOperation,
                "operation_duplicate",
            ));
        }
        result.push(operation);
    }
    Ok(result)
}

fn validate_operations(
    input: &OperationsInput,
    queries: &[PijulOperation],
    mutations: &[PijulOperation],
    unsupported: &[PijulOperation],
) -> Result<(), PijulValidationError> {
    let queries_exact = queries.len() == QUERY_COUNT
        && queries.contains(&PijulOperation::Log)
        && queries.contains(&PijulOperation::Diff)
        && queries.contains(&PijulOperation::ChannelList);
    let mutations_exact = mutations.len() == MUTATION_COUNT
        && mutations
            .iter()
            .all(|operation| operation.is_protected_mutation());
    let unsupported_exact =
        unsupported.len() == UNSUPPORTED_COUNT && unsupported.contains(&PijulOperation::Change);
    if input.confirmation_required && queries_exact && mutations_exact && unsupported_exact {
        Ok(())
    } else {
        Err(PijulValidationError::new(
            PijulValidationIssue::UnsupportedOperation,
            "operation_partition",
        ))
    }
}

fn validate_channels(
    input: &ChannelsInput,
    bounds: PijulBounds,
) -> Result<(), PijulValidationError> {
    if input.allow_unlisted || input.allowlist.is_empty() {
        return Err(PijulValidationError::new(
            PijulValidationIssue::InvalidChannel,
            "channel_policy",
        ));
    }
    let mut unique = BTreeSet::new();
    for channel in &input.allowlist {
        let bytes = u64::try_from(channel.len()).unwrap_or(u64::MAX);
        if channel.is_empty()
            || bytes > bounds.max_channel_bytes
            || !channel.bytes().all(is_safe_channel_byte)
        {
            return Err(PijulValidationError::new(
                PijulValidationIssue::InvalidChannel,
                "channel",
            ));
        }
        if !unique.insert(channel.as_str()) {
            return Err(PijulValidationError::new(
                PijulValidationIssue::InvalidChannel,
                "channel_duplicate",
            ));
        }
    }
    Ok(())
}

fn validate_identity(input: &IdentityInput) -> Result<(), PijulValidationError> {
    let statuses = input
        .signature_statuses
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let expected = [
        "not-reported",
        "present-unverified",
        "absent",
        "unsupported",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    if !input.load_local_identity_files.0
        && !input.report_author_class.0
        && !input.report_descriptive_timestamp.0
        && !input.signature_authenticity_claim.0
        && input.signature_statuses.len() == SIGNATURE_STATUS_COUNT
        && statuses == expected
    {
        Ok(())
    } else {
        Err(PijulValidationError::new(
            PijulValidationIssue::UnsafeEvidence,
            "identity_policy",
        ))
    }
}

fn validate_evidence(input: &EvidenceInput) -> Result<(), PijulValidationError> {
    let excluded = !input.raw_diffs_allowed.0
        && !input.file_bodies_allowed.0
        && !input.change_descriptions_allowed.0
        && !input.author_details_allowed.0
        && !input.key_material_allowed.0
        && !input.private_paths_allowed.0
        && !input.credentials_allowed.0
        && !input.unrestricted_diagnostics_allowed.0;
    let required = REQUIRED_NON_CLAIMS
        .iter()
        .all(|required| input.non_claims.iter().any(|actual| actual == required));
    if input.metadata_only.0
        && excluded
        && required
        && input.non_claims.len() == REQUIRED_NON_CLAIMS.len()
    {
        Ok(())
    } else {
        Err(PijulValidationError::new(
            PijulValidationIssue::UnsafeEvidence,
            "evidence_policy",
        ))
    }
}

fn validate_receipt(input: &ReceiptInput) -> Result<(), PijulValidationError> {
    let non_claims_exact = input.non_claims.len() == REQUIRED_NON_CLAIMS.len()
        && REQUIRED_NON_CLAIMS
            .iter()
            .all(|required| input.non_claims.iter().any(|actual| actual == required));
    let exact = input.repository_class == "explicit-local"
        && input.status == "validated"
        && input.redaction_class == "metadata-only"
        && input.omission_class == "content-omitted"
        && input.truncation_class == "not-truncated"
        && validate_projections(&input.projection_targets).is_ok()
        && non_claims_exact;
    if exact {
        Ok(())
    } else {
        Err(PijulValidationError::new(
            PijulValidationIssue::UnsafeEvidence,
            "receipt_policy",
        ))
    }
}

fn validate_projections(values: &[String]) -> Result<(), PijulValidationError> {
    let actual = values.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = PROJECTION_TARGETS.iter().copied().collect::<BTreeSet<_>>();
    if values.len() == PROJECTION_TARGETS.len() && actual == expected {
        Ok(())
    } else {
        Err(PijulValidationError::new(
            PijulValidationIssue::ProjectionRejected,
            "projection_targets",
        ))
    }
}

fn is_safe_channel_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.')
}
