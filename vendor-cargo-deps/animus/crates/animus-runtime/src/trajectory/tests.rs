use animus_adapters::trajectory::{
    DiagnosticDisposition, TRAJECTORY_HELPER_BLAKE3, TRAJECTORY_SCHEMA_BLAKE3, TrajectoryBounds,
    TrajectoryContentDisposition, TrajectoryDiagnostic, TrajectoryDiagnosticCode,
    TrajectoryDiagnosticPolicy, TrajectoryNormalizeRequest, TrajectoryNormalizeResult,
    TrajectoryPolicy, TrajectoryRecord, TrajectorySource, TrajectoryToolCall,
};

use super::{
    TrajectoryIssue, TrajectoryOmission, TrajectoryReceiptInput, TrajectoryReceiptSensitiveDraft,
    project_trajectory, trajectory_receipt, validate_policy, validate_trajectory,
};

const SOURCE_GENERATION: &str = "generation:fixture:one";
const SERVICE_IDENTITY: &str = "service:fixture:one";
const TIMESTAMP_ONE: &str = "2026-08-09T10:00:00Z";
const TIMESTAMP_TWO: &str = "2026-08-09T10:00:01Z";
const TIMESTAMP_THREE: &str = "2026-08-09T10:00:02Z";
const TOOL_CALL_ID: &str = "call:one";
const EXPECTED_LEDGER_ENTRY_COUNT: usize = 2;
const REQUIRED_NON_CLAIM_COUNT: usize = 9;
const BLAKE3_HEX_LENGTH: usize = 64;
const SYNTHESIZED_TIMESTAMP_COUNT: u32 = 2;
const RESTRICTED_RECORD_MAX: usize = 2;
const FIXTURE_MAX_TRANSCRIPT_BYTES: usize = 1_048_576;
const FIXTURE_MAX_OUTPUT_BYTES: usize = 2_097_152;
const FIXTURE_MAX_RECORDS: usize = 4_096;
const FIXTURE_MAX_CONTENT_BYTES: usize = 262_144;
const FIXTURE_MAX_TOOL_CALLS: usize = 1_024;
const FIXTURE_MAX_TOOL_ARGUMENT_BYTES: usize = 65_536;
const FIXTURE_MAX_TOOL_RESULT_BYTES: usize = 262_144;
const FIXTURE_MAX_DIAGNOSTICS: usize = 1_024;
const FIXTURE_SOURCE_COUNT: usize = 7;
const FIXTURE_DIAGNOSTIC_COUNT: usize = 15;

#[test]
fn every_source_validates_and_projects_deterministically() {
    for source in sources() {
        let policy = policy();
        let policy_identity = validate_policy(&policy).expect("policy");
        let request = request(source, policy_identity.as_str());
        let result = valid_result(source);
        let first = validate_trajectory(&policy, &request, &result).expect("validation");
        let second = validate_trajectory(&policy, &request, &result).expect("repeat");
        assert_eq!(first, second);

        let projection = project_trajectory(&policy, &first).expect("projection");
        assert_eq!(projection.ledger_entries.len(), EXPECTED_LEDGER_ENTRY_COUNT);
        assert_eq!(projection.batch_drafts.len(), result.records.len());
        assert!(
            projection
                .omissions
                .contains(&TrajectoryOmission::Reasoning)
        );
        assert!(
            projection
                .omissions
                .contains(&TrajectoryOmission::ToolArguments)
        );
        assert!(
            projection
                .omissions
                .contains(&TrajectoryOmission::ToolResults)
        );
        assert!(projection.batch_drafts[0].descriptive_timestamp.is_none());
        assert!(projection.batch_drafts[1].descriptive_timestamp.is_some());

        let receipt = trajectory_receipt(TrajectoryReceiptInput {
            validation: &first,
            projection: &projection,
            helper_identity: TRAJECTORY_HELPER_BLAKE3,
            service_identity: SERVICE_IDENTITY,
            sensitive: TrajectoryReceiptSensitiveDraft::default(),
        })
        .expect("receipt");
        assert_eq!(receipt.non_claims.len(), REQUIRED_NON_CLAIM_COUNT);
        assert_eq!(receipt.source, source);
        assert_eq!(receipt.receipt_identity.len(), BLAKE3_HEX_LENGTH);
    }
}

