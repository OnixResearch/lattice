const RAW_PROMPT_SENTINELS: &[&str] = &["raw prompt", "prompt:"];
const PROVIDER_SENTINELS: &[&str] = &["provider json", "provider payload"];
const WORKER_SENTINEL: &str = "worker output";
const JUDGE_SENTINEL: &str = "judge rationale";
const HIDDEN_SENTINEL: &str = "hidden reasoning";
const CREDENTIAL_SENTINELS: &[&str] = &["credential", "api_key", "secret"];
const PATH_SENTINELS: &[&str] = &["/home/", "/tmp/"];
const TRACE_SENTINELS: &[&str] = &["unbounded trace", "full trace"];

/// Router receipt leak class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptLeak {
    /// Raw prompt text was present.
    RawPrompt,
    /// Provider JSON or payload was present.
    ProviderPayload,
    /// Worker output was present.
    WorkerOutput,
    /// Judge rationale was present.
    JudgeRationale,
    /// Hidden reasoning text was present.
    HiddenReasoning,
    /// Credential-like material was present.
    Credential,
    /// Full path was present.
    FullPath,
    /// Unbounded trace was present.
    UnboundedTrace,
}

/// Validates safe receipt text for known leak sentinels.
///
/// # Errors
///
/// Returns [`ReceiptLeak`] when unsafe material is detected.
///
/// # Panics
///
/// Panics only if built-in leak sentinel tables are accidentally emptied.
pub fn validate_receipt_text(text: &str) -> Result<(), ReceiptLeak> {
    assert!(
        !RAW_PROMPT_SENTINELS.is_empty(),
        "raw prompt sentinels must be configured"
    );
    assert!(
        !TRACE_SENTINELS.is_empty(),
        "trace sentinels must be configured"
    );
    leak_for_text(text).map_or(Ok(()), Err)
}

fn leak_for_text(text: &str) -> Option<ReceiptLeak> {
    assert!(
        !PROVIDER_SENTINELS.is_empty(),
        "provider sentinels must be configured"
    );
    assert!(
        !CREDENTIAL_SENTINELS.is_empty(),
        "credential sentinels must be configured"
    );
    let lower = text.to_ascii_lowercase();
    if contains_any(&lower, RAW_PROMPT_SENTINELS) {
        return Some(ReceiptLeak::RawPrompt);
    }
    if contains_any(&lower, PROVIDER_SENTINELS) {
        return Some(ReceiptLeak::ProviderPayload);
    }
    if lower.contains(WORKER_SENTINEL) {
        return Some(ReceiptLeak::WorkerOutput);
    }
    if lower.contains(JUDGE_SENTINEL) {
        return Some(ReceiptLeak::JudgeRationale);
    }
    if lower.contains(HIDDEN_SENTINEL) {
        return Some(ReceiptLeak::HiddenReasoning);
    }
    if contains_any(&lower, CREDENTIAL_SENTINELS) {
        return Some(ReceiptLeak::Credential);
    }
    if contains_any(&lower, PATH_SENTINELS) {
        return Some(ReceiptLeak::FullPath);
    }
    if contains_any(&lower, TRACE_SENTINELS) {
        return Some(ReceiptLeak::UnboundedTrace);
    }
    None
}

fn contains_any(text: &str, sentinels: &[&str]) -> bool {
    sentinels.iter().any(|sentinel| text.contains(sentinel))
}
