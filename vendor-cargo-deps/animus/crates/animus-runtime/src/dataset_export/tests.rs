// Cairn adaptation compatibility modules intentionally keep public DTO names and table-driven validation compact while drained changes are archived.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    borrowed_argument_types
)]
#![allow(
    compound_condition,
    excessive_file_length,
    function_length,
    non_trait_imports
)]
#![allow(
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    unbounded_collection_growth
)]

use super::*;

const ACTION_ID: &str = "action";
const CONTEXT_USER_MESSAGE_INDEX: usize = 1;
const ASSISTANT_TEXT: &str = "Use the safe runtime DTO.";
const ASSISTANT_TEXT_WITH_ESCAPES: &str = "Use the \"safe\" runtime DTO.\nKeep rows bounded.";
const BRANCH_ALT: &str = "branch.alt";
const BRANCH_ID: &str = "branch.main";
const EXPECTED_CONTEXT_MESSAGES: usize = 3;
const EXPECTED_TOOL_MESSAGES: usize = 4;
const EXPECTED_EXPORTED_ROWS: u32 = 1;
const EXPECTED_WARNING_COUNT: usize = 1;
const JSONL_LINE_COUNT: usize = 2;
const SESSION_ID: &str = "session.one";
const SOURCE_SPAN_DUPLICATE: &str = "span:duplicate";
const SYSTEM_TEXT: &str = "Summarized safe project context.";
const TOOL_CALL_TEXT: &str = "call:read-only-search";
const TOOL_RESULT_RAW: &str = "tool_output: private raw payload";
const USER_MESSAGE_INDEX: usize = 0;
const USER_TEXT: &str = "What should I change?";

#[test]
fn chat_messages_export_sft_summary_branch_and_jsonl() {
    // r[agent_harness_runtime.hf_dataset_export.chat_messages.sft]
    // r[agent_harness_runtime.hf_dataset_export.closeout]
    let main_row = export_chat_messages_row(
        &source(vec![
            entry("system", DatasetReplayRole::System, SYSTEM_TEXT),
            entry("user", DatasetReplayRole::User, USER_TEXT),
            entry(
                "assistant",
                DatasetReplayRole::Assistant,
                ASSISTANT_TEXT_WITH_ESCAPES,
            ),
        ]),
        &policy_with_system(),
    )
    .expect("chat row with summary context");

    assert_eq!(main_row.messages.len(), EXPECTED_CONTEXT_MESSAGES);
    assert_eq!(main_row.messages[USER_MESSAGE_INDEX].role, ROLE_SYSTEM);
    assert_eq!(
        main_row.messages[CONTEXT_USER_MESSAGE_INDEX].role,
        ROLE_USER
    );
    assert_eq!(main_row.row_id.algorithm, BLAKE3_ALGORITHM);

    let mut branch_source = source(replay());
    branch_source.branch_id = BRANCH_ALT.to_string();
    let branch_row =
        export_chat_messages_row(&branch_source, &policy()).expect("branch-selected row");
    assert_ne!(main_row.row_id, branch_row.row_id);

    let line = render_chat_row_jsonl(&main_row).expect("jsonl row projection");
    assert!(line.contains("\"messages\""));
    assert!(line.contains("\\\"safe\\\""));
    assert!(line.contains("\\nKeep rows"));

    let batch = render_chat_rows_jsonl(&[main_row.clone(), branch_row]).expect("jsonl batch");
    assert_eq!(batch.lines().count(), JSONL_LINE_COUNT);
}

