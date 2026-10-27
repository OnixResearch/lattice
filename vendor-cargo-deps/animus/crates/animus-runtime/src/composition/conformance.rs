//! Composition conformance matrix and deterministic in-memory rail.
//!
//! The rail combines existing provider, adapter, headless, middleware,
//! provenance, and tool-dispatch contracts without constructing live providers,
//! processes, terminals, filesystems, networks, or persistence backends.

#![allow(unknown_lints)]
#![allow(
    assertion_density,
    clippy::missing_panics_doc,
    clippy::similar_names,
    clippy::struct_field_names,
    function_length,
    non_trait_imports,
    path_segment_repetition,
    unbounded_collection_growth
)]

const TOOL_NEGATIVE_CHECK_COUNT: usize = 6;
const CAPABILITY_CHECK_COUNT: usize = 4;

const HASH_DOMAIN: &str = "animus.composition-conformance.v1";
const HASH_ALGORITHM: &str = "BLAKE3";
const HASH_SEPARATOR: &str = "|";
const RAIL_NAME: &str = "composition-conformance";
const RAIL_SCHEMA: &str = "animus.focused-rail.v1";
const STATUS_PASS: &str = "pass";
const STATUS_REDACTED: &str = "redacted";
const OWNER_RUNTIME: &str = "animus-runtime";
const FEATURE_RUNTIME: &str = "animus-runtime";
const FEATURE_ADAPTERS: &str = "animus-adapters";
const FEATURE_PROVIDER_HARDENING: &str = "provider-hardening";
const FEATURE_PROVENANCE: &str = "provenance";
const FEATURE_HEADLESS: &str = "headless";
const FEATURE_MIDDLEWARE: &str = "middleware";
const FEATURE_TOOL_CONTRACTS: &str = "tool-contracts";
const FEATURE_CAPABILITY_GATE: &str = "capability-gate";
const SAFE_PROMPT_SUMMARY: &str = "intent-summary";
const SAFE_MODEL_MESSAGE: &str = "bounded-model-answer";
const SAFE_PROVIDER_ID: &str = "provider.local";
const SAFE_MODEL_ID: &str = "model.agent";
const SAFE_MODEL_DISPLAY: &str = "Agent Model";
const SAFE_CONTEXT_BUDGET: &str = "budget.small";
const SAFE_ENDPOINT: &str = "https://provider.invalid/v1";
const SAFE_EXTRA_BODY_ID: &str = "extra.reasoning";
const TOOL_NAME: &str = "lookup";
const TOOL_INPUT_SUMMARY: &str = "query-summary";
const TOOL_ARGUMENT_NAME: &str = "query";
const TOOL_CALL_ID: &str = "tool-call-lookup";
const TOOL_MUTATION_TARGET: &str = "resource:fixture-record";
const TOOL_OUTPUT_SUMMARY: &str = "lookup";
const TOOL_LABEL: &str = "Lookup";
const TOOL_DESCRIPTION: &str = "Read a deterministic in-memory fixture";
const TOOL_SCHEMA_ID: &str = "schema.lookup.v1";
const TOOL_PROMPT_SNIPPET: &str = "lookup available";
const TOOL_GUIDELINE: &str = "return bounded summaries";
const TOOL_SOURCE_ID: &str = "fixture.registry";
const TOOL_PROVENANCE: &str = "fixture.provenance";
const TOOL_CAPABILITY: &str = "tool.lookup";
const TOOL_RESULT_SHAPE: &str = "shape.summary";
const TOOL_SERVICE_LABEL: &str = "in-memory-tool-service";
const TOOL_EXECUTOR_BINDING: &str = "in-memory-tool-executor";
const TOOL_EXECUTOR_MAX_OUTPUT_BYTES: u64 = 64;
const TOOL_EFFECT_ABILITY: &str = "read-summary";
const TOOL_EFFECT_RESOURCE: &str = "fixture-record";
const TOOL_RECEIPT_ID: &str = "receipt-tool-lookup";
const TOOL_ARTIFACT_HASH: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const MIDDLEWARE_HOOK: &str = "composition_guard";
const MIDDLEWARE_REASON: &str = "policy_denied";
const MIDDLEWARE_FAILURE: &str = "guard_failed";
const HEADLESS_UNSAFE_RECORD: &str = "req-unsafe|prompt|new_turn|RAW_PROMPT: secret=abc";
const HEADLESS_SAFE_DENIAL: &str = "policy_denied";
const REPORT_SAFE_SECTION: &str = "safe_evidence";
const PRIVATE_PATH_MARKER: &str = "/home/";
const RAW_PROMPT_MARKER: &str = "RAW_PROMPT:";
const PROVIDER_PAYLOAD_MARKER: &str = "PROVIDER_PAYLOAD:";
const UNBOUNDED_OUTPUT_MARKER: &str = "UNBOUNDED_OUTPUT:";
const PRIVATE_KEY_MARKER: &str = "BEGIN PRIVATE KEY";
const API_KEY_MARKER: &str = "API_KEY=";
const AWS_SECRET_MARKER: &str = "AWS_SECRET_ACCESS_KEY";
const OPENAI_SECRET_PREFIX: &str = "sk-";
const SMALL_OUTPUT_BUDGET_CHARS: u32 = 8;
const EXPECTED_FAKE_CALL_COUNT: usize = 1;

const REQUIRED_SURFACES: [CompositionSurface; REQUIRED_SURFACE_COUNT] = [
    CompositionSurface::ProviderModelDispatch,
    CompositionSurface::ToolProvenanceReceipt,
    CompositionSurface::HeadlessRuntimeEvents,
    CompositionSurface::MiddlewareFailureReport,
    CompositionSurface::CapabilityNoFallback,
];
const REQUIRED_SURFACE_COUNT: usize = 5;

const UNSAFE_EVIDENCE_MARKERS: [&str; UNSAFE_EVIDENCE_MARKER_COUNT] = [
    RAW_PROMPT_MARKER,
    PROVIDER_PAYLOAD_MARKER,
    UNBOUNDED_OUTPUT_MARKER,
    PRIVATE_KEY_MARKER,
    API_KEY_MARKER,
    AWS_SECRET_MARKER,
    OPENAI_SECRET_PREFIX,
    PRIVATE_PATH_MARKER,
];
const UNSAFE_EVIDENCE_MARKER_COUNT: usize = 8;

/// Cross-module surface covered by the conformance matrix.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CompositionSurface {
    /// Provider hardening plus model dispatch.
    ProviderModelDispatch,
    /// Tool contracts plus provenance receipts.
    ToolProvenanceReceipt,
    /// Headless protocol projection plus runtime events.
    HeadlessRuntimeEvents,
    /// Middleware denials plus failure reports.
    MiddlewareFailureReport,
    /// Capability absence plus no ambient fallback.
    CapabilityNoFallback,
}

impl CompositionSurface {
    /// Returns the stable surface label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProviderModelDispatch => "provider_model_dispatch",
            Self::ToolProvenanceReceipt => "tool_provenance_receipt",
            Self::HeadlessRuntimeEvents => "headless_runtime_events",
            Self::MiddlewareFailureReport => "middleware_failure_report",
            Self::CapabilityNoFallback => "capability_no_fallback",
        }
    }
}

