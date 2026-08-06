// Cairn adaptation compatibility modules intentionally keep public DTO names and table-driven
// validation compact while drained changes are archived.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    borrowed_argument_types
)]
#![allow(compound_condition, excessive_file_length, function_length, non_trait_imports)]
#![allow(
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    unbounded_collection_growth
)]
use super::BLAKE3_ALGORITHM;
use super::EMPTY_COUNT;
use super::FIELD_LEDGER;
use super::FIELD_POLICY;
use super::FIELD_PROMPT_PRESSURE;
use super::FIELD_SUMMARIZER;
use super::FIELD_SUMMARY;
use super::FIELD_TOOL_ARTIFACT;
use super::FULL_BASIS_POINTS;
use super::HASH_PART_SEPARATOR;
use super::ZERO_BASIS_POINTS;
use super::ZEROSTACK_MIDTURN_HASH_DOMAIN;
use super::data::ZerostackCachedTokenRule;
use super::data::ZerostackInjectedServiceState;
use super::data::ZerostackMidturnDecision;
use super::data::ZerostackMidturnDiagnostic;
use super::data::ZerostackMidturnDiagnosticKind;
use super::data::ZerostackMidturnFingerprint;
use super::data::ZerostackMidturnPlan;
use super::data::ZerostackMidturnPolicy;
use super::data::ZerostackMidturnServices;
use super::data::ZerostackMidturnSummary;
use super::data::ZerostackMidturnSummaryKind;
use super::data::ZerostackMidturnValidationResult;
use super::data::ZerostackPromptPressure;
use super::data::ZerostackProviderUsage;
use super::data::ZerostackRouteMetadata;
use super::data::ZerostackToolArtifactDescriptor;
use super::data::ZerostackToolArtifactPolicy;

const MAX_SUMMARY_CHARS: usize = 4096;
const RAW_CREDENTIAL_MARKER: &str = "API_KEY";
const RAW_HOST_PATH_MARKER: &str = "/home/";
const RAW_PROVIDER_MARKER: &str = "provider_payload";
const RAW_TOOL_MARKER: &str = "tool_output:";
const WINDOWS_PATH_MARKER: &str = ":\\";

/// Computes a domain-separated BLAKE3 fingerprint for mid-turn compaction.
#[must_use]
pub fn blake3_zerostack_midturn_fingerprint(label: &str, bytes: &[u8]) -> ZerostackMidturnFingerprint {
    let mut hasher = blake3::Hasher::new();
    hasher.update(ZEROSTACK_MIDTURN_HASH_DOMAIN.as_bytes());
    hasher.update(HASH_PART_SEPARATOR.as_bytes());
    hasher.update(label.as_bytes());
    hasher.update(HASH_PART_SEPARATOR.as_bytes());
    hasher.update(bytes);
    ZerostackMidturnFingerprint::new(BLAKE3_ALGORITHM, hasher.finalize().to_hex().to_string())
}

/// Calculates provider prompt pressure using route-specific cached-token rules.
///
/// # Errors
///
/// Returns diagnostics when provider input tokens are missing, route metadata is
/// blank, cached-token fields contradict the route rule, or cache handling is
/// unknown.
pub fn calculate_zerostack_prompt_pressure(
    usage: &ZerostackProviderUsage,
    route: &ZerostackRouteMetadata,
) -> Result<ZerostackPromptPressure, Vec<ZerostackMidturnDiagnostic>> {
    let mut diagnostics = Vec::new();
    if route.route_label.trim().is_empty() {
        diagnostics.push(ZerostackMidturnDiagnostic::new(
            ZerostackMidturnDiagnosticKind::MissingField,
            FIELD_PROMPT_PRESSURE,
            "provider route label is required",
        ));
    }
    let Some(input_tokens) = usage.provider_input else {
        diagnostics.push(ZerostackMidturnDiagnostic::new(
            ZerostackMidturnDiagnosticKind::MalformedProviderUsage,
            FIELD_PROMPT_PRESSURE,
            "provider input token count is required for prompt pressure",
        ));
        return Err(diagnostics);
    };
    let prompt_pressure_tokens = match route.cached_token_rule {
        ZerostackCachedTokenRule::IncludedInInputTokens => input_tokens,
        ZerostackCachedTokenRule::SeparateFromInputTokens => {
            input_tokens.saturating_add(usage.cached.unwrap_or_default())
        }
        ZerostackCachedTokenRule::Absent => {
            if usage.cached.is_some() {
                diagnostics.push(ZerostackMidturnDiagnostic::new(
                    ZerostackMidturnDiagnosticKind::MalformedProviderUsage,
                    FIELD_PROMPT_PRESSURE,
                    "cached-token field was present for a route that declares it absent",
                ));
            }
            input_tokens
        }
        ZerostackCachedTokenRule::Unknown => {
            diagnostics.push(ZerostackMidturnDiagnostic::new(
                ZerostackMidturnDiagnosticKind::MalformedProviderUsage,
                FIELD_PROMPT_PRESSURE,
                "cached-token route rule is unknown",
            ));
            input_tokens
        }
    };
    if diagnostics.is_empty() {
        Ok(ZerostackPromptPressure {
            prompt_pressure_tokens,
            estimated_replay_tokens: usage.estimated_replay,
            cache_rule_used: route.cached_token_rule,
            route_label: route.route_label.clone(),
        })
    } else {
        Err(diagnostics)
    }
}