#[test]
fn chat_messages_reject_bad_sources_and_rows() {
    // r[agent_harness_runtime.hf_dataset_export.session_source.rejects_invalid]
    // r[agent_harness_runtime.hf_dataset_export.chat_messages.rejects_bad_rows]
    assert_error_kind(
        export_chat_messages_row(
            &DatasetSessionSource {
                replay: Vec::new(),
                ..source(replay())
            },
            &policy(),
        ),
        DatasetDiagnosticKind::InvalidSource,
    );

    assert_error_kind(
        export_chat_messages_row(
            &DatasetSessionSource {
                replay: vec![entry("user", DatasetReplayRole::User, USER_TEXT)],
                target: DatasetTarget::Preference,
                ..source(replay())
            },
            &policy(),
        ),
        DatasetDiagnosticKind::InvalidSource,
    );

    assert_error_kind(
        export_chat_messages_row(
            &DatasetSessionSource {
                replay: vec![
                    entry_with_span(
                        "user",
                        DatasetReplayRole::User,
                        USER_TEXT,
                        SOURCE_SPAN_DUPLICATE,
                    ),
                    entry_with_span(
                        "assistant",
                        DatasetReplayRole::Assistant,
                        ASSISTANT_TEXT,
                        SOURCE_SPAN_DUPLICATE,
                    ),
                ],
                ..source(replay())
            },
            &policy(),
        ),
        DatasetDiagnosticKind::InvalidSource,
    );

    assert_error_kind(
        export_chat_messages_row(
            &DatasetSessionSource {
                replay: vec![
                    entry("user", DatasetReplayRole::User, USER_TEXT),
                    entry("assistant", DatasetReplayRole::Assistant, ""),
                ],
                ..source(replay())
            },
            &policy(),
        ),
        DatasetDiagnosticKind::BadChatRow,
    );

    assert_error_kind(
        export_chat_messages_row(
            &DatasetSessionSource {
                replay: vec![DatasetReplayEntry {
                    role: DatasetReplayRole::Unsupported,
                    ..entry("bad", DatasetReplayRole::User, USER_TEXT)
                }],
                ..source(replay())
            },
            &policy(),
        ),
        DatasetDiagnosticKind::BadChatRow,
    );

    assert_error_kind(
        export_chat_messages_row(
            &DatasetSessionSource {
                replay: vec![entry(
                    "tool",
                    DatasetReplayRole::ToolResult,
                    TOOL_RESULT_RAW,
                )],
                ..source(replay())
            },
            &policy_with_tools(),
        ),
        DatasetDiagnosticKind::BadChatRow,
    );
}

#[test]
fn default_and_opt_in_policy_exclude_private_payloads() {
    // r[agent_harness_runtime.hf_dataset_export.policy]
    // r[agent_harness_runtime.hf_dataset_export.policy.leakage_default]
    // r[agent_harness_runtime.hf_dataset_export.policy.opt_in]
    let hidden_reasoning_source = DatasetSessionSource {
        replay: vec![
            entry("user", DatasetReplayRole::User, USER_TEXT),
            entry(
                "reason",
                DatasetReplayRole::HiddenReasoning,
                "hidden_reasoning: secret",
            ),
            entry("assistant", DatasetReplayRole::Assistant, ASSISTANT_TEXT),
        ],
        ..source(replay())
    };
    let row = export_chat_messages_row(&hidden_reasoning_source, &policy())
        .expect("reasoning excluded by default");
    assert_eq!(row.messages.len(), JSONL_LINE_COUNT);
    assert!(!row.messages.iter().any(|message| {
        message.content.contains("hidden_reasoning") || message.content.contains("secret")
    }));

    assert_error_kind(
        export_chat_messages_row(
            &DatasetSessionSource {
                replay: vec![
                    entry("user", DatasetReplayRole::User, USER_TEXT),
                    entry(
                        "assistant",
                        DatasetReplayRole::Assistant,
                        "private_provenance: reviewer note",
                    ),
                ],
                ..source(replay())
            },
            &policy(),
        ),
        DatasetDiagnosticKind::Leakage,
    );

    let tool_source = DatasetSessionSource {
        replay: vec![
            entry("user", DatasetReplayRole::User, USER_TEXT),
            entry("tool-call", DatasetReplayRole::ToolCall, TOOL_CALL_TEXT),
            entry(
                "tool-result",
                DatasetReplayRole::ToolResult,
                TOOL_RESULT_RAW,
            ),
            entry("assistant", DatasetReplayRole::Assistant, ASSISTANT_TEXT),
        ],
        ..source(replay())
    };
    let tool_row =
        export_chat_messages_row(&tool_source, &policy_with_tools()).expect("redacted tools");
    assert_eq!(tool_row.messages.len(), EXPECTED_TOOL_MESSAGES);
    assert!(tool_row.messages.iter().any(|message| {
        message.role == ROLE_TOOL && message.content == "[tool-result-redacted]"
    }));
    assert!(tool_row.messages.iter().all(|message| {
        !message.content.contains("tool_output") && !message.content.contains("private raw")
    }));
}

#[test]
fn jsonl_rendering_rejects_invalid_public_rows() {
    // r[agent_harness_runtime.hf_dataset_export.chat_messages]
    let row = export_chat_messages_row(&source(replay()), &policy()).expect("valid row");

    let mut invalid_role = row.clone();
    invalid_role.messages[USER_MESSAGE_INDEX].role = "developer".to_string();
    assert_error_kind(
        render_chat_row_jsonl(&invalid_role),
        DatasetDiagnosticKind::BadChatRow,
    );

    let mut invalid_identity = row.clone();
    invalid_identity.row_id.value_hex = "not-a-blake3-digest".to_string();
    assert_error_kind(
        render_chat_row_jsonl(&invalid_identity),
        DatasetDiagnosticKind::BadChatRow,
    );

    let mut leaking_span = row;
    leaking_span.source_spans[USER_MESSAGE_INDEX] = "/home/private/session.jsonl".to_string();
    assert_error_kind(
        render_chat_row_jsonl(&leaking_span),
        DatasetDiagnosticKind::Leakage,
    );

    assert_error_kind(
        render_chat_rows_jsonl(&[]),
        DatasetDiagnosticKind::BadChatRow,
    );
}

