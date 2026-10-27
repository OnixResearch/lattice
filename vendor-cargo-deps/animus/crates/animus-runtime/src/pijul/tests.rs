use animus_adapters::CheckpointBackendKind;
use animus_adapters::pijul::{
    PIJUL_COHORT_IDENTITY, PijulBackendResponse, PijulConfirmation, PijulExecutionOutcome,
    PijulExecutionStatus, PijulLoadedChannel, PijulLoadedDiffEntry, PijulLoadedLogEntry,
    PijulLoadedOutput, PijulOperation, PijulRequest,
};

use super::*;

const PROFILE_JSON: &str = include_str!("../../../../config/generated/pijul-vcs-adapter.json");
const REPOSITORY_IDENTITY: &str =
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const EXECUTOR_IDENTITY: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const CHANGE_HASH: &str = "QRYOAJLYA5KO7JAYU6LMYHPU3NSZCTUMYEPCK3HNZIRIDZDWSMBAC";
const SECOND_CHANGE_HASH: &str = "QRYOAJLYA5KO7JAYU6LMYHPU3NSZCTUMYEPCK3HNZIRIDZDWSMBA2";
const STATE_ID: &str = "RISR562S7FZP3NNYDFVBQUMDEGB6YSSMSH2RGZDRYBIPGVQDSLXQC";
const ZERO_EXIT_CODE: i32 = 0;
const POLICY_HASH_CHARS: usize = 64;
const QUERY_OPERATION_COUNT: usize = 3;
const PROTECTED_MUTATION_COUNT: usize = 6;
const PROJECTION_COUNT: usize = 4;
const REQUIRED_NON_CLAIM_COUNT: usize = 12;
const DIFF_RECORD_COUNT: u32 = 2;

fn profile() -> PijulProfile {
    parse_pijul_profile_json(PROFILE_JSON).expect("generated profile must validate")
}

fn request(operation: PijulOperation) -> PijulRequest {
    PijulRequest {
        operation,
        repository_path: "/explicit/repository".to_owned(),
        repository_identity: REPOSITORY_IDENTITY.to_owned(),
        channel: Some("main".to_owned()),
        change_selector: None,
        confirmation: PijulConfirmation::NotRequired,
    }
}

fn execution() -> PijulExecutionOutcome {
    PijulExecutionOutcome {
        status: PijulExecutionStatus::Exited,
        exit_code: Some(ZERO_EXIT_CODE),
        standard_output: Vec::new(),
        standard_error: Vec::new(),
        stdout_truncated: false,
        stderr_truncated: false,
    }
}

fn log_entry(hash: &str) -> PijulLoadedLogEntry {
    PijulLoadedLogEntry {
        hash: hash.to_owned(),
        state: None,
        authors: Vec::new(),
        timestamp: None,
        message: None,
        description: None,
        paths: Vec::new(),
    }
}

fn response(operation: PijulOperation, loaded: PijulLoadedOutput) -> PijulBackendResponse {
    PijulBackendResponse {
        operation,
        execution: execution(),
        loaded: Some(loaded),
    }
}

#[test]
fn generated_profile_is_exact_and_non_default() {
    let profile = profile();
    assert_eq!(profile.cohort_identity(), PIJUL_COHORT_IDENTITY);
    assert_eq!(profile.policy_identity().len(), POLICY_HASH_CHARS);
    assert_eq!(profile.query_allowlist().len(), QUERY_OPERATION_COUNT);
    assert_eq!(
        profile.protected_mutations().len(),
        PROTECTED_MUTATION_COUNT
    );
    assert_eq!(profile.unsupported(), [PijulOperation::Change]);
    assert_eq!(profile.projections().len(), PROJECTION_COUNT);
}

#[test]
fn cohort_drift_is_rejected() {
    let drifted = PROFILE_JSON.replace("1.0.0-beta.18", "1.0.0-beta.19");
    let error = parse_pijul_profile_json(drifted.as_str()).expect_err("cohort drift must fail");
    assert_eq!(error.issue(), PijulValidationIssue::CohortMismatch);
}