/// Plans explicit mid-turn stop/compact/resume behavior.
#[must_use]
pub fn plan_zerostack_midturn_compaction(
    policy: &ZerostackMidturnPolicy,
    pressure: Option<ZerostackPromptPressure>,
    services: ZerostackMidturnServices,
) -> ZerostackMidturnPlan {
    let mut diagnostics = validate_policy(policy);
    if !policy.enabled {
        return ZerostackMidturnPlan {
            decision: policy.disabled_decision,
            stop_before_continuing: false,
            compact_before_resume: false,
            resume_after_append: policy.disabled_decision != ZerostackMidturnDecision::Fail,
            pressure,
            diagnostics,
        };
    }
    let Some(pressure) = pressure else {
        diagnostics.push(ZerostackMidturnDiagnostic::new(
            ZerostackMidturnDiagnosticKind::MalformedProviderUsage,
            FIELD_PROMPT_PRESSURE,
            "prompt pressure is required before mid-turn compaction planning",
        ));
        return fail_plan(None, diagnostics);
    };
    if !diagnostics.is_empty() {
        return fail_plan(Some(pressure), diagnostics);
    }
    if !pressure_crosses_threshold(policy, pressure.prompt_pressure_tokens) {
        return ZerostackMidturnPlan {
            decision: ZerostackMidturnDecision::Continue,
            stop_before_continuing: false,
            compact_before_resume: false,
            resume_after_append: true,
            pressure: Some(pressure),
            diagnostics,
        };
    }
    if services.ledger != ZerostackInjectedServiceState::Ready {
        diagnostics.push(ZerostackMidturnDiagnostic::new(
            ZerostackMidturnDiagnosticKind::LedgerUnavailable,
            FIELD_LEDGER,
            service_state_detail(services.ledger),
        ));
    }
    if services.summarizer != ZerostackInjectedServiceState::Ready {
        diagnostics.push(ZerostackMidturnDiagnostic::new(
            ZerostackMidturnDiagnosticKind::SummarizerUnavailable,
            FIELD_SUMMARIZER,
            service_state_detail(services.summarizer),
        ));
    }
    if diagnostics.is_empty() {
        ZerostackMidturnPlan {
            decision: ZerostackMidturnDecision::StopCompactResume,
            stop_before_continuing: true,
            compact_before_resume: true,
            resume_after_append: true,
            pressure: Some(pressure),
            diagnostics,
        }
    } else {
        fail_plan(Some(pressure), diagnostics)
    }
}

/// Validates a host-injected mid-turn summary before ledger append.
///
/// # Errors
///
/// Returns diagnostics when summary text is empty, overlong, has raw unsafe
/// markers, uses an unsupported kind, has blank or unsafe metadata, or has
/// missing/non-BLAKE3 hashes.
pub fn validate_zerostack_midturn_summary(summary: &ZerostackMidturnSummary) -> ZerostackMidturnValidationResult {
    let mut diagnostics = Vec::new();
    let text_chars = summary.text.chars().count();
    if text_chars == EMPTY_COUNT || text_chars > MAX_SUMMARY_CHARS {
        diagnostics.push(ZerostackMidturnDiagnostic::new(
            ZerostackMidturnDiagnosticKind::InvalidSummary,
            FIELD_SUMMARY,
            "mid-turn summary text must be non-empty and bounded",
        ));
    }
    if contains_unsafe_marker(&summary.text) {
        diagnostics.push(ZerostackMidturnDiagnostic::new(
            ZerostackMidturnDiagnosticKind::InvalidSummary,
            FIELD_SUMMARY,
            "mid-turn summary text contains unsafe raw material marker",
        ));
    }
    if summary.kind == ZerostackMidturnSummaryKind::Unsupported {
        diagnostics.push(ZerostackMidturnDiagnostic::new(
            ZerostackMidturnDiagnosticKind::InvalidSummary,
            FIELD_SUMMARY,
            "mid-turn summary kind is unsupported",
        ));
    }
    for label in &summary.metadata_labels {
        if label.trim().is_empty() || contains_unsafe_marker(label) {
            diagnostics.push(ZerostackMidturnDiagnostic::new(
                ZerostackMidturnDiagnosticKind::InvalidSummary,
                FIELD_SUMMARY,
                "mid-turn summary metadata must contain safe non-empty labels",
            ));
        }
    }
    require_hash(&summary.source_hash, FIELD_SUMMARY, &mut diagnostics);
    require_hash(&summary.summary_hash, FIELD_SUMMARY, &mut diagnostics);
    to_validation_result(diagnostics)
}

