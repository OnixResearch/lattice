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

const ARTIFACT_LABEL: &str = "tool-output-artifact";
const CACHE_TOKENS: u32 = 40;
const CONTEXT_WINDOW_TOKENS: u32 = 1_000;
const ESTIMATED_REPLAY_TOKENS: u32 = 600;
const HEAD_CHARS: usize = 8;
const INPUT_TOKENS: u32 = 700;
const PRESSURE_BELOW_THRESHOLD: u32 = 300;
const PRESSURE_OVER_THRESHOLD: u32 = 760;
const RESPONSE_RESERVE_TOKENS: u32 = 100;
const SUMMARY_TEXT: &str = "Mid-turn summary preserves bounded context.";
const TAIL_CHARS: usize = 7;
const THRESHOLD_BASIS_POINTS: u32 = 8_000;
const TOOL_ARTIFACT_LIMIT: usize = 24;
const TOOL_OUTPUT: &str = "abcdefghij0123456789zyxwvutsrqponmlkjihgfedcba";
const ZERO_CHARS: usize = 0;

#[test]
fn prompt_pressure_applies_cache_rules_without_double_counting() {
    // r[agent_harness_runtime.zerostack_midturn_compaction.prompt_pressure.cache]
    let included = calculate_zerostack_prompt_pressure(
        &usage(Some(INPUT_TOKENS), Some(CACHE_TOKENS)),
        &route(ZerostackCachedTokenRule::IncludedInInputTokens),
    )
    .expect("included cache pressure");
    assert_eq!(included.prompt_pressure_tokens, INPUT_TOKENS);
    assert_eq!(
        included.cache_rule_used,
        ZerostackCachedTokenRule::IncludedInInputTokens
    );

    let separate = calculate_zerostack_prompt_pressure(
        &usage(Some(INPUT_TOKENS), Some(CACHE_TOKENS)),
        &route(ZerostackCachedTokenRule::SeparateFromInputTokens),
    )
    .expect("separate cache pressure");
    assert_eq!(separate.prompt_pressure_tokens, INPUT_TOKENS + CACHE_TOKENS);

    let absent = calculate_zerostack_prompt_pressure(
        &usage(Some(INPUT_TOKENS), None),
        &route(ZerostackCachedTokenRule::Absent),
    )
    .expect("absent cache pressure");
    assert_eq!(absent.prompt_pressure_tokens, INPUT_TOKENS);

    let malformed = calculate_zerostack_prompt_pressure(
        &usage(Some(INPUT_TOKENS), Some(CACHE_TOKENS)),
        &route(ZerostackCachedTokenRule::Absent),
    )
    .expect_err("malformed cache field");
    assert!(malformed.iter().any(|diagnostic| {
        diagnostic.kind == ZerostackMidturnDiagnosticKind::MalformedProviderUsage
    }));
}

#[test]
fn midturn_plan_respects_enabled_disabled_threshold_and_fail_policy() {
    // r[agent_harness_runtime.zerostack_midturn_compaction.plan.disabled]
    let pressure = prompt_pressure(PRESSURE_OVER_THRESHOLD);
    let services = ready_services();
    let plan = plan_zerostack_midturn_compaction(&policy(true), Some(pressure.clone()), services);
    assert_eq!(plan.decision, ZerostackMidturnDecision::StopCompactResume);
    assert!(plan.stop_before_continuing);
    assert!(plan.compact_before_resume);
    assert!(plan.resume_after_append);

    let disabled = plan_zerostack_midturn_compaction(&policy(false), Some(pressure), services);
    assert_eq!(disabled.decision, ZerostackMidturnDecision::Warn);
    assert!(!disabled.compact_before_resume);

    let under_threshold = plan_zerostack_midturn_compaction(
        &policy(true),
        Some(prompt_pressure(PRESSURE_BELOW_THRESHOLD)),
        services,
    );
    assert_eq!(under_threshold.decision, ZerostackMidturnDecision::Continue);

    let missing_pressure = plan_zerostack_midturn_compaction(&policy(true), None, services);
    assert_eq!(missing_pressure.decision, ZerostackMidturnDecision::Fail);
    assert!(!missing_pressure.resume_after_append);
}

#[test]
fn missing_ledger_service_halts_without_model_or_tool_fallback() {
    // r[agent_harness_runtime.zerostack_midturn_compaction.ledger_entries.missing]
    let services = ZerostackMidturnServices {
        ledger: ZerostackInjectedServiceState::Missing,
        summarizer: ZerostackInjectedServiceState::Ready,
    };
    let plan = plan_zerostack_midturn_compaction(
        &policy(true),
        Some(prompt_pressure(PRESSURE_OVER_THRESHOLD)),
        services,
    );

    assert_eq!(plan.decision, ZerostackMidturnDecision::Fail);
    assert!(plan.stop_before_continuing);
    assert!(!plan.compact_before_resume);
    assert!(!plan.resume_after_append);
    assert!(plan.diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == ZerostackMidturnDiagnosticKind::LedgerUnavailable
    }));
}