#[test]
fn widened_evidence_policy_is_rejected() {
    let widened = PROFILE_JSON.replace(
        "\"raw_diffs_allowed\": false",
        "\"raw_diffs_allowed\": true",
    );
    let error = parse_pijul_profile_json(widened.as_str()).expect_err("content widening must fail");
    assert_eq!(error.issue(), PijulValidationIssue::UnsafeEvidence);
}

#[test]
fn protected_mutation_is_denied_before_execution() {
    let mut protected = request(PijulOperation::Record);
    protected.confirmation = PijulConfirmation::Confirmed;
    let error =
        validate_pijul_request(&profile(), &protected).expect_err("mutation dispatch is absent");
    assert_eq!(error.issue(), PijulValidationIssue::ProtectedMutationDenied);
}

#[test]
fn valid_log_projects_into_existing_owners_and_metadata_receipt() {
    let profile = profile();
    let request = request(PijulOperation::Log);
    let response = response(
        PijulOperation::Log,
        PijulLoadedOutput::Log(vec![log_entry(CHANGE_HASH)]),
    );
    let validated = validate_pijul_response(&profile, &request, &response)
        .expect("reviewed log output must validate");
    let bundle =
        project_pijul_log(&profile, &request, &validated).expect("validated log must project");
    let receipt = project_pijul_receipt(
        &profile,
        &request,
        &validated,
        PijulReceiptContext {
            executor_identity: EXECUTOR_IDENTITY,
        },
    )
    .expect("safe receipt must build");
    assert_eq!(
        bundle.checkpoint.backend_kind(),
        CheckpointBackendKind::Other
    );
    assert_eq!(
        bundle.snapshot.backend(),
        animus_adapters::SnapshotBackendKind::Other
    );
    assert_eq!(bundle.workspace.source_id(), REPOSITORY_IDENTITY);
    assert_eq!(receipt.record_count, 1);
    assert_eq!(receipt.file_count, 0);
    assert_eq!(receipt.repository_class, "explicit-local");
    assert_eq!(receipt.redaction_class, "metadata-only");
    assert_eq!(receipt.omission_class, "content-omitted");
    assert_eq!(receipt.truncation_class, "not-truncated");
    assert_eq!(receipt.non_claims.len(), REQUIRED_NON_CLAIM_COUNT);
    let debug = format!("{receipt:?}");
    assert!(!debug.contains("/explicit/repository"));
    assert!(!debug.contains("descriptive-author"));
}

#[test]
fn descriptive_log_output_that_can_read_identity_files_is_rejected() {
    let profile = profile();
    let request = request(PijulOperation::Log);
    let mut entry = log_entry(CHANGE_HASH);
    entry.state = Some(STATE_ID.to_owned());
    entry.authors = vec!["descriptive-author".to_owned()];
    let response = response(PijulOperation::Log, PijulLoadedOutput::Log(vec![entry]));
    let error = validate_pijul_response(&profile, &request, &response)
        .expect_err("rich log output exceeds the initial CLI authority boundary");
    assert_eq!(error.issue(), PijulValidationIssue::UnsafeEvidence);
}

#[test]
fn malformed_state_identity_is_rejected_before_rich_output_denial() {
    let profile = profile();
    let request = request(PijulOperation::Log);
    let mut entry = log_entry(CHANGE_HASH);
    entry.state = Some("malformed-state".to_owned());
    let response = response(PijulOperation::Log, PijulLoadedOutput::Log(vec![entry]));
    let error = validate_pijul_response(&profile, &request, &response)
        .expect_err("malformed state identity must fail");
    assert_eq!(error.issue(), PijulValidationIssue::InvalidIdentity);
}

#[test]
fn malformed_hash_is_rejected() {
    let profile = profile();
    let request = request(PijulOperation::Log);
    let response = response(
        PijulOperation::Log,
        PijulLoadedOutput::Log(vec![log_entry("lowercase-is-not-a-pijul-change-hash")]),
    );
    let error = validate_pijul_response(&profile, &request, &response)
        .expect_err("malformed change hash must fail");
    assert_eq!(error.issue(), PijulValidationIssue::InvalidIdentity);
}