/// Produces a bounded model-visible descriptor for long tool output.
///
/// # Errors
///
/// Returns diagnostics when artifact policy budgets are empty or inconsistent.
pub fn artifactize_zerostack_tool_output(
    artifact_label: &str,
    output: &str,
    policy: &ZerostackToolArtifactPolicy,
) -> Result<ZerostackToolArtifactDescriptor, Vec<ZerostackMidturnDiagnostic>> {
    let diagnostics = validate_tool_policy(artifact_label, policy);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let chars: Vec<char> = output.chars().collect();
    let total_chars = chars.len();
    let truncated = total_chars > policy.artifact_required_over;
    let visible_head = chars.iter().take(policy.visible_head).collect::<String>();
    let tail_start = total_chars.saturating_sub(policy.visible_tail);
    let visible_tail = chars.iter().skip(tail_start).collect::<String>();
    Ok(ZerostackToolArtifactDescriptor {
        artifact_label: artifact_label.to_string(),
        visible_head,
        visible_tail,
        total_chars,
        truncated,
        embeds_full_output: false,
        artifact_hash: blake3_zerostack_midturn_fingerprint(artifact_label, output.as_bytes()),
    })
}

fn contains_unsafe_marker(text: &str) -> bool {
    text.contains(RAW_TOOL_MARKER)
        || text.contains(RAW_PROVIDER_MARKER)
        || text.contains(RAW_CREDENTIAL_MARKER)
        || text.contains(RAW_HOST_PATH_MARKER)
        || text.contains(WINDOWS_PATH_MARKER)
}

fn fail_plan(
    pressure: Option<ZerostackPromptPressure>,
    diagnostics: Vec<ZerostackMidturnDiagnostic>,
) -> ZerostackMidturnPlan {
    ZerostackMidturnPlan {
        decision: ZerostackMidturnDecision::Fail,
        stop_before_continuing: true,
        compact_before_resume: false,
        resume_after_append: false,
        pressure,
        diagnostics,
    }
}

fn pressure_crosses_threshold(policy: &ZerostackMidturnPolicy, pressure_tokens: u32) -> bool {
    let usable_tokens = policy.context_window_tokens.saturating_sub(policy.response_reserve_tokens);
    let pressure_scaled = u64::from(pressure_tokens) * u64::from(FULL_BASIS_POINTS);
    let threshold_scaled = u64::from(usable_tokens) * u64::from(policy.threshold_basis_points);
    pressure_scaled >= threshold_scaled
}

fn require_hash(
    fingerprint: &ZerostackMidturnFingerprint,
    field: &str,
    diagnostics: &mut Vec<ZerostackMidturnDiagnostic>,
) {
    if fingerprint.algorithm != BLAKE3_ALGORITHM || fingerprint.value_hex.trim().is_empty() {
        diagnostics.push(ZerostackMidturnDiagnostic::new(
            ZerostackMidturnDiagnosticKind::InvalidSummary,
            field,
            "mid-turn hashes must use non-empty BLAKE3 fingerprints",
        ));
    }
}

fn service_state_detail(state: ZerostackInjectedServiceState) -> &'static str {
    match state {
        ZerostackInjectedServiceState::Ready => "service is ready",
        ZerostackInjectedServiceState::Missing => "required service is missing",
        ZerostackInjectedServiceState::Denied => "required service denied the request",
        ZerostackInjectedServiceState::Cancelled => "required service was cancelled",
        ZerostackInjectedServiceState::MalformedOutput => "required service returned malformed output",
    }
}

fn to_validation_result(diagnostics: Vec<ZerostackMidturnDiagnostic>) -> ZerostackMidturnValidationResult {
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn validate_policy(policy: &ZerostackMidturnPolicy) -> Vec<ZerostackMidturnDiagnostic> {
    let mut diagnostics = Vec::new();
    if policy.context_window_tokens <= policy.response_reserve_tokens
        || policy.threshold_basis_points == ZERO_BASIS_POINTS
        || policy.threshold_basis_points > FULL_BASIS_POINTS
    {
        diagnostics.push(ZerostackMidturnDiagnostic::new(
            ZerostackMidturnDiagnosticKind::InvalidPolicy,
            FIELD_POLICY,
            "context window must exceed response reserve and threshold must be within basis-point bounds",
        ));
    }
    diagnostics
}

fn validate_tool_policy(artifact_label: &str, policy: &ZerostackToolArtifactPolicy) -> Vec<ZerostackMidturnDiagnostic> {
    let mut diagnostics = Vec::new();
    if artifact_label.trim().is_empty()
        || policy.visible_head == EMPTY_COUNT
        || policy.visible_tail == EMPTY_COUNT
        || policy.artifact_required_over == EMPTY_COUNT
    {
        diagnostics.push(ZerostackMidturnDiagnostic::new(
            ZerostackMidturnDiagnosticKind::InvalidToolArtifactPolicy,
            FIELD_TOOL_ARTIFACT,
            "artifact labels and visible windows must be non-empty",
        ));
    }
    diagnostics
}
