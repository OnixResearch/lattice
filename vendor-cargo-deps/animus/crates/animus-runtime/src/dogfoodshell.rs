//! Fake-backed headless dogfood shell contracts.
//!
//! This module is a pure functional core for the non-default dogfood path. The
//! host shell owns CLI parsing, stdout, persistence, model execution, and tool
//! execution; this module admits already-loaded DTOs, projects ordered events,
//! and validates safe receipts.

// Cairn adaptation compatibility module keeps DTOs, validators, and fixtures compact while the change is drained.
#![allow(missing_docs)]
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
    non_trait_imports,
    path_segment_repetition,
    underscore_in_module_filename,
    dead_code
)]
#![allow(numeric_units, raw_arithmetic_overflow, unbounded_collection_growth)]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::struct_excessive_bools
)]

const HASH_DOMAIN: &str = "animus.dogfood-headless-shell.v1";
const HASH_SEPARATOR: &str = "|";
const RECEIPT_SCHEMA: &str = "animus.dogfood-headless-shell.receipt.v1";
const BLAKE3_ALGORITHM: &str = "BLAKE3";
const REQUIRED_EVENT_COUNT_WITHOUT_TOOL: usize = 5;
const REQUIRED_EVENT_COUNT_WITH_TOOL: usize = 6;
const EMPTY_COUNT: usize = 0;
const MIN_COUNT: u32 = 1;
const MAX_LABEL_CHARS: usize = 96;
const HASH_HEX_CHARS: usize = 64;
const PROMPT_ADMISSION_EVENT: &str = "prompt-admission";
const MODEL_REQUEST_EVENT: &str = "model-request";
const MODEL_RESPONSE_EVENT: &str = "model-response";
const TOOL_ADMISSION_EVENT: &str = "tool-admission";
const COMPLETION_EVENT: &str = "completion";
const SHUTDOWN_EVENT: &str = "shutdown";
const FAKE_BACKED_RUNG: &str = "fake-backed-integration";
const LIVE_PROVIDER_SURFACE: &str = "live-provider";
const PERSISTENCE_SURFACE: &str = "persistence";
const DAEMON_SURFACE: &str = "daemon";
const TUI_SURFACE: &str = "tui";
const PLUGIN_SURFACE: &str = "plugin-runtime";
const PRODUCTION_SURFACE: &str = "production-readiness";
const REFERENCE_PARITY_SURFACE: &str = "reference-parity";
const UNSAFE_MARKERS: &[&str] = &[
    "RAW_PROMPT:",
    "RAW_MODEL_OUTPUT:",
    "RAW_TOOL_BODY:",
    "PROVIDER_PAYLOAD:",
    "BEGIN PRIVATE KEY",
    "API_KEY=",
    "sk-",
    "/home/",
    "UNBOUNDED_LOG:",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DogfoodServiceState {
    Available,
    Missing,
    Denied,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DogfoodRunRequest {
    pub prompt_hash: String,
    pub policy_profile: String,
    pub model_capability: String,
    pub output_contract: String,
    pub tool_metadata: Option<String>,
    pub noninteractive: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DogfoodFakeServices {
    pub model_service: DogfoodServiceState,
    pub tool_service: DogfoodServiceState,
    pub allowed_capabilities: Vec<String>,
    pub model_response_summary: String,
    pub tool_response_summary: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DogfoodEvent {
    pub sequence: u32,
    pub kind: String,
    pub label_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DogfoodReceipt {
    pub schema: String,
    pub status: DogfoodStatus,
    pub event_count: usize,
    pub adapter_labels: Vec<String>,
    pub receipt_hash: String,
    pub prompt_hash: String,
    pub redaction: DogfoodRedaction,
    pub evidence_rung: String,
    pub omitted_surfaces: Vec<String>,
    pub issue_codes: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DogfoodRedaction {
    Redacted,
    Unsafe,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DogfoodStatus {
    Completed,
    MissingService,
    DeniedCapability,
    MalformedOutput,
    UnsafeEvidence,
    ReceiptRejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DogfoodIssue {
    EmptyField,
    InvalidHash,
    UnsafeEvidence,
    MissingModelService,
    MissingToolService,
    DeniedCapability,
    MalformedOutput,
    ReceiptLeak,
    Overclaim,
    EventOrder,
}

impl DogfoodIssue {
    pub const fn code(self) -> &'static str {
        match self {
            Self::EmptyField => "empty-field",
            Self::InvalidHash => "invalid-hash",
            Self::UnsafeEvidence => "unsafe-evidence",
            Self::MissingModelService => "missing-model-service",
            Self::MissingToolService => "missing-tool-service",
            Self::DeniedCapability => "denied-capability",
            Self::MalformedOutput => "malformed-output",
            Self::ReceiptLeak => "receipt-leak",
            Self::Overclaim => "overclaim",
            Self::EventOrder => "event-order",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DogfoodValidationReport {
    pub valid: bool,
    pub issues: Vec<DogfoodIssue>,
}

pub fn run_fake_dogfood_shell(
    request: &DogfoodRunRequest,
    services: &DogfoodFakeServices,
) -> Result<(Vec<DogfoodEvent>, DogfoodReceipt), DogfoodValidationReport> {
    let admission = admit_dogfood_run(request, services);
    if !admission.valid {
        return Err(admission);
    }

    let event_kinds = event_kinds(request.tool_metadata.is_some());
    let events = event_kinds
        .iter()
        .enumerate()
        .map(|(index, kind)| DogfoodEvent {
            sequence: u32::try_from(index).map_or(MIN_COUNT, |value| value + MIN_COUNT),
            kind: (*kind).to_owned(),
            label_hash: dogfood_hash(kind, request.prompt_hash.as_bytes()),
        })
        .collect::<Vec<_>>();

    let receipt = project_dogfood_receipt(request, services, &events);
    let receipt_report = validate_dogfood_receipt(&receipt);
    if !receipt_report.valid {
        return Err(receipt_report);
    }

    Ok((events, receipt))
}

pub fn admit_dogfood_run(request: &DogfoodRunRequest, services: &DogfoodFakeServices) -> DogfoodValidationReport {
    let mut issues = Vec::new();
    validate_hash(request.prompt_hash.as_str(), &mut issues);
    validate_label(request.policy_profile.as_str(), &mut issues);
    validate_label(request.model_capability.as_str(), &mut issues);
    validate_label(request.output_contract.as_str(), &mut issues);
    if let Some(tool_metadata) = &request.tool_metadata {
        validate_label(tool_metadata.as_str(), &mut issues);
    }
    validate_label(services.model_response_summary.as_str(), &mut issues);
    if let Some(tool_response) = &services.tool_response_summary {
        validate_label(tool_response.as_str(), &mut issues);
    }
    if services.model_service != DogfoodServiceState::Available {
        issues.push(DogfoodIssue::MissingModelService);
    }
    if request.tool_metadata.is_some() && services.tool_service != DogfoodServiceState::Available {
        issues.push(DogfoodIssue::MissingToolService);
    }
    if !services.allowed_capabilities.iter().any(|capability| capability == &request.model_capability) {
        issues.push(DogfoodIssue::DeniedCapability);
    }
    if services.model_response_summary.trim().is_empty() || services.model_response_summary.contains("MALFORMED") {
        issues.push(DogfoodIssue::MalformedOutput);
    }
    validation_report(issues)
}

pub fn project_dogfood_receipt(
    request: &DogfoodRunRequest,
    services: &DogfoodFakeServices,
    events: &[DogfoodEvent],
) -> DogfoodReceipt {
    let adapter_labels = adapter_labels(request, services);
    let omitted_surfaces = required_non_claims();
    let issue_codes = validate_event_order(events, request.tool_metadata.is_some())
        .issues
        .into_iter()
        .map(|issue| issue.code().to_owned())
        .collect::<Vec<_>>();
    let mut receipt = DogfoodReceipt {
        schema: RECEIPT_SCHEMA.to_owned(),
        status: DogfoodStatus::Completed,
        event_count: events.len(),
        adapter_labels,
        receipt_hash: String::new(),
        prompt_hash: request.prompt_hash.clone(),
        redaction: DogfoodRedaction::Redacted,
        evidence_rung: FAKE_BACKED_RUNG.to_owned(),
        omitted_surfaces,
        issue_codes,
    };
    receipt.receipt_hash = receipt_hash(&receipt);
    receipt
}

pub fn validate_dogfood_receipt(receipt: &DogfoodReceipt) -> DogfoodValidationReport {
    let mut issues = Vec::new();
    if receipt.schema != RECEIPT_SCHEMA {
        issues.push(DogfoodIssue::EmptyField);
    }
    validate_hash(receipt.prompt_hash.as_str(), &mut issues);
    validate_hash(receipt.receipt_hash.as_str(), &mut issues);
    if receipt.redaction != DogfoodRedaction::Redacted {
        issues.push(DogfoodIssue::ReceiptLeak);
    }
    if receipt.event_count == EMPTY_COUNT {
        issues.push(DogfoodIssue::EventOrder);
    }
    if receipt.evidence_rung != FAKE_BACKED_RUNG {
        issues.push(DogfoodIssue::Overclaim);
    }
    for label in &receipt.adapter_labels {
        validate_label(label.as_str(), &mut issues);
    }
    for surface in required_non_claims() {
        if !receipt.omitted_surfaces.iter().any(|omitted| omitted == &surface) {
            issues.push(DogfoodIssue::Overclaim);
        }
    }
    validation_report(issues)
}

fn event_kinds(include_tool: bool) -> Vec<&'static str> {
    let mut kinds = vec![PROMPT_ADMISSION_EVENT, MODEL_REQUEST_EVENT, MODEL_RESPONSE_EVENT];
    if include_tool {
        kinds.push(TOOL_ADMISSION_EVENT);
    }
    kinds.push(COMPLETION_EVENT);
    kinds.push(SHUTDOWN_EVENT);
    kinds
}

fn adapter_labels(request: &DogfoodRunRequest, services: &DogfoodFakeServices) -> Vec<String> {
    let mut labels = vec![
        request.policy_profile.clone(),
        request.model_capability.clone(),
        services.model_response_summary.clone(),
    ];
    if let Some(tool_metadata) = &request.tool_metadata {
        labels.push(tool_metadata.clone());
    }
    labels
}

fn required_non_claims() -> Vec<String> {
    [
        LIVE_PROVIDER_SURFACE,
        PERSISTENCE_SURFACE,
        DAEMON_SURFACE,
        TUI_SURFACE,
        PLUGIN_SURFACE,
        PRODUCTION_SURFACE,
        REFERENCE_PARITY_SURFACE,
    ]
    .iter()
    .map(|surface| (*surface).to_owned())
    .collect()
}

fn validate_event_order(events: &[DogfoodEvent], include_tool: bool) -> DogfoodValidationReport {
    let expected = event_kinds(include_tool);
    let expected_count = if include_tool {
        REQUIRED_EVENT_COUNT_WITH_TOOL
    } else {
        REQUIRED_EVENT_COUNT_WITHOUT_TOOL
    };
    let mut issues = Vec::new();
    if events.len() != expected_count {
        issues.push(DogfoodIssue::EventOrder);
    }
    for (index, expected_kind) in expected.iter().enumerate() {
        if events.get(index).is_none_or(|event| event.kind.as_str() != *expected_kind) {
            issues.push(DogfoodIssue::EventOrder);
        }
    }
    validation_report(issues)
}

fn validate_label(value: &str, issues: &mut Vec<DogfoodIssue>) {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_LABEL_CHARS {
        issues.push(DogfoodIssue::EmptyField);
    }
    if UNSAFE_MARKERS.iter().any(|marker| trimmed.contains(marker)) {
        issues.push(DogfoodIssue::UnsafeEvidence);
    }
}

fn validate_hash(value: &str, issues: &mut Vec<DogfoodIssue>) {
    if value.len() != HASH_HEX_CHARS || !value.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        issues.push(DogfoodIssue::InvalidHash);
    }
}

fn dogfood_hash(label: &str, bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(HASH_DOMAIN.as_bytes());
    hasher.update(HASH_SEPARATOR.as_bytes());
    hasher.update(label.as_bytes());
    hasher.update(HASH_SEPARATOR.as_bytes());
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

fn receipt_hash(receipt: &DogfoodReceipt) -> String {
    let canonical = format!(
        "{}{}{}{}{}{}{}{}{}{}{}",
        receipt.schema,
        HASH_SEPARATOR,
        receipt.prompt_hash,
        HASH_SEPARATOR,
        receipt.event_count,
        HASH_SEPARATOR,
        receipt.evidence_rung,
        HASH_SEPARATOR,
        receipt.adapter_labels.join(HASH_SEPARATOR),
        HASH_SEPARATOR,
        BLAKE3_ALGORITHM
    );
    dogfood_hash("receipt", canonical.as_bytes())
}

fn validation_report(mut issues: Vec<DogfoodIssue>) -> DogfoodValidationReport {
    issues.sort_by_key(|issue| issue.code());
    issues.dedup();
    DogfoodValidationReport {
        valid: issues.is_empty(),
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROMPT_BYTES: &[u8] = b"bounded prompt fixture";
    const POLICY_PROFILE: &str = "local-fake-policy";
    const MODEL_CAPABILITY: &str = "fake-model-text";
    const OUTPUT_CONTRACT: &str = "text-summary";
    const TOOL_METADATA: &str = "fake-readonly-tool";
    const MODEL_SUMMARY: &str = "fake-model-summary";
    const TOOL_SUMMARY: &str = "fake-tool-summary";

    #[test]
    fn fake_backed_prompt_emits_ordered_events_and_safe_receipt() {
        // r[verify agent_harness_runtime.dogfood_headless_shell.entrypoint]
        // r[verify agent_harness_runtime.dogfood_headless_shell.events]
        // r[verify agent_harness_runtime.dogfood_headless_shell.receipts]
        let request = request(Some(TOOL_METADATA));
        let services = services();

        let (events, receipt) = run_fake_dogfood_shell(&request, &services).expect("dogfood run");

        assert_eq!(events[0].kind, PROMPT_ADMISSION_EVENT);
        assert_eq!(events[1].kind, MODEL_REQUEST_EVENT);
        assert_eq!(events[2].kind, MODEL_RESPONSE_EVENT);
        assert_eq!(events[3].kind, TOOL_ADMISSION_EVENT);
        assert_eq!(events[4].kind, COMPLETION_EVENT);
        assert_eq!(events[5].kind, SHUTDOWN_EVENT);
        assert_eq!(receipt.status, DogfoodStatus::Completed);
        assert_eq!(receipt.event_count, REQUIRED_EVENT_COUNT_WITH_TOOL);
        assert_eq!(receipt.evidence_rung, FAKE_BACKED_RUNG);
        assert!(receipt.omitted_surfaces.contains(&LIVE_PROVIDER_SURFACE.to_owned()));
        assert!(receipt.omitted_surfaces.contains(&PRODUCTION_SURFACE.to_owned()));
        assert!(validate_dogfood_receipt(&receipt).valid);
    }

    #[test]
    fn admission_fails_closed_for_missing_denied_and_malformed_services() {
        // r[verify agent_harness_runtime.dogfood_headless_shell.admission]
        // r[verify agent_harness_runtime.dogfood_headless_shell.verification]
        let request = request(None);
        let mut missing = services();
        missing.model_service = DogfoodServiceState::Missing;
        let missing_report = admit_dogfood_run(&request, &missing);
        assert!(missing_report.issues.contains(&DogfoodIssue::MissingModelService));

        let mut denied = services();
        denied.allowed_capabilities.clear();
        let denied_report = admit_dogfood_run(&request, &denied);
        assert!(denied_report.issues.contains(&DogfoodIssue::DeniedCapability));

        let mut malformed = services();
        malformed.model_response_summary = "MALFORMED".to_owned();
        let malformed_report = admit_dogfood_run(&request, &malformed);
        assert!(malformed_report.issues.contains(&DogfoodIssue::MalformedOutput));
    }

    #[test]
    fn receipt_validation_rejects_leaks_and_overclaims() {
        // r[verify agent_harness_runtime.dogfood_headless_shell.receipts.leak_rejection]
        // r[verify agent_harness_runtime.dogfood_headless_shell.docs]
        let request = request(None);
        let services = services();
        let events = event_kinds(false)
            .iter()
            .enumerate()
            .map(|(index, kind)| DogfoodEvent {
                sequence: u32::try_from(index).expect("test index fits") + MIN_COUNT,
                kind: (*kind).to_owned(),
                label_hash: dogfood_hash(kind, request.prompt_hash.as_bytes()),
            })
            .collect::<Vec<_>>();
        let mut receipt = project_dogfood_receipt(&request, &services, &events);
        receipt.adapter_labels.push("RAW_PROMPT:secret".to_owned());
        receipt.redaction = DogfoodRedaction::Unsafe;
        receipt.evidence_rung = "production-ready".to_owned();
        receipt.omitted_surfaces.retain(|surface| surface != LIVE_PROVIDER_SURFACE);

        let report = validate_dogfood_receipt(&receipt);

        assert!(report.issues.contains(&DogfoodIssue::UnsafeEvidence));
        assert!(report.issues.contains(&DogfoodIssue::ReceiptLeak));
        assert!(report.issues.contains(&DogfoodIssue::Overclaim));
    }

    fn request(tool_metadata: Option<&str>) -> DogfoodRunRequest {
        DogfoodRunRequest {
            prompt_hash: dogfood_hash("prompt", PROMPT_BYTES),
            policy_profile: POLICY_PROFILE.to_owned(),
            model_capability: MODEL_CAPABILITY.to_owned(),
            output_contract: OUTPUT_CONTRACT.to_owned(),
            tool_metadata: tool_metadata.map(str::to_owned),
            noninteractive: true,
        }
    }

    fn services() -> DogfoodFakeServices {
        DogfoodFakeServices {
            model_service: DogfoodServiceState::Available,
            tool_service: DogfoodServiceState::Available,
            allowed_capabilities: vec![MODEL_CAPABILITY.to_owned()],
            model_response_summary: MODEL_SUMMARY.to_owned(),
            tool_response_summary: Some(TOOL_SUMMARY.to_owned()),
        }
    }
}
