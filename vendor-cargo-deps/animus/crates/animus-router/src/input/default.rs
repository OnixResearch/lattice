const TASK_SIMPLE: &str = "simple";
const TASK_REASONING: &str = "reasoning";
const TASK_CODING: &str = "coding";
const TASK_REVIEW: &str = "review";
const TASK_WORKFLOW: &str = "workflow";

const BAND_LOW: &str = "low";
const BAND_MEDIUM: &str = "medium";
const BAND_HIGH: &str = "high";
const BAND_STRICT: &str = "strict";

const WORKFLOW_TOOL_CALL_THRESHOLD: u32 = 2;
const HIGH_TOOL_CALL_THRESHOLD: u32 = 1;
const MEDIUM_PROMPT_CHAR_THRESHOLD: u32 = 2_000;
const HIGH_PROMPT_CHAR_THRESHOLD: u32 = 8_000;
const STRICT_PROMPT_CHAR_THRESHOLD: u32 = 24_000;
const REVIEW_PROMPT_CHAR_THRESHOLD: u32 = 4_000;
const MEDIUM_CONTEXT_PRESSURE_PER_MILLE: u16 = 400;
const HIGH_CONTEXT_PRESSURE_PER_MILLE: u16 = 700;
const STRICT_CONTEXT_PRESSURE_PER_MILLE: u16 = 900;

/// Implementation-free signal-source seam for host classifiers.
pub trait SignalSource {
    /// Projects bounded request metadata into a router signal draft.
    ///
    /// # Errors
    ///
    /// Returns [`super::SignalProjectionError`] when the source cannot produce safe
    /// bounded signal labels.
    fn project(
        &self,
        metadata: &super::RequestMetadata,
    ) -> Result<crate::SignalDraft<'static>, super::SignalProjectionError>;
}

/// Deterministic default signal source.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RuleSignalSource;

impl SignalSource for RuleSignalSource {
    fn project(
        &self,
        metadata: &super::RequestMetadata,
    ) -> Result<crate::SignalDraft<'static>, super::SignalProjectionError> {
        Ok(project_signals(metadata))
    }
}

/// Pure default projector from bounded request metadata to router signal bands.
#[must_use]
pub fn project_signals(metadata: &super::RequestMetadata) -> crate::SignalDraft<'static> {
    crate::SignalDraft {
        task_shape: task_shape(metadata),
        risk_band: risk_band(metadata),
        contract_pressure: contract_pressure(metadata),
        context_pressure: context_pressure(metadata),
        latency_band: latency_band(metadata),
        cost_band: cost_band(metadata),
        privacy_band: metadata.privacy_signal(),
        fallback_eligibility: metadata.fallback_signal(),
    }
}

fn task_shape(metadata: &super::RequestMetadata) -> &'static str {
    if metadata.contains_code() {
        return TASK_CODING;
    }
    if metadata.tool_call_count() >= WORKFLOW_TOOL_CALL_THRESHOLD {
        return TASK_WORKFLOW;
    }
    if is_structured_review(metadata) {
        return TASK_REVIEW;
    }
    if metadata.prompt_char_estimate() >= MEDIUM_PROMPT_CHAR_THRESHOLD {
        return TASK_REASONING;
    }
    TASK_SIMPLE
}

fn is_structured_review(metadata: &super::RequestMetadata) -> bool {
    if !metadata.requires_structured_output() {
        return false;
    }
    metadata.prompt_char_estimate() >= REVIEW_PROMPT_CHAR_THRESHOLD
}

fn risk_band(metadata: &super::RequestMetadata) -> &'static str {
    if strict_risk_applies(metadata) {
        return BAND_STRICT;
    }
    if high_risk_applies(metadata) {
        return BAND_HIGH;
    }
    if medium_risk_applies(metadata) {
        return BAND_MEDIUM;
    }
    BAND_LOW
}

fn strict_risk_applies(metadata: &super::RequestMetadata) -> bool {
    if metadata.prompt_char_estimate() >= STRICT_PROMPT_CHAR_THRESHOLD {
        return true;
    }
    metadata.context_window_fill_per_mille() >= STRICT_CONTEXT_PRESSURE_PER_MILLE
}

fn high_risk_applies(metadata: &super::RequestMetadata) -> bool {
    if metadata.prompt_char_estimate() >= HIGH_PROMPT_CHAR_THRESHOLD {
        return true;
    }
    metadata.tool_call_count() >= HIGH_TOOL_CALL_THRESHOLD
}

fn medium_risk_applies(metadata: &super::RequestMetadata) -> bool {
    if metadata.prompt_char_estimate() >= MEDIUM_PROMPT_CHAR_THRESHOLD {
        return true;
    }
    metadata.context_window_fill_per_mille() >= MEDIUM_CONTEXT_PRESSURE_PER_MILLE
}

fn contract_pressure(metadata: &super::RequestMetadata) -> &'static str {
    if metadata.requires_structured_output() {
        return BAND_STRICT;
    }
    if metadata.tool_call_count() >= HIGH_TOOL_CALL_THRESHOLD {
        return BAND_HIGH;
    }
    BAND_LOW
}

fn context_pressure(metadata: &super::RequestMetadata) -> &'static str {
    if metadata.context_window_fill_per_mille() >= STRICT_CONTEXT_PRESSURE_PER_MILLE {
        return BAND_STRICT;
    }
    if metadata.context_window_fill_per_mille() >= HIGH_CONTEXT_PRESSURE_PER_MILLE {
        return BAND_HIGH;
    }
    if metadata.context_window_fill_per_mille() >= MEDIUM_CONTEXT_PRESSURE_PER_MILLE {
        return BAND_MEDIUM;
    }
    BAND_LOW
}

fn latency_band(metadata: &super::RequestMetadata) -> &'static str {
    match risk_band(metadata) {
        BAND_STRICT | BAND_HIGH => BAND_HIGH,
        BAND_MEDIUM => BAND_MEDIUM,
        _ => BAND_LOW,
    }
}

fn cost_band(metadata: &super::RequestMetadata) -> &'static str {
    if metadata.tool_call_count() >= WORKFLOW_TOOL_CALL_THRESHOLD {
        return BAND_HIGH;
    }
    latency_band(metadata)
}