#[test]
fn ambiguous_prefix_is_rejected() {
    let profile = profile();
    let mut request = request(PijulOperation::Log);
    request.change_selector = Some("QRYOAJLY".to_owned());
    let response = response(
        PijulOperation::Log,
        PijulLoadedOutput::Log(vec![log_entry(CHANGE_HASH), log_entry(SECOND_CHANGE_HASH)]),
    );
    let error = validate_pijul_response(&profile, &request, &response)
        .expect_err("ambiguous prefix must fail");
    assert_eq!(error.issue(), PijulValidationIssue::AmbiguousPrefix);
}

#[test]
fn unsafe_diff_path_is_rejected() {
    let profile = profile();
    let request = request(PijulOperation::Diff);
    let response = response(
        PijulOperation::Diff,
        PijulLoadedOutput::Diff(vec![PijulLoadedDiffEntry {
            path: "../private".to_owned(),
            operation: "edit".to_owned(),
            line: Some(1),
        }]),
    );
    let error =
        validate_pijul_response(&profile, &request, &response).expect_err("unsafe path must fail");
    assert_eq!(error.issue(), PijulValidationIssue::UnsafeEvidence);
}

#[test]
fn valid_diff_redacts_paths_and_counts_conflicts() {
    let profile = profile();
    let request = request(PijulOperation::Diff);
    let response = response(
        PijulOperation::Diff,
        PijulLoadedOutput::Diff(vec![
            PijulLoadedDiffEntry {
                path: "src/lib.rs".to_owned(),
                operation: "edit".to_owned(),
                line: Some(1),
            },
            PijulLoadedDiffEntry {
                path: "src/main.rs".to_owned(),
                operation: "solve order conflict".to_owned(),
                line: None,
            },
        ]),
    );
    let validated =
        validate_pijul_response(&profile, &request, &response).expect("bounded diff must validate");
    assert_eq!(validated.record_count(), DIFF_RECORD_COUNT);
    assert_eq!(validated.conflict_count(), 1);
    assert!(!format!("{validated:?}").contains("src/lib.rs"));
}

#[test]
fn prompt_like_error_output_is_rejected() {
    let profile = profile();
    let request = request(PijulOperation::ChannelList);
    let mut response = response(
        PijulOperation::ChannelList,
        PijulLoadedOutput::Channels(vec![PijulLoadedChannel {
            name: "main".to_owned(),
            is_current: true,
        }]),
    );
    response.execution.standard_error = b"Password:".to_vec();
    let error = validate_pijul_response(&profile, &request, &response)
        .expect_err("interactive prompt must fail");
    assert_eq!(error.issue(), PijulValidationIssue::InteractiveOutput);
}

#[test]
fn over_limit_log_collection_is_rejected_before_record_validation() {
    let profile = profile();
    let request = request(PijulOperation::Log);
    let over_limit_count = profile.bounds().max_log_entries.saturating_add(1);
    let entries = (0..over_limit_count)
        .map(|_index| log_entry(CHANGE_HASH))
        .collect();
    let response = response(PijulOperation::Log, PijulLoadedOutput::Log(entries));
    let error = validate_pijul_response(&profile, &request, &response)
        .expect_err("over-limit log output must fail");
    assert_eq!(error.issue(), PijulValidationIssue::BoundExceeded);
}

#[test]
fn output_truncation_is_rejected() {
    let profile = profile();
    let request = request(PijulOperation::ChannelList);
    let mut response = response(
        PijulOperation::ChannelList,
        PijulLoadedOutput::Channels(Vec::new()),
    );
    response.execution.stdout_truncated = true;
    let error = validate_pijul_response(&profile, &request, &response)
        .expect_err("truncated output must fail");
    assert_eq!(error.issue(), PijulValidationIssue::BoundExceeded);
}

#[test]
fn unlisted_channel_is_rejected() {
    let profile = profile();
    let request = request(PijulOperation::ChannelList);
    let response = response(
        PijulOperation::ChannelList,
        PijulLoadedOutput::Channels(vec![PijulLoadedChannel {
            name: "private".to_owned(),
            is_current: true,
        }]),
    );
    let error = validate_pijul_response(&profile, &request, &response)
        .expect_err("unlisted channel must fail");
    assert_eq!(error.issue(), PijulValidationIssue::InvalidChannel);
}