#[test]
fn preference_rows_validate_positive_and_negative_cases() {
    // r[agent_harness_runtime.hf_dataset_export.preference_rows.validated]
    // r[agent_harness_runtime.hf_dataset_export.preference_rows.rejects_malformed]
    let valid = valid_preference();
    let row = export_preference_row(&valid).expect("preference row");
    assert_eq!(row.chosen, "good");
    assert_eq!(row.rejected, "bad");

    let missing_prompt = DatasetPreferenceExample {
        prompt: Vec::new(),
        ..valid.clone()
    };
    let duplicate_candidate = DatasetPreferenceExample {
        rejected: "good".to_string(),
        ..valid.clone()
    };
    let missing_label = DatasetPreferenceExample {
        outcome_label: String::new(),
        ..valid.clone()
    };
    let unsafe_metadata = DatasetPreferenceExample {
        metadata: vec!["env_secret:TOKEN=value".to_string()],
        ..valid.clone()
    };
    let invalid_prompt_role = DatasetPreferenceExample {
        prompt: vec![message("developer", USER_TEXT)],
        ..valid
    };

    for invalid in [
        missing_prompt,
        duplicate_candidate,
        missing_label,
        unsafe_metadata,
        invalid_prompt_role,
    ] {
        assert_error_kind(
            export_preference_row(&invalid),
            DatasetDiagnosticKind::MalformedPreference,
        );
    }
}

#[test]
fn rl_trajectories_validate_positive_and_negative_cases() {
    // r[agent_harness_runtime.hf_dataset_export.rl_trajectories.validated]
    // r[agent_harness_runtime.hf_dataset_export.rl_trajectories.rejects_incomplete]
    let valid = valid_trajectory();
    let row = export_rl_trajectory_row(&valid).expect("trajectory row");
    assert_eq!(row.reward_label, "reward:+1");
    assert_eq!(row.terminal_outcome, "success");

    let missing_reward = DatasetRlTrajectory {
        reward_label: None,
        ..valid.clone()
    };
    let missing_terminal = DatasetRlTrajectory {
        terminal_outcome: None,
        ..valid.clone()
    };
    let orphan_observation = DatasetRlTrajectory {
        steps: vec![
            step("prompt", DatasetRlStepKind::Prompt, None),
            step(ACTION_ID, DatasetRlStepKind::Action, None),
            step("observation", DatasetRlStepKind::Observation, None),
        ],
        ..valid.clone()
    };
    let invalid_action_link = DatasetRlTrajectory {
        steps: vec![
            step("prompt", DatasetRlStepKind::Prompt, None),
            step(ACTION_ID, DatasetRlStepKind::Action, Some("unexpected")),
            step(
                "observation",
                DatasetRlStepKind::Observation,
                Some(ACTION_ID),
            ),
        ],
        ..valid.clone()
    };
    let missing_action_link_target = DatasetRlTrajectory {
        steps: vec![
            step("prompt", DatasetRlStepKind::Prompt, None),
            step(ACTION_ID, DatasetRlStepKind::Action, None),
            step(
                "observation",
                DatasetRlStepKind::Observation,
                Some("missing-action"),
            ),
        ],
        ..valid.clone()
    };
    let unsafe_metadata = DatasetRlTrajectory {
        metadata: vec!["private_provenance:raw".to_string()],
        ..valid
    };

    for invalid in [
        missing_reward,
        missing_terminal,
        orphan_observation,
        invalid_action_link,
        missing_action_link_target,
        unsafe_metadata,
    ] {
        assert_error_kind(
            export_rl_trajectory_row(&invalid),
            DatasetDiagnosticKind::IncompleteTrajectory,
        );
    }
}