#[test]
fn admitted_cleanup_and_lossy_diagnostics_remain_explicit() {
    let policy = policy();
    let identity = validate_policy(&policy).expect("policy");
    let request = request(TrajectorySource::Pi, identity.as_str());
    let mut result = valid_result(TrajectorySource::Pi);
    result.diagnostics = vec![
        TrajectoryDiagnostic {
            code: TrajectoryDiagnosticCode::TimestampsSynthesized,
            count: Some(SYNTHESIZED_TIMESTAMP_COUNT),
        },
        TrajectoryDiagnostic {
            code: TrajectoryDiagnosticCode::ToolArgumentsReshaped,
            count: None,
        },
    ];
    let validated = validate_trajectory(&policy, &request, &result).expect("lossy admitted");
    assert_eq!(validated.disposition, DiagnosticDisposition::Lossy);
    assert_eq!(validated.diagnostic_counts.len(), 2);
}

#[test]
fn malformed_order_content_tools_and_bounds_fail_closed() {
    let base_policy = policy();
    let identity = validate_policy(&base_policy).expect("policy");
    let base_request = request(TrajectorySource::Pi, identity.as_str());

    let mut no_meta = valid_result(TrajectorySource::Pi);
    no_meta.records.remove(0);
    assert_eq!(
        validate_trajectory(&base_policy, &base_request, &no_meta),
        Err(TrajectoryIssue::Meta)
    );

    let mut bad_time = valid_result(TrajectorySource::Pi);
    if let TrajectoryRecord::User { timestamp, .. } = &mut bad_time.records[1] {
        *timestamp = String::from("not-a-time");
    }
    assert_eq!(
        validate_trajectory(&base_policy, &base_request, &bad_time),
        Err(TrajectoryIssue::Content)
    );

    let mut non_object_args = valid_result(TrajectorySource::Pi);
    if let TrajectoryRecord::AssistantToolCalls { tool_calls, .. } = &mut non_object_args.records[3]
    {
        tool_calls[0].arguments = String::from("[]");
    }
    assert_eq!(
        validate_trajectory(&base_policy, &base_request, &non_object_args),
        Err(TrajectoryIssue::ToolArguments)
    );

    let mut duplicate_call = valid_result(TrajectorySource::Pi);
    if let TrajectoryRecord::AssistantToolCalls { tool_calls, .. } = &mut duplicate_call.records[3]
    {
        tool_calls.push(tool_calls[0].clone());
    }
    assert_eq!(
        validate_trajectory(&base_policy, &base_request, &duplicate_call),
        Err(TrajectoryIssue::DuplicateToolCall)
    );

    let mut orphan_result = valid_result(TrajectorySource::Pi);
    if let TrajectoryRecord::Tool { tool_call_id, .. } = &mut orphan_result.records[4] {
        *tool_call_id = String::from("call:missing");
    }
    assert_eq!(
        validate_trajectory(&base_policy, &base_request, &orphan_result),
        Err(TrajectoryIssue::ToolResult)
    );

    let mut bounded_policy = policy();
    bounded_policy.bounds.max_records = RESTRICTED_RECORD_MAX;
    let bounded_identity = validate_policy(&bounded_policy).expect("bounded policy");
    let bounded_request = request(TrajectorySource::Pi, bounded_identity.as_str());
    assert_eq!(
        validate_trajectory(
            &bounded_policy,
            &bounded_request,
            &valid_result(TrajectorySource::Pi)
        ),
        Err(TrajectoryIssue::ResultBound)
    );
}