/// Positive or negative conformance case class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompositionCaseKind {
    /// Valid inputs compose across module boundaries.
    Positive,
    /// Invalid inputs fail closed across module boundaries.
    Negative,
}

impl CompositionCaseKind {
    /// Returns the stable case-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Positive => "positive",
            Self::Negative => "negative",
        }
    }
}

/// Validation or execution diagnostic class for the conformance rail.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompositionDiagnosticKind {
    /// Matrix entry is missing an owner.
    MissingOwner,
    /// Matrix entry has no feature requirements.
    MissingFeatureRequirements,
    /// Matrix entry is missing a positive case label.
    MissingPositiveCase,
    /// Matrix entry is missing a negative case label.
    MissingNegativeCase,
    /// Matrix entry is missing a safe evidence label.
    MissingSafeEvidenceLabel,
    /// Required surface is absent from the matrix.
    MissingRequiredSurface,
    /// Scenario execution failed.
    ScenarioFailed,
    /// Evidence text contains unsafe material.
    UnsafeEvidence,
}

impl CompositionDiagnosticKind {
    /// Returns the stable diagnostic label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingOwner => "missing_owner",
            Self::MissingFeatureRequirements => "missing_feature_requirements",
            Self::MissingPositiveCase => "missing_positive_case",
            Self::MissingNegativeCase => "missing_negative_case",
            Self::MissingSafeEvidenceLabel => "missing_safe_evidence_label",
            Self::MissingRequiredSurface => "missing_required_surface",
            Self::ScenarioFailed => "scenario_failed",
            Self::UnsafeEvidence => "unsafe_evidence",
        }
    }
}

/// One conformance matrix entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionScenario {
    /// Stable scenario identity.
    pub id: String,
    /// Cross-module surface covered by the scenario.
    pub surface: CompositionSurface,
    /// Owning crate, document, or team label.
    pub owner: String,
    /// Required feature or module labels.
    pub feature_requirements: Vec<String>,
    /// Positive case label.
    pub positive_case: String,
    /// Negative case label.
    pub negative_case: String,
    /// Safe evidence label used in receipts.
    pub safe_evidence_label: String,
}

/// One bounded evidence row emitted by the conformance rail.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionEvidenceRecord {
    /// Matrix scenario identity.
    pub scenario_id: String,
    /// Cross-module surface.
    pub surface: CompositionSurface,
    /// Positive or negative case class.
    pub case_kind: CompositionCaseKind,
    /// Bounded outcome label.
    pub outcome: String,
    /// Safe evidence label.
    pub evidence_label: String,
    /// BLAKE3 hash over safe record fields.
    pub evidence_hash: String,
}

/// Bounded conformance rail report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionRailReport {
    /// Number of matrix scenarios.
    pub scenario_count: usize,
    /// Number of positive evidence records.
    pub positive_count: usize,
    /// Number of negative evidence records.
    pub negative_count: usize,
    /// Per-case evidence rows.
    pub records: Vec<CompositionEvidenceRecord>,
    /// BLAKE3 hash over safe report fields.
    pub report_hash: String,
}

/// Conformance validation or execution diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionDiagnostic {
    /// Diagnostic kind.
    pub kind: CompositionDiagnosticKind,
    /// Scenario or surface label.
    pub subject: String,
    /// Bounded diagnostic detail.
    pub detail: String,
}

impl CompositionDiagnostic {
    /// Builds a conformance diagnostic.
    #[must_use]
    pub fn new(
        kind: CompositionDiagnosticKind,
        subject: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
            detail: detail.into(),
        }
    }
}

/// Returns the maintained composition conformance matrix.
#[must_use]
pub fn composition_conformance_matrix() -> Vec<CompositionScenario> {
    vec![
        scenario(ScenarioSeed {
            id: "provider-model-dispatch",
            surface: CompositionSurface::ProviderModelDispatch,
            features: &[
                FEATURE_RUNTIME,
                FEATURE_ADAPTERS,
                FEATURE_PROVIDER_HARDENING,
            ],
            positive_case: "text-agent-provider-selection-dispatches-model",
            negative_case: "malformed-response-and-disabled-model-class-fail-closed",
            safe_evidence_label: "provider_model_dispatch_safe",
        }),
        scenario(ScenarioSeed {
            id: "tool-provenance-receipt",
            surface: CompositionSurface::ToolProvenanceReceipt,
            features: &[FEATURE_ADAPTERS, FEATURE_PROVENANCE, FEATURE_TOOL_CONTRACTS],
            positive_case: "bounded-tool-output-produces-safe-provenance-receipt",
            negative_case: "unauthorized-oversized-and-unsafe-receipts-fail-closed",
            safe_evidence_label: "tool_provenance_receipt_safe",
        }),
        scenario(ScenarioSeed {
            id: "headless-runtime-events",
            surface: CompositionSurface::HeadlessRuntimeEvents,
            features: &[FEATURE_RUNTIME, FEATURE_HEADLESS],
            positive_case: "runtime-events-project-to-headless-safe-summaries",
            negative_case: "unsafe-headless-records-redact-payloads",
            safe_evidence_label: "headless_runtime_events_safe",
        }),
        scenario(ScenarioSeed {
            id: "middleware-failure-report",
            surface: CompositionSurface::MiddlewareFailureReport,
            features: &[FEATURE_RUNTIME, FEATURE_MIDDLEWARE],
            positive_case: "middleware-accepts-safe-dispatch",
            negative_case: "middleware-denials-and-failures-produce-failed-reports",
            safe_evidence_label: "middleware_failure_report_safe",
        }),
        scenario(ScenarioSeed {
            id: "capability-no-fallback",
            surface: CompositionSurface::CapabilityNoFallback,
            features: &[FEATURE_ADAPTERS, FEATURE_CAPABILITY_GATE],
            positive_case: "available-capability-dispatches-without-fallback",
            negative_case: "missing-service-and-denied-capability-do-not-fallback",
            safe_evidence_label: "capability_no_fallback_safe",
        }),
    ]
}