#[test]
fn inspection_reports_safe_skips_policy_and_warning_counts() {
    // r[agent_harness_runtime.hf_dataset_export.inspection]
    // r[agent_harness_runtime.hf_dataset_export.inspection.safe_reports]
    let diagnostics = vec![
        DatasetDiagnostic::new(
            DatasetDiagnosticKind::Leakage,
            FIELD_POLICY,
            "safe leakage class without raw payload",
        ),
        DatasetDiagnostic::new(
            DatasetDiagnosticKind::BadChatRow,
            FIELD_ROW,
            "assistant target is below quality threshold",
        ),
    ];
    let report = inspect_dataset_exports(EXPECTED_EXPORTED_ROWS, &diagnostics, &policy());
    assert_eq!(report.exported_rows, EXPECTED_EXPORTED_ROWS);
    assert_eq!(report.skipped_rows[USER_MESSAGE_INDEX].reason, "Leakage");
    assert_eq!(report.leakage_warning_count, EXPECTED_WARNING_COUNT);
    assert_eq!(report.quality_warning_count, EXPECTED_WARNING_COUNT);
    assert!(report.usage_summary.is_none());
    assert!(report.policy_decisions.iter().any(|decision| {
        decision.contains("hidden_reasoning") && !decision.contains("hidden_reasoning:")
    }));
}

fn assert_error_kind<T: std::fmt::Debug>(
    result: DatasetExportResult<T>,
    kind: DatasetDiagnosticKind,
) {
    let diagnostics = result.expect_err("expected dataset export diagnostic");
    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic.kind == kind),
        "expected {kind:?}, got {diagnostics:?}"
    );
}

fn entry(id: &str, role: DatasetReplayRole, content: &str) -> DatasetReplayEntry {
    entry_with_span(id, role, content, &format!("span:{id}"))
}

fn entry_with_span(
    id: &str,
    role: DatasetReplayRole,
    content: &str,
    source_span: &str,
) -> DatasetReplayEntry {
    DatasetReplayEntry {
        entry_id: id.to_string(),
        role,
        content: content.to_string(),
        source_span: source_span.to_string(),
    }
}

fn message(role: &str, content: &str) -> DatasetChatMessage {
    DatasetChatMessage {
        role: role.to_string(),
        content: content.to_string(),
    }
}

fn policy() -> DatasetExportPolicy {
    DatasetExportPolicy {
        system_text: DatasetMetadataProjection::Exclude,
        tool_calls: DatasetMetadataProjection::Exclude,
        tool_results: DatasetMetadataProjection::Exclude,
        hidden_reasoning: DatasetMetadataProjection::Exclude,
        metadata: DatasetMetadataProjection::Exclude,
        min_assistant_chars: REQUIRED_COUNT,
    }
}

fn policy_with_system() -> DatasetExportPolicy {
    DatasetExportPolicy {
        system_text: DatasetMetadataProjection::SafeLabels,
        ..policy()
    }
}

fn policy_with_tools() -> DatasetExportPolicy {
    DatasetExportPolicy {
        tool_calls: DatasetMetadataProjection::SafeLabels,
        tool_results: DatasetMetadataProjection::SafeLabels,
        ..policy()
    }
}

fn replay() -> Vec<DatasetReplayEntry> {
    vec![
        entry("user", DatasetReplayRole::User, USER_TEXT),
        entry("assistant", DatasetReplayRole::Assistant, ASSISTANT_TEXT),
    ]
}

fn source(replay: Vec<DatasetReplayEntry>) -> DatasetSessionSource {
    DatasetSessionSource {
        session_id: SESSION_ID.to_string(),
        branch_id: BRANCH_ID.to_string(),
        target: DatasetTarget::ChatMessages,
        replay,
    }
}

fn valid_preference() -> DatasetPreferenceExample {
    DatasetPreferenceExample {
        prompt: vec![message(ROLE_USER, USER_TEXT)],
        chosen: "good".to_string(),
        rejected: "bad".to_string(),
        outcome_label: "human-preferred".to_string(),
        metadata: vec!["safe".to_string()],
    }
}

fn valid_trajectory() -> DatasetRlTrajectory {
    DatasetRlTrajectory {
        trajectory_id: "traj.one".to_string(),
        steps: vec![
            step("prompt", DatasetRlStepKind::Prompt, None),
            step(ACTION_ID, DatasetRlStepKind::Action, None),
            step(
                "observation",
                DatasetRlStepKind::Observation,
                Some(ACTION_ID),
            ),
        ],
        reward_label: Some("reward:+1".to_string()),
        terminal_outcome: Some("success".to_string()),
        metadata: vec!["safe".to_string()],
    }
}

fn step(id: &str, kind: DatasetRlStepKind, link_id: Option<&str>) -> DatasetRlStep {
    DatasetRlStep {
        step_id: id.to_string(),
        kind,
        content: id.to_string(),
        link_id: link_id.map(std::string::ToString::to_string),
    }
}