#[test]
fn cohort_policy_request_and_diagnostic_drift_fail_closed() {
    let mut drifted = policy();
    drifted.helper_identity = String::from("0").repeat(BLAKE3_HEX_LENGTH);
    assert_eq!(validate_policy(&drifted), Err(TrajectoryIssue::Cohort));

    let mut duplicate_source = policy();
    duplicate_source.sources[0] = TrajectorySource::Pi;
    assert_eq!(
        validate_policy(&duplicate_source),
        Err(TrajectoryIssue::SourcePolicy)
    );

    let mut zero_bound = policy();
    zero_bound.bounds.max_records = 0;
    assert_eq!(
        validate_policy(&zero_bound),
        Err(TrajectoryIssue::BoundPolicy)
    );

    let base_policy = policy();
    let identity = validate_policy(&base_policy).expect("policy");
    let mut mismatched = request(TrajectorySource::Pi, identity.as_str());
    mismatched.helper_identity = String::from("f").repeat(BLAKE3_HEX_LENGTH);
    assert_eq!(
        validate_trajectory(
            &base_policy,
            &mismatched,
            &valid_result(TrajectorySource::Pi),
        ),
        Err(TrajectoryIssue::Request)
    );

    let mut rejecting = policy();
    rejecting.diagnostics[0].disposition = DiagnosticDisposition::Rejected;
    let rejecting_identity = validate_policy(&rejecting).expect("rejecting policy");
    let rejecting_request = request(TrajectorySource::Pi, rejecting_identity.as_str());
    let mut diagnostic_result = valid_result(TrajectorySource::Pi);
    diagnostic_result.diagnostics.push(TrajectoryDiagnostic {
        code: rejecting.diagnostics[0].code,
        count: None,
    });
    assert_eq!(
        validate_trajectory(&rejecting, &rejecting_request, &diagnostic_result),
        Err(TrajectoryIssue::Diagnostic)
    );
}

#[test]
fn receipt_rejects_each_sensitive_or_overclaiming_class() {
    let policy = policy();
    let identity = validate_policy(&policy).expect("policy");
    let request = request(TrajectorySource::Pi, identity.as_str());
    let validated =
        validate_trajectory(&policy, &request, &valid_result(TrajectorySource::Pi)).expect("valid");
    let projection = project_trajectory(&policy, &validated).expect("projection");
    let drafts = [
        TrajectoryReceiptSensitiveDraft {
            transcript: Some("raw"),
            ..TrajectoryReceiptSensitiveDraft::default()
        },
        TrajectoryReceiptSensitiveDraft {
            reasoning: Some("private"),
            ..TrajectoryReceiptSensitiveDraft::default()
        },
        TrajectoryReceiptSensitiveDraft {
            tool_body: Some("tool"),
            ..TrajectoryReceiptSensitiveDraft::default()
        },
        TrajectoryReceiptSensitiveDraft {
            diagnostic_message: Some("message"),
            ..TrajectoryReceiptSensitiveDraft::default()
        },
        TrajectoryReceiptSensitiveDraft {
            path_metadata: Some("/home/private"),
            ..TrajectoryReceiptSensitiveDraft::default()
        },
        TrajectoryReceiptSensitiveDraft {
            credential: Some("token"),
            ..TrajectoryReceiptSensitiveDraft::default()
        },
        TrajectoryReceiptSensitiveDraft {
            provider_payload: Some("payload"),
            ..TrajectoryReceiptSensitiveDraft::default()
        },
        TrajectoryReceiptSensitiveDraft {
            content_fingerprint: Some("hash"),
            ..TrajectoryReceiptSensitiveDraft::default()
        },
        TrajectoryReceiptSensitiveDraft {
            helper_output: Some("stdout"),
            ..TrajectoryReceiptSensitiveDraft::default()
        },
        TrajectoryReceiptSensitiveDraft {
            claims_truth: true,
            ..TrajectoryReceiptSensitiveDraft::default()
        },
    ];
    for draft in drafts {
        assert_eq!(
            trajectory_receipt(TrajectoryReceiptInput {
                validation: &validated,
                projection: &projection,
                helper_identity: TRAJECTORY_HELPER_BLAKE3,
                service_identity: SERVICE_IDENTITY,
                sensitive: draft,
            }),
            Err(TrajectoryIssue::Receipt)
        );
    }
}

fn policy() -> TrajectoryPolicy {
    TrajectoryPolicy {
        helper_identity: String::from(TRAJECTORY_HELPER_BLAKE3),
        schema_identity: String::from(TRAJECTORY_SCHEMA_BLAKE3),
        sources: sources().to_vec(),
        bounds: TrajectoryBounds {
            max_transcript_bytes: FIXTURE_MAX_TRANSCRIPT_BYTES,
            max_output_bytes: FIXTURE_MAX_OUTPUT_BYTES,
            max_records: FIXTURE_MAX_RECORDS,
            max_content_bytes: FIXTURE_MAX_CONTENT_BYTES,
            max_tool_calls: FIXTURE_MAX_TOOL_CALLS,
            max_tool_argument_bytes: FIXTURE_MAX_TOOL_ARGUMENT_BYTES,
            max_tool_result_bytes: FIXTURE_MAX_TOOL_RESULT_BYTES,
            max_diagnostics: FIXTURE_MAX_DIAGNOSTICS,
        },
        diagnostics: diagnostic_codes()
            .iter()
            .copied()
            .map(|code| TrajectoryDiagnosticPolicy {
                code,
                disposition: if matches!(
                    code,
                    TrajectoryDiagnosticCode::InjectedContextDropped
                        | TrajectoryDiagnosticCode::NoiseRecordDropped
                        | TrajectoryDiagnosticCode::TimestampsSynthesized
                        | TrajectoryDiagnosticCode::TimestampsInterpolated
                ) {
                    DiagnosticDisposition::AcceptedCleanup
                } else {
                    DiagnosticDisposition::Lossy
                },
            })
            .collect(),
        reasoning: TrajectoryContentDisposition::Omit,
        tool_arguments: TrajectoryContentDisposition::Omit,
        tool_results: TrajectoryContentDisposition::Omit,
        path_metadata: TrajectoryContentDisposition::Omit,
    }
}