#[test]
fn summary_validation_rejects_malformed_and_unsafe_outputs() {
    // r[agent_harness_runtime.zerostack_midturn_compaction.summarizer.malformed]
    let valid = summary(
        SUMMARY_TEXT,
        ZerostackMidturnSummaryKind::Midturn,
        vec!["turn:one"],
    );
    assert!(validate_zerostack_midturn_summary(&valid).is_ok());

    let malformed = ZerostackMidturnSummary {
        text: "tool_output: provider_payload API_KEY /home/user/file".to_string(),
        kind: ZerostackMidturnSummaryKind::Unsupported,
        metadata_labels: vec!["/home/user/path".to_string()],
        source_hash: ZerostackMidturnFingerprint::new("SHA256", "abc"),
        ..summary(
            SUMMARY_TEXT,
            ZerostackMidturnSummaryKind::Midturn,
            vec!["turn:one"],
        )
    };
    let diagnostics = validate_zerostack_midturn_summary(&malformed).expect_err("bad summary");
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == ZerostackMidturnDiagnosticKind::InvalidSummary
        })
    );
}

#[test]
fn long_tool_output_artifact_keeps_only_bounded_visible_windows() {
    // r[agent_harness_runtime.zerostack_midturn_compaction.tool_artifacts.bounded]
    let artifact = artifactize_zerostack_tool_output(
        ARTIFACT_LABEL,
        TOOL_OUTPUT,
        &tool_artifact_policy(HEAD_CHARS, TAIL_CHARS),
    )
    .expect("artifact descriptor");

    assert!(artifact.truncated);
    assert!(!artifact.embeds_full_output);
    assert_eq!(artifact.visible_head, "abcdefgh");
    assert_eq!(artifact.visible_tail, "gfedcba");
    assert_eq!(artifact.total_chars, TOOL_OUTPUT.chars().count());
    assert_eq!(artifact.artifact_hash.algorithm, BLAKE3_ALGORITHM);

    let invalid = artifactize_zerostack_tool_output(
        ARTIFACT_LABEL,
        TOOL_OUTPUT,
        &tool_artifact_policy(ZERO_CHARS, TAIL_CHARS),
    )
    .expect_err("invalid artifact policy");
    assert!(invalid.iter().any(|diagnostic| {
        diagnostic.kind == ZerostackMidturnDiagnosticKind::InvalidToolArtifactPolicy
    }));
}

fn fp(label: &str) -> ZerostackMidturnFingerprint {
    blake3_zerostack_midturn_fingerprint(label, label.as_bytes())
}

fn policy(enabled: bool) -> ZerostackMidturnPolicy {
    ZerostackMidturnPolicy {
        enabled,
        context_window_tokens: CONTEXT_WINDOW_TOKENS,
        response_reserve_tokens: RESPONSE_RESERVE_TOKENS,
        threshold_basis_points: THRESHOLD_BASIS_POINTS,
        disabled_decision: ZerostackMidturnDecision::Warn,
    }
}

fn prompt_pressure(prompt_pressure_tokens: u32) -> ZerostackPromptPressure {
    ZerostackPromptPressure {
        prompt_pressure_tokens,
        estimated_replay_tokens: ESTIMATED_REPLAY_TOKENS,
        cache_rule_used: ZerostackCachedTokenRule::IncludedInInputTokens,
        route_label: "route.local".to_string(),
    }
}

fn ready_services() -> ZerostackMidturnServices {
    ZerostackMidturnServices {
        ledger: ZerostackInjectedServiceState::Ready,
        summarizer: ZerostackInjectedServiceState::Ready,
    }
}

fn route(cached_token_rule: ZerostackCachedTokenRule) -> ZerostackRouteMetadata {
    ZerostackRouteMetadata {
        route_label: "route.local".to_string(),
        cached_token_rule,
    }
}

fn summary(
    text: &str,
    kind: ZerostackMidturnSummaryKind,
    metadata_labels: Vec<&str>,
) -> ZerostackMidturnSummary {
    ZerostackMidturnSummary {
        text: text.to_string(),
        kind,
        metadata_labels: metadata_labels
            .into_iter()
            .map(std::string::ToString::to_string)
            .collect(),
        source_hash: fp("source"),
        summary_hash: fp("summary"),
    }
}

fn tool_artifact_policy(
    visible_head_chars: usize,
    visible_tail_chars: usize,
) -> ZerostackToolArtifactPolicy {
    ZerostackToolArtifactPolicy {
        visible_head: visible_head_chars,
        visible_tail: visible_tail_chars,
        artifact_required_over: TOOL_ARTIFACT_LIMIT,
    }
}

fn usage(provider_input_tokens: Option<u32>, cached_tokens: Option<u32>) -> ZerostackProviderUsage {
    ZerostackProviderUsage {
        estimated_replay: ESTIMATED_REPLAY_TOKENS,
        provider_input: provider_input_tokens,
        cached: cached_tokens,
    }
}