/// Validates matrix structure and required surface coverage.
///
/// # Errors
///
/// Returns diagnostics when an entry lacks owner, feature, positive, negative,
/// or safe-evidence metadata, or when a required surface is absent.
pub fn validate_composition_matrix(
    matrix: &[CompositionScenario],
) -> Result<(), Vec<CompositionDiagnostic>> {
    let mut diagnostics = Vec::new();
    for entry in matrix {
        diagnostics.extend(validate_matrix_entry(entry));
    }
    diagnostics.extend(validate_required_surfaces(matrix));
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

/// Runs all deterministic in-memory conformance scenarios.
///
/// # Errors
///
/// Returns diagnostics when matrix validation fails, a scenario fails, or the
/// rendered evidence would contain unsafe material.
pub fn run_composition_conformance() -> Result<CompositionRailReport, Vec<CompositionDiagnostic>> {
    let matrix = composition_conformance_matrix();
    validate_composition_matrix(&matrix)?;
    let mut records = Vec::new();
    let mut diagnostics = Vec::new();
    for entry in &matrix {
        match evaluate_scenario(entry) {
            Ok(mut scenario_records) => records.append(&mut scenario_records),
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let scenario_count = matrix.len();
    let rail_snapshot = report_from_records(scenario_count, records);
    let rendered = render_composition_evidence(&rail_snapshot);
    validate_composition_evidence_text(&rendered)?;
    Ok(rail_snapshot)
}

/// Renders bounded safe evidence for a conformance report.
#[must_use]
pub fn render_composition_evidence(report: &CompositionRailReport) -> String {
    let mut rendered = String::new();
    rendered.push_str("schema: ");
    rendered.push_str(RAIL_SCHEMA);
    rendered.push_str("\nrail: ");
    rendered.push_str(RAIL_NAME);
    rendered.push_str("\nstatus: ");
    rendered.push_str(STATUS_PASS);
    rendered.push_str("\nscenario_count: ");
    rendered.push_str(&report.scenario_count.to_string());
    rendered.push_str("\npositive_count: ");
    rendered.push_str(&report.positive_count.to_string());
    rendered.push_str("\nnegative_count: ");
    rendered.push_str(&report.negative_count.to_string());
    rendered.push_str("\nreport_hash: ");
    rendered.push_str(&report.report_hash);
    rendered.push_str("\nrecords:\n");
    for record in &report.records {
        rendered.push_str("- scenario=");
        rendered.push_str(&record.scenario_id);
        rendered.push_str(" surface=");
        rendered.push_str(record.surface.as_str());
        rendered.push_str(" class=");
        rendered.push_str(record.case_kind.as_str());
        rendered.push_str(" outcome=");
        rendered.push_str(&record.outcome);
        rendered.push_str(" label=");
        rendered.push_str(&record.evidence_label);
        rendered.push_str(" hash=");
        rendered.push_str(&record.evidence_hash);
        rendered.push('\n');
    }
    rendered.push_str(REPORT_SAFE_SECTION);
    rendered.push_str(":\n");
    rendered.push_str("- scenario names, case classes, counts, labels, and hashes only\n");
    rendered.push_str("- raw prompts, provider payloads, credentials, private paths, and unbounded tool outputs omitted\n");
    rendered
}

/// Validates that rendered conformance evidence omits unsafe material.
///
/// # Errors
///
/// Returns a diagnostic when the text contains known unsafe markers.
pub fn validate_composition_evidence_text(text: &str) -> Result<(), Vec<CompositionDiagnostic>> {
    for marker in UNSAFE_EVIDENCE_MARKERS {
        if text.contains(marker) {
            return Err(vec![CompositionDiagnostic::new(
                CompositionDiagnosticKind::UnsafeEvidence,
                RAIL_NAME,
                format!("evidence contains unsafe marker {marker}"),
            )]);
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct ScenarioSeed<'a> {
    id: &'a str,
    surface: CompositionSurface,
    features: &'a [&'a str],
    positive_case: &'a str,
    negative_case: &'a str,
    safe_evidence_label: &'a str,
}

fn scenario(seed: ScenarioSeed<'_>) -> CompositionScenario {
    CompositionScenario {
        id: String::from(seed.id),
        surface: seed.surface,
        owner: String::from(OWNER_RUNTIME),
        feature_requirements: seed
            .features
            .iter()
            .map(|feature| String::from(*feature))
            .collect(),
        positive_case: String::from(seed.positive_case),
        negative_case: String::from(seed.negative_case),
        safe_evidence_label: String::from(seed.safe_evidence_label),
    }
}

fn validate_matrix_entry(entry: &CompositionScenario) -> Vec<CompositionDiagnostic> {
    let mut diagnostics = Vec::new();
    if entry.owner.trim().is_empty() {
        diagnostics.push(matrix_issue(
            CompositionDiagnosticKind::MissingOwner,
            entry,
            "owner is required",
        ));
    }
    if entry.feature_requirements.is_empty() {
        diagnostics.push(matrix_issue(
            CompositionDiagnosticKind::MissingFeatureRequirements,
            entry,
            "feature requirements are required",
        ));
    }
    if entry.positive_case.trim().is_empty() {
        diagnostics.push(matrix_issue(
            CompositionDiagnosticKind::MissingPositiveCase,
            entry,
            "positive case is required",
        ));
    }
    if entry.negative_case.trim().is_empty() {
        diagnostics.push(matrix_issue(
            CompositionDiagnosticKind::MissingNegativeCase,
            entry,
            "negative case is required",
        ));
    }
    if entry.safe_evidence_label.trim().is_empty() {
        diagnostics.push(matrix_issue(
            CompositionDiagnosticKind::MissingSafeEvidenceLabel,
            entry,
            "safe evidence label is required",
        ));
    }
    diagnostics
}

fn validate_required_surfaces(matrix: &[CompositionScenario]) -> Vec<CompositionDiagnostic> {
    let mut diagnostics = Vec::new();
    for required in REQUIRED_SURFACES {
        if !matrix.iter().any(|entry| entry.surface == required) {
            diagnostics.push(CompositionDiagnostic::new(
                CompositionDiagnosticKind::MissingRequiredSurface,
                required.as_str(),
                "required conformance surface is missing",
            ));
        }
    }
    diagnostics
}

fn matrix_issue(
    kind: CompositionDiagnosticKind,
    entry: &CompositionScenario,
    detail: &str,
) -> CompositionDiagnostic {
    CompositionDiagnostic::new(kind, entry.id.clone(), detail)
}

fn evaluate_scenario(
    entry: &CompositionScenario,
) -> Result<Vec<CompositionEvidenceRecord>, CompositionDiagnostic> {
    match entry.surface {
        CompositionSurface::ProviderModelDispatch => provider_model_dispatch(entry),
        CompositionSurface::ToolProvenanceReceipt => tool_provenance_receipt(entry),
        CompositionSurface::HeadlessRuntimeEvents => headless_runtime_events(entry),
        CompositionSurface::MiddlewareFailureReport => middleware_failure_report(entry),
        CompositionSurface::CapabilityNoFallback => capability_no_fallback(entry),
    }
}

fn provider_model_dispatch(
    entry: &CompositionScenario,
) -> Result<Vec<CompositionEvidenceRecord>, CompositionDiagnostic> {
    let style = crate::resolve_provider_api_style(
        Some(crate::ProviderApiStyle::Responses),
        false,
        crate::ProviderApiStylePolicy::RequireExplicit,
    )
    .map_err(|error| scenario_failure(entry, format!("provider style: {error:?}")))?;
    let model = text_agent_model();
    let selected = crate::filter_agent_model(&model, false)
        .map_err(|error| scenario_failure(entry, format!("model filter: {error:?}")))?;
    let request = animus_adapters::ModelRequest::new(SAFE_PROMPT_SUMMARY)
        .map_err(|error| scenario_failure(entry, format!("model request: {error:?}")))?;
    let mut adapter = FakeModelAdapter::new(SAFE_MODEL_MESSAGE);
    let response = animus_adapters::ComponentAdapter::call(&mut adapter, &request)
        .map_err(|error| scenario_failure(entry, error))?;
    if response.message() != SAFE_MODEL_MESSAGE || adapter.calls != EXPECTED_FAKE_CALL_COUNT {
        return Err(scenario_failure(
            entry,
            "fake model adapter did not dispatch once",
        ));
    }
    let receipt = crate::provider_selection_receipt(&crate::ProviderSelectionReceiptInput {
        provider_id: String::from(SAFE_PROVIDER_ID),
        model: selected,
        endpoint: String::from(SAFE_ENDPOINT),
        sensitivity: crate::ProviderEndpointSensitivity::Sensitive,
        api_style: style,
        extra_body_descriptor_ids: vec![String::from(SAFE_EXTRA_BODY_ID)],
        diagnostics: Vec::new(),
    });
    if !receipt.endpoint_identity.starts_with(HASH_ALGORITHM) {
        return Err(scenario_failure(
            entry,
            "sensitive provider endpoint was not hashed",
        ));
    }

    let is_malformed_response_rejected = animus_adapters::ModelResponse::new("").is_err();
    let is_disabled_model_rejected = crate::filter_agent_model(&image_model(), false).is_err();
    let is_ambiguous_style_rejected = crate::resolve_provider_api_style(
        None,
        false,
        crate::ProviderApiStylePolicy::RequireExplicit,
    )
    .is_err();
    if !is_malformed_response_rejected
        || !is_disabled_model_rejected
        || !is_ambiguous_style_rejected
    {
        return Err(scenario_failure(
            entry,
            "provider/model negative cases did not fail closed",
        ));
    }

    Ok(vec![
        record(
            entry,
            CompositionCaseKind::Positive,
            "model_dispatch_completed",
        ),
        record(
            entry,
            CompositionCaseKind::Negative,
            "malformed_response_rejected",
        ),
        record(
            entry,
            CompositionCaseKind::Negative,
            "disabled_feature_surface_rejected",
        ),
    ])
}

fn tool_provenance_receipt(
    entry: &CompositionScenario,
) -> Result<Vec<CompositionEvidenceRecord>, CompositionDiagnostic> {
    let positive = tool_provenance_positive(entry)?;
    tool_provenance_negative(entry, &positive.definition, positive.output_byte_count)?;
    Ok(vec![
        record(
            entry,
            CompositionCaseKind::Positive,
            "tool_receipt_completed",
        ),
        record(
            entry,
            CompositionCaseKind::Negative,
            "unauthorized_tool_denied",
        ),
        record(
            entry,
            CompositionCaseKind::Negative,
            "oversized_output_rejected",
        ),
        record(
            entry,
            CompositionCaseKind::Negative,
            "unsafe_receipt_rejected",
        ),
    ])
}

struct ToolProvenancePositive {
    definition: animus_adapters::tool::Definition,
    output_byte_count: u64,
}

fn tool_provenance_positive(
    entry: &CompositionScenario,
) -> Result<ToolProvenancePositive, CompositionDiagnostic> {
    let definition = tool_definition(entry)?;
    let request = tool_request(entry)?;
    let mut adapter = FakeToolAdapter::new(TOOL_OUTPUT_SUMMARY);
    let response = animus_adapters::ComponentAdapter::call(&mut adapter, &request)
        .map_err(|error| scenario_failure(entry, error))?;
    let output = bounded_tool_output(
        TOOL_OUTPUT_SUMMARY,
        animus_adapters::tool::HashRequirement::Required,
    )
    .map_err(|error| scenario_failure(entry, format!("bounded output: {error:?}")))?;
    if response.output() != output.visible_text() || adapter.calls != EXPECTED_FAKE_CALL_COUNT {
        return Err(scenario_failure(
            entry,
            "fake tool adapter did not return bounded output",
        ));
    }
    let kit_receipt = tool_executor_kit_receipt(entry, definition.clone(), &request)?;
    if kit_receipt.status() != crate::tool_executor_kit::ToolExecutionReceiptStatus::Completed {
        return Err(scenario_failure(
            entry,
            "tool executor kit did not produce a completed safe receipt",
        ));
    }
    let effect =
        completed_tool_effect(entry, Some(output.byte_count()), Some(output.byte_count()))?;
    let (_fact, receipt) = effect
        .completed_evidence(TOOL_RECEIPT_ID, TOOL_ARTIFACT_HASH)
        .map_err(|error| scenario_failure(entry, format!("completed evidence: {error:?}")))?;
    if receipt.artifact_hashes.is_empty() || receipt.redaction_markers.is_empty() {
        return Err(scenario_failure(
            entry,
            "receipt omitted artifact hash or redaction marker",
        ));
    }
    Ok(ToolProvenancePositive {
        definition,
        output_byte_count: output.byte_count(),
    })
}

fn tool_provenance_negative(
    entry: &CompositionScenario,
    definition: &animus_adapters::tool::Definition,
    output_byte_count: u64,
) -> Result<(), CompositionDiagnostic> {
    let denied = dispatch_decision(
        Some(definition),
        animus_adapters::tool::ExecutorStatus::Available,
        animus_adapters::tool::CapabilityStatus::Denied,
        animus_adapters::tool::OutputBoundaryStatus::Bounded,
    );
    let output_boundary_decision = dispatch_decision(
        Some(definition),
        animus_adapters::tool::ExecutorStatus::Available,
        animus_adapters::tool::CapabilityStatus::Allowed,
        animus_adapters::tool::OutputBoundaryStatus::UnboundedOversized,
    );
    let is_denial_failed_closed = failed_without_fallback(
        denied,
        Some(animus_adapters::tool::FailureClass::DeniedCapability),
    );
    let is_boundary_rejection_recorded = failed_without_fallback(
        output_boundary_decision,
        Some(animus_adapters::tool::FailureClass::UnboundedOversizedOutput),
    );
    let is_unsafe_receipt_rejected = unsafe_receipt_is_rejected(output_byte_count);
    let is_denied_effect_completion_rejected = denied_effect_refuses_completion(entry)?;
    let is_truncation_hashed = oversized_output_is_hashed(entry)?;
    let is_missing_kit_executor_denied = tool_executor_kit_missing_fails_closed(entry, definition)?;
    if all_checks_pass::<TOOL_NEGATIVE_CHECK_COUNT>([
        is_denial_failed_closed,
        is_boundary_rejection_recorded,
        is_unsafe_receipt_rejected,
        is_denied_effect_completion_rejected,
        is_truncation_hashed,
        is_missing_kit_executor_denied,
    ]) {
        return Ok(());
    }
    Err(scenario_failure(
        entry,
        "tool/provenance negative cases did not fail closed",
    ))
}

fn unsafe_receipt_is_rejected(output_byte_count: u64) -> bool {
    crate::ProvenanceReceipt::validate(&crate::ProvenanceReceiptDraft {
        id: TOOL_RECEIPT_ID,
        parents: &[],
        status: STATUS_PASS,
        issue_code: None,
        actor_label: RAW_PROMPT_MARKER,
        session_label: TOOL_SERVICE_LABEL,
        trace_label: HASH_DOMAIN,
        event_count: output_byte_count,
        artifact_hashes: &[TOOL_ARTIFACT_HASH],
        mutation_kind: None,
        effect_kind: Some(TOOL_EFFECT_ABILITY),
        backend_label: Some(TOOL_SERVICE_LABEL),
        interchange_label: None,
        redaction_markers: &[STATUS_REDACTED],
    })
    .is_err()
}

fn denied_effect_refuses_completion(
    entry: &CompositionScenario,
) -> Result<bool, CompositionDiagnostic> {
    let denied_effect = crate::ProvenanceToolEffect::validate(&crate::ProvenanceToolEffectDraft {
        tool_kind: TOOL_NAME,
        service_label: TOOL_SERVICE_LABEL,
        effect_ability: TOOL_EFFECT_ABILITY,
        effect_resource: TOOL_EFFECT_RESOURCE,
        status: crate::ProvenanceToolEffectStatus::CapabilityDenied,
        original_output_bytes: None,
        retained_output_bytes: None,
    })
    .map_err(|error| scenario_failure(entry, format!("denied effect: {error:?}")))?;
    Ok(denied_effect
        .completed_evidence(TOOL_RECEIPT_ID, TOOL_ARTIFACT_HASH)
        .is_err())
}

fn oversized_output_is_hashed(entry: &CompositionScenario) -> Result<bool, CompositionDiagnostic> {
    let truncated = bounded_tool_output(
        "oversized-result-body",
        animus_adapters::tool::HashRequirement::Required,
    )
    .map_err(|error| scenario_failure(entry, format!("truncated output: {error:?}")))?;
    Ok(
        truncated.status() == animus_adapters::tool::OutputStatus::Truncated
            && truncated.content_hash().is_some(),
    )
}

fn headless_runtime_events(
    entry: &CompositionScenario,
) -> Result<Vec<CompositionEvidenceRecord>, CompositionDiagnostic> {
    let request = tool_request(entry)?;
    let events = crate::map_runtime_events_to_headless(&[
        crate::RuntimeEvent::TurnStarted { turn_index: 0 },
        crate::RuntimeEvent::ToolDenied {
            request,
            reason: String::from(HEADLESS_SAFE_DENIAL),
        },
    ])
    .map_err(|error| scenario_failure(entry, format!("headless projection: {error:?}")))?;
    if events.is_empty()
        || events
            .iter()
            .any(|event| event.summary().contains(TOOL_INPUT_SUMMARY))
    {
        return Err(scenario_failure(
            entry,
            "headless event projection leaked tool input",
        ));
    }
    let unsafe_response = match crate::parse_headless_command_record(HEADLESS_UNSAFE_RECORD) {
        Ok(_command) => {
            return Err(scenario_failure(
                entry,
                "unsafe headless record was accepted",
            ));
        }
        Err(response) => response,
    };
    let is_error_redacted = unsafe_response
        .error()
        .is_some_and(|error| !error.message().contains("secret"));
    if !is_error_redacted {
        return Err(scenario_failure(
            entry,
            "unsafe headless record echoed unsafe payload",
        ));
    }
    Ok(vec![
        record(
            entry,
            CompositionCaseKind::Positive,
            "headless_projection_completed",
        ),
        record(
            entry,
            CompositionCaseKind::Negative,
            "unsafe_headless_record_redacted",
        ),
    ])
}

fn middleware_failure_report(
    entry: &CompositionScenario,
) -> Result<Vec<CompositionEvidenceRecord>, CompositionDiagnostic> {
    let request = tool_request(entry)?;
    let mut stack = crate::RuntimeMiddlewareStack::empty();
    let is_dispatch_accepted = matches!(
        stack.prepare_tool_dispatch(request.clone()),
        crate::hooks::chain::MiddlewareOutcome::Accepted(_)
    );
    let blocked_turn = crate::complete_with_guard_block(
        turn_state(entry)?,
        crate::AdapterRequest::Tool {
            request: request.clone(),
        },
        String::from(MIDDLEWARE_HOOK),
        String::from(MIDDLEWARE_REASON),
    );
    let failure = crate::MiddlewareError::new(
        crate::HookBoundary::ToolDispatch,
        MIDDLEWARE_HOOK,
        MIDDLEWARE_FAILURE,
    );
    let failed_turn = crate::complete_with_guard_failure(
        turn_state(entry)?,
        crate::AdapterRequest::Tool { request },
        &failure,
    );
    let is_block_failed_closed = failed_middleware_report(&blocked_turn, MIDDLEWARE_REASON)
        && blocked_turn
            .events()
            .iter()
            .any(|event| matches!(event, crate::RuntimeEvent::HookBlocked { .. }));
    let is_failure_failed_closed = failed_middleware_report(&failed_turn, MIDDLEWARE_FAILURE)
        && failed_turn
            .events()
            .iter()
            .any(|event| matches!(event, crate::RuntimeEvent::HookFailed { .. }));
    if !is_dispatch_accepted || !is_block_failed_closed || !is_failure_failed_closed {
        return Err(scenario_failure(
            entry,
            "middleware acceptance or failure report did not compose",
        ));
    }
    Ok(vec![
        record(
            entry,
            CompositionCaseKind::Positive,
            "middleware_dispatch_accepted",
        ),
        record(
            entry,
            CompositionCaseKind::Negative,
            "middleware_denial_failed_report",
        ),
    ])
}

fn capability_no_fallback(
    entry: &CompositionScenario,
) -> Result<Vec<CompositionEvidenceRecord>, CompositionDiagnostic> {
    let definition = tool_definition(entry)?;
    let ready = dispatch_decision(
        Some(&definition),
        animus_adapters::tool::ExecutorStatus::Available,
        animus_adapters::tool::CapabilityStatus::Allowed,
        animus_adapters::tool::OutputBoundaryStatus::Bounded,
    );
    let missing = dispatch_decision(
        Some(&definition),
        animus_adapters::tool::ExecutorStatus::Missing,
        animus_adapters::tool::CapabilityStatus::Allowed,
        animus_adapters::tool::OutputBoundaryStatus::Bounded,
    );
    let denied = dispatch_decision(
        Some(&definition),
        animus_adapters::tool::ExecutorStatus::Available,
        animus_adapters::tool::CapabilityStatus::Denied,
        animus_adapters::tool::OutputBoundaryStatus::Bounded,
    );
    let unknown_tool = dispatch_decision(
        None,
        animus_adapters::tool::ExecutorStatus::Available,
        animus_adapters::tool::CapabilityStatus::Allowed,
        animus_adapters::tool::OutputBoundaryStatus::Bounded,
    );
    let is_ready_without_fallback = ready.status() == animus_adapters::tool::DispatchStatus::Ready
        && !ready.fallback_attempted();
    let is_missing_failed_closed = failed_without_fallback(
        missing,
        Some(animus_adapters::tool::FailureClass::MissingExecutor),
    );
    let is_denied_failed_closed = failed_without_fallback(
        denied,
        Some(animus_adapters::tool::FailureClass::DeniedCapability),
    );
    let is_unknown_tool_failed_closed = failed_without_fallback(
        unknown_tool,
        Some(animus_adapters::tool::FailureClass::InvalidInput),
    );
    if !all_checks_pass::<CAPABILITY_CHECK_COUNT>([
        is_ready_without_fallback,
        is_missing_failed_closed,
        is_denied_failed_closed,
        is_unknown_tool_failed_closed,
    ]) {
        return Err(scenario_failure(
            entry,
            "capability dispatch attempted fallback or missed denial",
        ));
    }
    Ok(vec![
        record(
            entry,
            CompositionCaseKind::Positive,
            "capability_dispatch_ready",
        ),
        record(
            entry,
            CompositionCaseKind::Negative,
            "missing_service_no_fallback",
        ),
        record(
            entry,
            CompositionCaseKind::Negative,
            "ambient_fallback_not_attempted",
        ),
    ])
}

fn text_agent_model() -> crate::ProviderModelMetadata {
    model(crate::ProviderModelClass::TextAgent)
}

fn image_model() -> crate::ProviderModelMetadata {
    model(crate::ProviderModelClass::Image)
}

fn model(model_class: crate::ProviderModelClass) -> crate::ProviderModelMetadata {
    crate::ProviderModelMetadata {
        provider_id: String::from(SAFE_PROVIDER_ID),
        model_id: String::from(SAFE_MODEL_ID),
        display_name: String::from(SAFE_MODEL_DISPLAY),
        model_class,
        context_budget_label: String::from(SAFE_CONTEXT_BUDGET),
        reasoning: true,
    }
}

fn tool_definition(
    entry: &CompositionScenario,
) -> Result<animus_adapters::tool::Definition, CompositionDiagnostic> {
    animus_adapters::tool::Definition::validate(&animus_adapters::tool::DefinitionDraft {
        name: Some(TOOL_NAME),
        label: Some(TOOL_LABEL),
        description: Some(TOOL_DESCRIPTION),
        parameter_schema_identity: Some(TOOL_SCHEMA_ID),
        prompt_snippet: Some(TOOL_PROMPT_SNIPPET),
        prompt_guidelines: &[TOOL_GUIDELINE],
        source_identity: Some(TOOL_SOURCE_ID),
        provenance: Some(TOOL_PROVENANCE),
        capability_labels: &[TOOL_CAPABILITY],
        result_metadata_shape: Some(TOOL_RESULT_SHAPE),
        status: animus_adapters::tool::DefinitionStatus::Active,
    })
    .map_err(|error| scenario_failure(entry, format!("tool definition: {error:?}")))
}

fn tool_request(
    entry: &CompositionScenario,
) -> Result<animus_adapters::ToolRequest, CompositionDiagnostic> {
    let fields = [animus_adapters::tool::ArgumentFieldDraft {
        name: Some(TOOL_ARGUMENT_NAME),
        value: Some(TOOL_INPUT_SUMMARY),
    }];
    let arguments = animus_adapters::tool::PreparedArguments::prepare(
        &animus_adapters::tool::ArgumentPreparationRequest {
            current_schema_identity: TOOL_SCHEMA_ID,
            allowed_field_names: &[TOOL_ARGUMENT_NAME],
            compatibility_rules: &[],
            fields: &fields,
        },
    )
    .map_err(|error| scenario_failure(entry, format!("tool arguments: {error:?}")))?;
    animus_adapters::ToolRequest::from_draft(&animus_adapters::tool::RequestDraft {
        name: Some(TOOL_NAME),
        arguments: Some(&arguments),
        correlation_id: Some(TOOL_CALL_ID),
        output_policy: animus_adapters::tool::OutputPolicy {
            visible_char_budget: SMALL_OUTPUT_BUDGET_CHARS,
        },
        mutation_targets: &[TOOL_MUTATION_TARGET],
        required_capability_labels: &[TOOL_CAPABILITY],
        approval_posture: animus_adapters::tool::ApprovalPosture::PreApproved.as_str(),
    })
    .map_err(|error| scenario_failure(entry, format!("tool request: {error:?}")))
}

fn tool_executor_kit_receipt(
    entry: &CompositionScenario,
    definition: animus_adapters::tool::Definition,
    request: &animus_adapters::ToolRequest,
) -> Result<crate::tool_executor_kit::ToolExecutionReceipt, CompositionDiagnostic> {
    let catalog = tool_executor_kit_catalog(entry, definition)?;
    let registry = tool_executor_kit_registry(entry)?;
    let executor_binding = crate::tool_executor_kit::ToolExecutorBinding::new(
        TOOL_EXECUTOR_BINDING,
        crate::tool_executor_kit::ToolExecutorReadiness::Ready,
    )
    .map_err(|error| scenario_failure(entry, format!("executor binding: {error:?}")))?;
    let plan = crate::tool_executor_kit::plan_tool_execution(
        crate::tool_executor_kit::ToolExecutionPlanRequest {
            request,
            catalog: &catalog,
            capabilities: &registry,
            executors: &[executor_binding],
            admission: crate::tool_executor_kit::ToolExecutorAdmission::admitted(),
        },
    );
    let mut executor = FakeKitExecutor::new();
    let outcome = crate::tool_executor_kit::execute_admitted_tool(&mut executor, &plan, request)
        .map_err(|error| scenario_failure(entry, format!("kit execution: {error:?}")))?;
    if executor.calls != EXPECTED_FAKE_CALL_COUNT {
        return Err(scenario_failure(
            entry,
            "tool executor kit fake service was not called exactly once",
        ));
    }
    Ok(outcome.receipt().clone())
}

fn tool_executor_kit_missing_fails_closed(
    entry: &CompositionScenario,
    definition: &animus_adapters::tool::Definition,
) -> Result<bool, CompositionDiagnostic> {
    let request = tool_request(entry)?;
    let catalog = tool_executor_kit_catalog(entry, definition.clone())?;
    let registry = tool_executor_kit_registry(entry)?;
    let plan = crate::tool_executor_kit::plan_tool_execution(
        crate::tool_executor_kit::ToolExecutionPlanRequest {
            request: &request,
            catalog: &catalog,
            capabilities: &registry,
            executors: &[],
            admission: crate::tool_executor_kit::ToolExecutorAdmission::admitted(),
        },
    );
    let receipt = crate::tool_executor_kit::denied_tool_receipt(&plan)
        .map_err(|error| scenario_failure(entry, format!("kit denial receipt: {error:?}")))?;
    Ok(plan.failure_class()
        == Some(crate::tool_executor_kit::ToolExecutionFailureClass::MissingExecutor)
        && receipt.status() == crate::tool_executor_kit::ToolExecutionReceiptStatus::Denied)
}

fn tool_executor_kit_catalog(
    entry: &CompositionScenario,
    definition: animus_adapters::tool::Definition,
) -> Result<crate::tool_executor_kit::ToolCatalog, CompositionDiagnostic> {
    let output_policy = crate::tool_executor_kit::ToolExecutorOutputPolicy::new(
        SMALL_OUTPUT_BUDGET_CHARS,
        TOOL_EXECUTOR_MAX_OUTPUT_BYTES,
        false,
    )
    .map_err(|error| scenario_failure(entry, format!("kit output policy: {error:?}")))?;
    let definition = crate::tool_executor_kit::ToolDefinition::from_draft(
        crate::tool_executor_kit::ToolDefinitionDraft {
            metadata: definition,
            executor_binding: TOOL_EXECUTOR_BINDING,
            argument_names: &[TOOL_ARGUMENT_NAME],
            mutation_posture: crate::tool_executor_kit::ToolMutationPosture::DeclaredTargets,
            output_policy,
            approval_posture: animus_adapters::tool::ApprovalPosture::PreApproved,
        },
    )
    .map_err(|error| scenario_failure(entry, format!("kit definition: {error:?}")))?;
    crate::tool_executor_kit::ToolCatalog::new(vec![definition])
        .map_err(|error| scenario_failure(entry, format!("kit catalog: {error:?}")))
}

fn tool_executor_kit_registry(
    entry: &CompositionScenario,
) -> Result<crate::RuntimeCapabilityRegistry, CompositionDiagnostic> {
    crate::RuntimeCapabilityRegistry::from_drafts(&[crate::RuntimeCapabilityDescriptorDraft {
        label: TOOL_CAPABILITY,
        service_kind: crate::RuntimeCapabilityServiceKind::Tool.as_str(),
        scope: None,
    }])
    .map_err(|error| scenario_failure(entry, format!("kit capability registry: {error:?}")))
}

fn tool_response(
    body: &str,
) -> Result<animus_adapters::ToolResponse, animus_adapters::tool::ContractError> {
    let bounded = bounded_tool_output(body, animus_adapters::tool::HashRequirement::Required)?;
    Ok(animus_adapters::ToolResponse::from_bounded(
        bounded,
        animus_adapters::tool::OutputRedactionClass::Safe,
    ))
}

fn bounded_tool_output(
    body: &str,
    hash_requirement: animus_adapters::tool::HashRequirement,
) -> Result<animus_adapters::tool::BoundedOutput, animus_adapters::tool::ContractError> {
    animus_adapters::tool::BoundedOutput::new(
        &animus_adapters::tool::OutputDraft {
            body,
            full_output_reference: Some("target/cairn/focused-rails/composition-conformance.md"),
            hash_requirement,
        },
        animus_adapters::tool::OutputPolicy {
            visible_char_budget: SMALL_OUTPUT_BUDGET_CHARS,
        },
    )
}

fn completed_tool_effect(
    entry: &CompositionScenario,
    original_output_bytes: Option<u64>,
    retained_output_bytes: Option<u64>,
) -> Result<crate::ProvenanceToolEffect, CompositionDiagnostic> {
    crate::ProvenanceToolEffect::validate(&crate::ProvenanceToolEffectDraft {
        tool_kind: TOOL_NAME,
        service_label: TOOL_SERVICE_LABEL,
        effect_ability: TOOL_EFFECT_ABILITY,
        effect_resource: TOOL_EFFECT_RESOURCE,
        status: crate::ProvenanceToolEffectStatus::Completed,
        original_output_bytes,
        retained_output_bytes,
    })
    .map_err(|error| scenario_failure(entry, format!("tool effect: {error:?}")))
}

fn dispatch_decision(
    definition: Option<&animus_adapters::tool::Definition>,
    executor_status: animus_adapters::tool::ExecutorStatus,
    capability_status: animus_adapters::tool::CapabilityStatus,
    output_boundary_status: animus_adapters::tool::OutputBoundaryStatus,
) -> animus_adapters::tool::DispatchDecision {
    animus_adapters::tool::evaluate_dispatch(animus_adapters::tool::DispatchRequest {
        definition,
        executor_status,
        capability_status,
        output_boundary_status,
    })
}

fn all_checks_pass<const CHECK_COUNT: usize>(checks: [bool; CHECK_COUNT]) -> bool {
    checks.into_iter().all(|is_check_passed| is_check_passed)
}

fn failed_without_fallback(
    decision: animus_adapters::tool::DispatchDecision,
    failure_class: Option<animus_adapters::tool::FailureClass>,
) -> bool {
    decision.status() == animus_adapters::tool::DispatchStatus::Failed
        && decision.failure_class() == failure_class
        && !decision.fallback_attempted()
}

fn turn_state(entry: &CompositionScenario) -> Result<crate::TurnState, CompositionDiagnostic> {
    let input = crate::RuntimeInput::new(entry.id.as_str())
        .map_err(|error| scenario_failure(entry, format!("runtime input: {error:?}")))?;
    Ok(crate::TurnState::new(&input))
}

fn failed_middleware_report(report: &crate::RunReport, message: &str) -> bool {
    matches!(
        report.status(),
        crate::RuntimeStatus::Failed {
            failure: crate::RuntimeFailure::Middleware { message: failure_message, .. }
        } if failure_message == message
    )
}

fn report_from_records(
    scenario_count: usize,
    records: Vec<CompositionEvidenceRecord>,
) -> CompositionRailReport {
    let positive_count = records
        .iter()
        .filter(|record| record.case_kind == CompositionCaseKind::Positive)
        .count();
    let negative_count = records
        .iter()
        .filter(|record| record.case_kind == CompositionCaseKind::Negative)
        .count();
    let record_hashes: Vec<&str> = records
        .iter()
        .map(|record| record.evidence_hash.as_str())
        .collect();
    let scenario_count_text = scenario_count.to_string();
    let positive_count_text = positive_count.to_string();
    let negative_count_text = negative_count.to_string();
    let rail_fingerprint = fingerprint(
        "report",
        &[
            scenario_count_text.as_str(),
            positive_count_text.as_str(),
            negative_count_text.as_str(),
            &record_hashes.join(HASH_SEPARATOR),
        ],
    );
    CompositionRailReport {
        scenario_count,
        positive_count,
        negative_count,
        records,
        report_hash: rail_fingerprint,
    }
}

fn record(
    entry: &CompositionScenario,
    case_kind: CompositionCaseKind,
    outcome: &str,
) -> CompositionEvidenceRecord {
    let evidence_hash = fingerprint(
        "record",
        &[
            entry.id.as_str(),
            entry.surface.as_str(),
            case_kind.as_str(),
            outcome,
            entry.safe_evidence_label.as_str(),
        ],
    );
    CompositionEvidenceRecord {
        scenario_id: entry.id.clone(),
        surface: entry.surface,
        case_kind,
        outcome: String::from(outcome),
        evidence_label: entry.safe_evidence_label.clone(),
        evidence_hash,
    }
}

fn fingerprint(label: &str, parts: &[&str]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(HASH_DOMAIN.as_bytes());
    hasher.update(HASH_SEPARATOR.as_bytes());
    hasher.update(label.as_bytes());
    for part in parts {
        hasher.update(HASH_SEPARATOR.as_bytes());
        hasher.update(part.as_bytes());
    }
    format!("{HASH_ALGORITHM}:{}", hasher.finalize().to_hex())
}

fn scenario_failure(
    entry: &CompositionScenario,
    detail: impl Into<String>,
) -> CompositionDiagnostic {
    CompositionDiagnostic::new(
        CompositionDiagnosticKind::ScenarioFailed,
        entry.id.clone(),
        detail,
    )
}

struct FakeModelAdapter {
    response_message: &'static str,
    calls: usize,
}

impl FakeModelAdapter {
    fn new(response_message: &'static str) -> Self {
        Self {
            response_message,
            calls: 0,
        }
    }
}

impl
    animus_adapters::ComponentAdapter<animus_adapters::ModelRequest, animus_adapters::ModelResponse>
    for FakeModelAdapter
{
    type Error = String;

    fn call(
        &mut self,
        request: &animus_adapters::ModelRequest,
    ) -> Result<animus_adapters::ModelResponse, Self::Error> {
        if request.prompt().trim().is_empty() {
            return Err(String::from("empty_prompt"));
        }
        self.calls += 1;
        animus_adapters::ModelResponse::new(self.response_message)
            .map_err(|error| format!("{error:?}"))
    }
}

struct FakeKitExecutor {
    calls: usize,
}

impl FakeKitExecutor {
    const fn new() -> Self {
        Self { calls: 0 }
    }
}

impl crate::tool_executor_kit::ToolExecutorService for FakeKitExecutor {
    fn binding_label(&self) -> &str {
        TOOL_EXECUTOR_BINDING
    }

    fn execute(
        &mut self,
        _plan: &crate::tool_executor_kit::ToolExecutionPlan,
        _request: &animus_adapters::ToolRequest,
    ) -> Result<
        crate::tool_executor_kit::ToolExecutorOutput,
        crate::tool_executor_kit::ToolExecutorServiceFailure,
    > {
        self.calls += EXPECTED_FAKE_CALL_COUNT;
        Ok(crate::tool_executor_kit::ToolExecutorOutput::new(
            TOOL_OUTPUT_SUMMARY,
            None,
            animus_adapters::tool::OutputRedactionClass::Safe,
        ))
    }
}

struct FakeToolAdapter {
    output: &'static str,
    calls: usize,
}

impl FakeToolAdapter {
    fn new(output: &'static str) -> Self {
        Self { output, calls: 0 }
    }
}

impl animus_adapters::ComponentAdapter<animus_adapters::ToolRequest, animus_adapters::ToolResponse>
    for FakeToolAdapter
{
    type Error = String;

    fn call(
        &mut self,
        request: &animus_adapters::ToolRequest,
    ) -> Result<animus_adapters::ToolResponse, Self::Error> {
        if request.name() != TOOL_NAME {
            return Err(String::from("unknown_tool"));
        }
        self.calls += 1;
        tool_response(self.output).map_err(|error| format!("{error:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIRST_SCENARIO_INDEX: usize = 0;
    const EXPECTED_SCENARIO_COUNT: usize = 5;
    const EXPECTED_POSITIVE_RECORDS: usize = 5;
    const EXPECTED_NEGATIVE_RECORDS: usize = 9;

    #[test]
    fn conformance_matrix_covers_required_positive_and_negative_cases() {
        // r[verify agent_harness_runtime.composability.conformance_matrix.coverage]
        let matrix = composition_conformance_matrix();

        validate_composition_matrix(&matrix).expect("matrix should be complete");
        assert_eq!(matrix.len(), EXPECTED_SCENARIO_COUNT);
        assert!(matrix.iter().all(|entry| !entry.owner.is_empty()));
        assert!(
            matrix
                .iter()
                .all(|entry| !entry.feature_requirements.is_empty())
        );
        assert!(matrix.iter().all(|entry| !entry.positive_case.is_empty()));
        assert!(matrix.iter().all(|entry| !entry.negative_case.is_empty()));
        assert!(
            matrix
                .iter()
                .all(|entry| !entry.safe_evidence_label.is_empty())
        );
    }

    #[test]
    fn incomplete_matrix_entries_fail_validation() {
        // r[verify agent_harness_runtime.composability.conformance_matrix.coverage]
        let mut matrix = composition_conformance_matrix();
        matrix[FIRST_SCENARIO_INDEX].owner.clear();
        matrix[FIRST_SCENARIO_INDEX].feature_requirements.clear();
        matrix[FIRST_SCENARIO_INDEX].positive_case.clear();
        matrix[FIRST_SCENARIO_INDEX].negative_case.clear();
        matrix[FIRST_SCENARIO_INDEX].safe_evidence_label.clear();

        let diagnostics = validate_composition_matrix(&matrix).expect_err("matrix should fail");

        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.kind == CompositionDiagnosticKind::MissingOwner)
        );
        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic.kind
                == CompositionDiagnosticKind::MissingFeatureRequirements)
        );
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.kind
            == CompositionDiagnosticKind::MissingPositiveCase));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.kind
            == CompositionDiagnosticKind::MissingNegativeCase));
        assert!(diagnostics.iter().any(
            |diagnostic| diagnostic.kind == CompositionDiagnosticKind::MissingSafeEvidenceLabel
        ));
    }

    #[test]
    fn fake_backed_positive_composition_scenarios_pass_without_live_services() {
        // r[verify agent_harness_runtime.composability.positive_conformance.fake_backed]
        let rail_snapshot = run_composition_conformance().expect("rail should pass");

        assert_eq!(rail_snapshot.scenario_count, EXPECTED_SCENARIO_COUNT);
        assert_eq!(rail_snapshot.positive_count, EXPECTED_POSITIVE_RECORDS);
        assert!(
            rail_snapshot
                .records
                .iter()
                .any(|record| record.outcome == "model_dispatch_completed")
        );
        assert!(
            rail_snapshot
                .records
                .iter()
                .any(|record| record.outcome == "tool_receipt_completed")
        );
        assert!(
            rail_snapshot
                .records
                .iter()
                .any(|record| record.outcome == "headless_projection_completed")
        );
        assert!(
            rail_snapshot
                .records
                .iter()
                .any(|record| record.outcome == "middleware_dispatch_accepted")
        );
        assert!(
            rail_snapshot
                .records
                .iter()
                .any(|record| record.outcome == "capability_dispatch_ready")
        );
    }

    #[test]
    fn negative_composition_scenarios_fail_closed() {
        // r[verify agent_harness_runtime.composability.negative_conformance.fail_closed]
        let rail_snapshot = run_composition_conformance().expect("rail should pass");

        assert_eq!(rail_snapshot.negative_count, EXPECTED_NEGATIVE_RECORDS);
        for outcome in [
            "malformed_response_rejected",
            "disabled_feature_surface_rejected",
            "unauthorized_tool_denied",
            "oversized_output_rejected",
            "unsafe_receipt_rejected",
            "unsafe_headless_record_redacted",
            "middleware_denial_failed_report",
            "missing_service_no_fallback",
            "ambient_fallback_not_attempted",
        ] {
            assert!(
                rail_snapshot
                    .records
                    .iter()
                    .any(|record| record.outcome == outcome
                        && record.case_kind == CompositionCaseKind::Negative),
                "missing negative outcome {outcome}"
            );
        }
    }

    #[test]
    fn conformance_evidence_renders_safe_counts_and_hashes() {
        // r[verify agent_harness_runtime.composability.conformance_rail.safe_evidence]
        let rail_snapshot = run_composition_conformance().expect("rail should pass");
        let rendered = render_composition_evidence(&rail_snapshot);

        validate_composition_evidence_text(&rendered).expect("evidence should be safe");
        assert!(rendered.contains("scenario_count: 5"));
        assert!(rendered.contains("positive_count: 5"));
        assert!(rendered.contains("negative_count: 9"));
        assert!(rendered.contains(HASH_ALGORITHM));
        assert!(!rendered.contains(SAFE_PROMPT_SUMMARY));
        assert!(!rendered.contains(TOOL_INPUT_SUMMARY));
    }

    #[test]
    fn conformance_evidence_rejects_leak_markers() {
        // r[verify agent_harness_runtime.composability.conformance_rail.safe_evidence]
        for marker in UNSAFE_EVIDENCE_MARKERS {
            let evidence = format!("schema: {RAIL_SCHEMA}\nrail: {RAIL_NAME}\n{marker}\n");
            let diagnostics =
                validate_composition_evidence_text(&evidence).expect_err("leak marker should fail");
            assert_eq!(
                diagnostics[0].kind,
                CompositionDiagnosticKind::UnsafeEvidence
            );
        }
    }
}