fn request(source: TrajectorySource, policy_identity: &str) -> TrajectoryNormalizeRequest {
    TrajectoryNormalizeRequest {
        source,
        transcript: String::from("synthetic transcript"),
        source_generation_identity: String::from(SOURCE_GENERATION),
        policy_identity: String::from(policy_identity),
        helper_identity: String::from(TRAJECTORY_HELPER_BLAKE3),
    }
}

fn valid_result(source: TrajectorySource) -> TrajectoryNormalizeResult {
    TrajectoryNormalizeResult {
        records: vec![
            TrajectoryRecord::Meta {
                source: String::from(source.as_str()),
                cwd: Some(String::from("/private/omitted")),
                git_branch: Some(String::from("private-branch")),
                model: Some(String::from("provider-model")),
            },
            TrajectoryRecord::User {
                content: String::from("hello"),
                timestamp: String::from(TIMESTAMP_ONE),
            },
            TrajectoryRecord::Reasoning {
                content: String::from("private reasoning"),
                timestamp: String::from(TIMESTAMP_TWO),
            },
            TrajectoryRecord::AssistantToolCalls {
                tool_calls: vec![TrajectoryToolCall {
                    id: String::from(TOOL_CALL_ID),
                    name: String::from("lookup"),
                    arguments: String::from("{\"query\":\"private\"}"),
                }],
                timestamp: String::from(TIMESTAMP_TWO),
            },
            TrajectoryRecord::Tool {
                tool_call_id: String::from(TOOL_CALL_ID),
                content: String::from("private tool result"),
                timestamp: String::from(TIMESTAMP_THREE),
            },
            TrajectoryRecord::Assistant {
                content: String::from("done"),
                timestamp: String::from(TIMESTAMP_THREE),
            },
        ],
        diagnostics: Vec::new(),
    }
}

const fn sources() -> [TrajectorySource; FIXTURE_SOURCE_COUNT] {
    [
        TrajectorySource::ClaudeCode,
        TrajectorySource::Codex,
        TrajectorySource::Hermes,
        TrajectorySource::LettaCode,
        TrajectorySource::OpenClaw,
        TrajectorySource::OpenHands,
        TrajectorySource::Pi,
    ]
}

const fn diagnostic_codes() -> [TrajectoryDiagnosticCode; FIXTURE_DIAGNOSTIC_COUNT] {
    [
        TrajectoryDiagnosticCode::InvalidJsonLine,
        TrajectoryDiagnosticCode::NonObjectJsonLine,
        TrajectoryDiagnosticCode::InjectedContextDropped,
        TrajectoryDiagnosticCode::NoiseRecordDropped,
        TrajectoryDiagnosticCode::SidechainRecordDropped,
        TrajectoryDiagnosticCode::ToolCallIdSynthesized,
        TrajectoryDiagnosticCode::DuplicateToolCallId,
        TrajectoryDiagnosticCode::OrphanToolResult,
        TrajectoryDiagnosticCode::DuplicateToolResult,
        TrajectoryDiagnosticCode::UnknownToolName,
        TrajectoryDiagnosticCode::ToolArgumentsReshaped,
        TrajectoryDiagnosticCode::ToolArgumentsTruncated,
        TrajectoryDiagnosticCode::ToolResultTruncated,
        TrajectoryDiagnosticCode::TimestampsSynthesized,
        TrajectoryDiagnosticCode::TimestampsInterpolated,
    ]
}
