//! Optional model-evidence adapter contracts.
//!
//! Hosts may collect verifier, conversion, kernel-proof, or benchmark reports
//! before constructing these DTOs. Reusable crates validate only already-loaded
//! report metadata, BLAKE3 identities, freshness envelopes, diagnostics,
//! capability labels, advisory boundaries, and non-claims.

#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::ambiguous_params,
        tigerstyle::assertion_density,
        tigerstyle::bool_naming,
        tigerstyle::borrowed_argument_types,
        tigerstyle::function_length,
        tigerstyle::non_trait_imports,
        tigerstyle::numeric_units,
        tigerstyle::path_segment_repetition
    )
)]

use alloc::string::String;
use alloc::vec::Vec;

use crate::validation::AdapterContractError;
use crate::validation::FieldName;
use crate::validation::{self};

const REPORT_ID_FIELD: FieldName = FieldName::new("model_evidence_report_id");
const EVIDENCE_KIND_FIELD: FieldName = FieldName::new("model_evidence_kind");
const MODEL_LABEL_FIELD: FieldName = FieldName::new("model_evidence_model_label");
const PROPERTY_SCOPE_FIELD: FieldName = FieldName::new("model_evidence_property_scope");
const VERIFIER_KIND_FIELD: FieldName = FieldName::new("model_evidence_verifier_kind");
const VERIFIER_IDENTITY_FIELD: FieldName = FieldName::new("model_evidence_verifier_identity");
const DIAGNOSTIC_FIELD: FieldName = FieldName::new("model_evidence_diagnostic");
const ADMISSION_LABEL_FIELD: FieldName = FieldName::new("model_evidence_admission_label");
const RECEIPT_BACKEND_LABEL: &str = "model-evidence";
const RECEIPT_INTERCHANGE_LABEL: &str = "safe-dto";
const FIELD_DIAGNOSTIC_COUNT: &str = "diagnostic_count";
const FIELD_CAPABILITY_COUNT: &str = "capability_count";
const FIELD_NON_CLAIM_COUNT: &str = "non_claim_count";
const FIELD_SUPPORTING_EVIDENCE_COUNT: &str = "supporting_evidence_count";
const FIELD_RESULT_COUNT: &str = "result_count";
const RAW_MODEL_MARKER: &str = "raw model";
const RAW_PROVIDER_MARKER: &str = "provider payload";
const RAW_PROMPT_MARKER: &str = "prompt:";
const TOKEN_MARKER: &str = "token=";
const SECRET_MARKER: &str = "secret";
const HOME_PATH_MARKER: &str = "/home/";
const TMP_PATH_MARKER: &str = "/tmp/";
const WINDOWS_USER_MARKER: &str = "\\users\\";

const ZERO_COUNT: u32 = 0;
const ZERO_ITEMS: usize = 0;
const MIN_SUPPORTING_EVIDENCE_COUNT: u32 = 1;
const MAX_DIAGNOSTICS: usize = 16;
const MAX_CAPABILITY_LABELS: usize = 8;
const REQUIRED_MODEL_NON_CLAIMS: usize = 8;
const MAX_DIAGNOSTIC_CHARS: usize = 256;
const RESULT_COUNT: u32 = 1;

const UNSAFE_TEXT_MARKERS: &[&str] = &[
    RAW_MODEL_MARKER,
    RAW_PROVIDER_MARKER,
    RAW_PROMPT_MARKER,
    TOKEN_MARKER,
    SECRET_MARKER,
    HOME_PATH_MARKER,
    TMP_PATH_MARKER,
    WINDOWS_USER_MARKER,
];

const _: () = assert!(MIN_SUPPORTING_EVIDENCE_COUNT > ZERO_COUNT);
const _: () = assert!(MAX_DIAGNOSTICS > ZERO_ITEMS);
const _: () = assert!(MAX_CAPABILITY_LABELS > ZERO_ITEMS);
const _: () = assert!(REQUIRED_MODEL_NON_CLAIMS > ZERO_ITEMS);
const _: () = assert!(MAX_DIAGNOSTIC_CHARS >= validation::MIN_ADAPTER_FIELD_CHARS);
const _: () = assert!(RESULT_COUNT > ZERO_COUNT);

/// Product behaviors that model-evidence DTO validation explicitly does not perform.
pub const PROHIBITED_MODEL_EVIDENCE_EFFECTS: &[&str] = &[
    "model execution",
    "GPU access",
    "filesystem reads",
    "verifier invocation",
    "process spawning",
    "environment inspection",
    "clock access",
    "network access",
    "report persistence",
];

/// Supported model-evidence report kind.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ModelEvidenceKind {
    /// Bound-propagation or robustness bound report.
    BoundReport,
    /// Model conversion or export report.
    ConversionReport,
    /// Kernel-proof or checked-kernel summary.
    KernelProofSummary,
    /// Benchmark row admitted only when supporting evidence is also present.
    BenchmarkSummary,
}

impl ModelEvidenceKind {
    /// Parses a supported model-evidence kind label.
    ///
    /// # Errors
    ///
    /// Returns [`ModelEvidenceValidationIssue`] when the label is empty, unsafe,
    /// or unsupported.
    pub fn parse(value: &str) -> Result<Self, ModelEvidenceValidationIssue> {
        let normalized = normalize_label(EVIDENCE_KIND_FIELD, value)?;
        match normalized.as_str() {
            "bound-report" | "verification-bound" => Ok(Self::BoundReport),
            "conversion-report" => Ok(Self::ConversionReport),
            "kernel-proof-summary" => Ok(Self::KernelProofSummary),
            "benchmark-summary" => Ok(Self::BenchmarkSummary),
            _unsupported => Err(ModelEvidenceValidationIssue::UnsupportedEvidenceKind),
        }
    }

    /// Returns the stable kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BoundReport => "bound-report",
            Self::ConversionReport => "conversion-report",
            Self::KernelProofSummary => "kernel-proof-summary",
            Self::BenchmarkSummary => "benchmark-summary",
        }
    }
}

/// Supported verifier or tool family for model-evidence reports.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ModelEvidenceVerifierKind {
    /// Gamma-CROWN-like bound verifier.
    GammaCrown,
    /// mly-style conversion checker.
    MlyConversion,
    /// Kani or kernel-proof checker.
    KernelProof,
    /// Benchmark harness for advisory rows.
    BenchmarkHarness,
}

impl ModelEvidenceVerifierKind {
    /// Parses a supported verifier kind label.
    ///
    /// # Errors
    ///
    /// Returns [`ModelEvidenceValidationIssue`] when the label is empty, unsafe,
    /// or unsupported.
    pub fn parse(value: &str) -> Result<Self, ModelEvidenceValidationIssue> {
        let normalized = normalize_label(VERIFIER_KIND_FIELD, value)?;
        match normalized.as_str() {
            "gamma-crown" => Ok(Self::GammaCrown),
            "mly-conversion" => Ok(Self::MlyConversion),
            "kernel-proof" | "kani-kernel-proof" => Ok(Self::KernelProof),
            "benchmark-harness" => Ok(Self::BenchmarkHarness),
            _unsupported => Err(ModelEvidenceValidationIssue::UnsupportedVerifier),
        }
    }

    /// Returns the stable verifier kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GammaCrown => "gamma-crown",
            Self::MlyConversion => "mly-conversion",
            Self::KernelProof => "kernel-proof",
            Self::BenchmarkHarness => "benchmark-harness",
        }
    }
}

/// Result class reported by a model-evidence artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelEvidenceResultStatus {
    /// Evidence passed for its declared bounded scope.
    Passed,
    /// Evidence failed for its declared bounded scope.
    Failed,
    /// Evidence is inconclusive and remains advisory.
    Inconclusive,
}

impl ModelEvidenceResultStatus {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Inconclusive => "inconclusive",
        }
    }
}

/// Freshness envelope supplied by the host shell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelEvidenceFreshness {
    /// Report is current for the supplied model and corpus identities.
    Current,
    /// Report is stale and must fail closed.
    Stale,
}

impl ModelEvidenceFreshness {
    /// Returns the stable freshness label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Stale => "stale",
        }
    }
}

/// Capability or policy label attached to model evidence.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ModelEvidenceCapabilityLabel {
    /// Evidence may advise routing policy.
    AdvisoryRouting,
    /// Evidence may advise local admission policy.
    AdmissionHint,
    /// Evidence attempts to grant runtime authority and is rejected.
    RuntimeAuthority,
    /// Evidence attempts to claim production model safety and is rejected.
    ProductionSafety,
}

impl ModelEvidenceCapabilityLabel {
    /// Returns the stable capability label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AdvisoryRouting => "advisory_routing",
            Self::AdmissionHint => "admission_hint",
            Self::RuntimeAuthority => "runtime_authority",
            Self::ProductionSafety => "production_safety",
        }
    }

    const fn is_overclaim(self) -> bool {
        matches!(self, Self::RuntimeAuthority | Self::ProductionSafety)
    }
}

/// Explicit non-claims that must accompany accepted model evidence.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ModelEvidenceNonClaim {
    /// No arbitrary robustness proof is implied.
    ArbitraryRobustness,
    /// Model output correctness is not established.
    OutputCorrectness,
    /// Hallucination prevention is not established.
    HallucinationPrevention,
    /// Verifier soundness is not established by the report.
    VerifierSoundness,
    /// Benchmark superiority is not established.
    BenchmarkSuperiority,
    /// Release eligibility is not established.
    ReleaseEligibility,
    /// GPU correctness is not established.
    GpuCorrectness,
    /// Whole-system safety is not established.
    WholeSystemSafety,
}

impl ModelEvidenceNonClaim {
    /// Returns the stable non-claim label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ArbitraryRobustness => "no_arbitrary_robustness_claim",
            Self::OutputCorrectness => "no_output_correctness_claim",
            Self::HallucinationPrevention => "no_hallucination_prevention_claim",
            Self::VerifierSoundness => "no_verifier_soundness_claim",
            Self::BenchmarkSuperiority => "no_benchmark_superiority_claim",
            Self::ReleaseEligibility => "no_release_eligibility_claim",
            Self::GpuCorrectness => "no_gpu_correctness_claim",
            Self::WholeSystemSafety => "no_whole_system_safety_claim",
        }
    }
}

/// Deterministic model-evidence validation issue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelEvidenceValidationIssue {
    /// Required text field was empty, too large, or contained a NUL byte.
    InvalidField,
    /// Metadata looked like raw content, credentials, paths, prompts, or provider payloads.
    UnsafeMetadata,
    /// Evidence kind was unsupported.
    UnsupportedEvidenceKind,
    /// Verifier kind was unsupported.
    UnsupportedVerifier,
    /// Model hash was malformed or mismatched.
    MismatchedModel,
    /// Input corpus hash was malformed.
    InvalidCorpusHash,
    /// Freshness envelope was stale.
    StaleFreshness,
    /// Property scope was missing.
    MissingPropertyScope,
    /// Diagnostics were missing or unsafe.
    MissingDiagnostics,
    /// Benchmark evidence had no supporting verification or conversion evidence.
    BenchmarkOnly,
    /// Capability label attempted to overclaim authority or safety.
    CapabilityOverclaim,
    /// Required non-claims were absent.
    MissingNonClaim,
    /// Collection exceeded the public DTO bound.
    TooManyItems,
    /// Safe receipt construction rejected metadata.
    SafeReceiptRejected,
}

impl ModelEvidenceValidationIssue {
    /// Returns the stable issue label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidField => "invalid_field",
            Self::UnsafeMetadata => "unsafe_metadata",
            Self::UnsupportedEvidenceKind => "unsupported_evidence_kind",
            Self::UnsupportedVerifier => "unsupported_verifier",
            Self::MismatchedModel => "mismatched_model",
            Self::InvalidCorpusHash => "invalid_corpus_hash",
            Self::StaleFreshness => "stale_freshness",
            Self::MissingPropertyScope => "missing_property_scope",
            Self::MissingDiagnostics => "missing_diagnostics",
            Self::BenchmarkOnly => "benchmark_only",
            Self::CapabilityOverclaim => "capability_overclaim",
            Self::MissingNonClaim => "missing_non_claim",
            Self::TooManyItems => "too_many_items",
            Self::SafeReceiptRejected => "safe_receipt_rejected",
        }
    }
}

/// Safe diagnostic emitted by model-evidence validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelEvidenceDiagnostic {
    issue: ModelEvidenceValidationIssue,
    field: &'static str,
    message: String,
}

impl ModelEvidenceDiagnostic {
    /// Builds a safe model-evidence diagnostic.
    #[must_use]
    pub fn new(issue: ModelEvidenceValidationIssue, field: &'static str, message: &str) -> Self {
        Self {
            issue,
            field,
            message: String::from(message),
        }
    }

    /// Returns the issue code.
    #[must_use]
    pub const fn issue(&self) -> ModelEvidenceValidationIssue {
        self.issue
    }

    /// Returns the field label.
    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }

    /// Returns the safe diagnostic message.
    #[must_use]
    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}

/// Borrowed model artifact identity fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelArtifactIdentityDraft<'a> {
    /// Safe model label.
    pub model_label: &'a str,
    /// Expected model artifact hash.
    pub model_hash: &'a str,
    /// Observed model artifact hash at evidence collection time.
    pub observed_model_hash: &'a str,
}

/// Validated model artifact identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelArtifactIdentity {
    model_label: String,
    model_hash: crate::ArtifactHash,
}

impl ModelArtifactIdentity {
    /// Validates model identity and freshness.
    ///
    /// # Errors
    ///
    /// Returns diagnostics when the label is unsafe, either hash is malformed,
    /// or the observed hash differs from the expected model hash.
    pub fn new(draft: ModelArtifactIdentityDraft<'_>) -> Result<Self, ModelEvidenceError> {
        let mut diagnostics = Vec::new();
        let model_label = validate_safe_label(MODEL_LABEL_FIELD, draft.model_label, &mut diagnostics);
        let model_hash = validate_hash(
            "model_evidence_model_hash",
            draft.model_hash,
            ModelEvidenceValidationIssue::MismatchedModel,
            &mut diagnostics,
        );
        let observed_hash = validate_hash(
            "model_evidence_observed_model_hash",
            draft.observed_model_hash,
            ModelEvidenceValidationIssue::MismatchedModel,
            &mut diagnostics,
        );
        if model_hash
            .as_ref()
            .zip(observed_hash.as_ref())
            .is_some_and(|(expected, observed)| expected != observed)
        {
            diagnostics.push(ModelEvidenceDiagnostic::new(
                ModelEvidenceValidationIssue::MismatchedModel,
                "model_evidence_model_hash",
                "model artifact hash does not match observed evidence hash",
            ));
        }
        if diagnostics.is_empty() {
            if let (Some(model_label), Some(model_hash)) = (model_label, model_hash) {
                return Ok(Self {
                    model_label,
                    model_hash,
                });
            }
            diagnostics.push(ModelEvidenceDiagnostic::new(
                ModelEvidenceValidationIssue::InvalidField,
                "model_evidence_identity",
                "model evidence identity validation ended without required fields",
            ));
        }
        Err(ModelEvidenceError { diagnostics })
    }

    /// Returns the safe model label.
    #[must_use]
    pub fn model_label(&self) -> &str {
        self.model_label.as_str()
    }

    /// Returns the model artifact hash.
    #[must_use]
    pub const fn model_hash(&self) -> &crate::ArtifactHash {
        &self.model_hash
    }
}

/// Borrowed model-evidence report before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelEvidenceReportDraft<'a> {
    /// Safe report identifier.
    pub report_id: &'a str,
    /// Evidence kind label.
    pub evidence_kind: &'a str,
    /// Model artifact identity.
    pub model: ModelArtifactIdentityDraft<'a>,
    /// Property scope label.
    pub property_scope: &'a str,
    /// Verifier kind label.
    pub verifier_kind: &'a str,
    /// Verifier or tool identity label.
    pub verifier_identity: &'a str,
    /// Input corpus BLAKE3 hash.
    pub input_corpus_hash: &'a str,
    /// Evidence result status.
    pub result_status: ModelEvidenceResultStatus,
    /// Freshness envelope.
    pub freshness: ModelEvidenceFreshness,
    /// Supporting non-benchmark evidence count.
    pub supporting_evidence_count: u32,
    /// Safe diagnostics.
    pub diagnostics: &'a [ModelEvidenceDiagnostic],
    /// Capability labels.
    pub capability_labels: &'a [ModelEvidenceCapabilityLabel],
    /// Explicit non-claims.
    pub non_claims: &'a [ModelEvidenceNonClaim],
}

/// Validated model-evidence report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelEvidenceReport {
    report_id: String,
    evidence_kind: ModelEvidenceKind,
    model: ModelArtifactIdentity,
    property_scope: String,
    verifier_kind: ModelEvidenceVerifierKind,
    verifier_identity: String,
    input_corpus_hash: crate::ArtifactHash,
    result_status: ModelEvidenceResultStatus,
    freshness: ModelEvidenceFreshness,
    supporting_evidence_count: u32,
    diagnostics: Vec<ModelEvidenceDiagnostic>,
    capability_labels: Vec<ModelEvidenceCapabilityLabel>,
    non_claims: Vec<ModelEvidenceNonClaim>,
    receipt: crate::safe::receipt::Record,
}

impl ModelEvidenceReport {
    /// Validates a model-evidence report draft.
    ///
    /// # Errors
    ///
    /// Returns deterministic diagnostics when the report is malformed, stale,
    /// mismatched, unsupported, benchmark-only, overclaiming, or missing non-claims.
    pub fn new(draft: ModelEvidenceReportDraft<'_>) -> Result<Self, ModelEvidenceError> {
        validate_model_evidence_report(draft)
    }

    /// Returns report identifier.
    #[must_use]
    pub fn report_id(&self) -> &str {
        self.report_id.as_str()
    }

    /// Returns evidence kind.
    #[must_use]
    pub const fn evidence_kind(&self) -> ModelEvidenceKind {
        self.evidence_kind
    }

    /// Returns model identity.
    #[must_use]
    pub const fn model(&self) -> &ModelArtifactIdentity {
        &self.model
    }

    /// Returns property scope.
    #[must_use]
    pub fn property_scope(&self) -> &str {
        self.property_scope.as_str()
    }

    /// Returns verifier kind.
    #[must_use]
    pub const fn verifier_kind(&self) -> ModelEvidenceVerifierKind {
        self.verifier_kind
    }

    /// Returns verifier identity.
    #[must_use]
    pub fn verifier_identity(&self) -> &str {
        self.verifier_identity.as_str()
    }

    /// Returns input corpus hash.
    #[must_use]
    pub const fn input_corpus_hash(&self) -> &crate::ArtifactHash {
        &self.input_corpus_hash
    }

    /// Returns result status.
    #[must_use]
    pub const fn result_status(&self) -> ModelEvidenceResultStatus {
        self.result_status
    }

    /// Returns freshness status.
    #[must_use]
    pub const fn freshness(&self) -> ModelEvidenceFreshness {
        self.freshness
    }

    /// Returns diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[ModelEvidenceDiagnostic] {
        self.diagnostics.as_slice()
    }

    /// Returns capability labels.
    #[must_use]
    pub fn capability_labels(&self) -> &[ModelEvidenceCapabilityLabel] {
        self.capability_labels.as_slice()
    }

    /// Returns explicit non-claims.
    #[must_use]
    pub fn non_claims(&self) -> &[ModelEvidenceNonClaim] {
        self.non_claims.as_slice()
    }

    /// Returns safe receipt metadata.
    #[must_use]
    pub const fn receipt(&self) -> &crate::safe::receipt::Record {
        &self.receipt
    }

    /// Builds an advisory routing or admission receipt preserving evidence non-claims.
    ///
    /// # Errors
    ///
    /// Returns diagnostics when the decision label is unsafe or safe receipt
    /// construction rejects metadata.
    pub fn advisory_decision_receipt(
        &self,
        decision_label: &str,
    ) -> Result<crate::safe::receipt::Record, ModelEvidenceError> {
        let mut diagnostics = Vec::new();
        let decision_label = validate_safe_label(ADMISSION_LABEL_FIELD, decision_label, &mut diagnostics);
        if !diagnostics.is_empty() {
            return Err(ModelEvidenceError { diagnostics });
        }
        let Some(decision_label) = decision_label else {
            return Err(ModelEvidenceError {
                diagnostics: alloc::vec![ModelEvidenceDiagnostic::new(
                    ModelEvidenceValidationIssue::InvalidField,
                    ADMISSION_LABEL_FIELD.as_str(),
                    "model evidence decision label validation ended without required field",
                )],
            });
        };
        build_receipt(
            &ReceiptInput {
                report_id: self.report_id.as_str(),
                evidence_kind: self.evidence_kind,
                model: &self.model,
                property_scope: self.property_scope.as_str(),
                verifier_kind: self.verifier_kind,
                verifier_identity: self.verifier_identity.as_str(),
                input_corpus_hash: &self.input_corpus_hash,
                result_status: self.result_status,
                freshness: self.freshness,
                supporting_evidence_count: self.supporting_evidence_count,
                diagnostic_count: len_to_u32(self.diagnostics.len()),
                capability_count: len_to_u32(self.capability_labels.len()),
                non_claim_count: len_to_u32(self.non_claims.len()),
                capabilities: self.capability_labels.as_slice(),
                non_claims: self.non_claims.as_slice(),
                extra_label: Some(decision_label.as_str()),
            },
            crate::safe::receipt::Operation::ProvenanceObservation,
        )
        .map_err(|diagnostic| ModelEvidenceError {
            diagnostics: alloc::vec![diagnostic],
        })
    }
}

/// Model-evidence validation error with deterministic diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelEvidenceError {
    diagnostics: Vec<ModelEvidenceDiagnostic>,
}

impl ModelEvidenceError {
    /// Returns validation diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[ModelEvidenceDiagnostic] {
        self.diagnostics.as_slice()
    }
}

impl core::fmt::Display for ModelEvidenceError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "model evidence validation failed with {} diagnostic(s)", self.diagnostics.len())
    }
}

/// Validates an already-loaded model-evidence report DTO.
///
/// # Errors
///
/// Returns deterministic diagnostics when validation fails. This function is
/// pure over the supplied DTO and performs no model execution, GPU access, file
/// reads, verifier invocation, process spawning, clock access, or network I/O.
pub fn validate_model_evidence_report(
    draft: ModelEvidenceReportDraft<'_>,
) -> Result<ModelEvidenceReport, ModelEvidenceError> {
    let mut diagnostics = Vec::new();
    let report_id = validate_safe_label(REPORT_ID_FIELD, draft.report_id, &mut diagnostics);
    let evidence_kind = match ModelEvidenceKind::parse(draft.evidence_kind) {
        Ok(kind) => Some(kind),
        Err(issue) => {
            diagnostics.push(ModelEvidenceDiagnostic::new(
                issue,
                EVIDENCE_KIND_FIELD.as_str(),
                "model evidence kind is unsupported",
            ));
            None
        }
    };
    let model = match ModelArtifactIdentity::new(draft.model) {
        Ok(identity) => Some(identity),
        Err(error) => {
            diagnostics.extend(error.diagnostics);
            None
        }
    };
    let property_scope = validate_property_scope(draft.property_scope, &mut diagnostics);
    let verifier_kind = match ModelEvidenceVerifierKind::parse(draft.verifier_kind) {
        Ok(kind) => Some(kind),
        Err(issue) => {
            diagnostics.push(ModelEvidenceDiagnostic::new(
                issue,
                VERIFIER_KIND_FIELD.as_str(),
                "model evidence verifier kind is unsupported",
            ));
            None
        }
    };
    let verifier_identity = validate_safe_label(VERIFIER_IDENTITY_FIELD, draft.verifier_identity, &mut diagnostics);
    let input_corpus_hash = validate_hash(
        "model_evidence_input_corpus_hash",
        draft.input_corpus_hash,
        ModelEvidenceValidationIssue::InvalidCorpusHash,
        &mut diagnostics,
    );
    validate_status_freshness_and_benchmark(draft, evidence_kind, &mut diagnostics);
    validate_supplied_diagnostics(draft.diagnostics, &mut diagnostics);
    let capability_labels = validate_capability_labels(draft.capability_labels, &mut diagnostics);
    let non_claims = validate_non_claims(draft.non_claims, &mut diagnostics);

    if diagnostics.is_empty() {
        return assemble_model_evidence_report(draft, ModelEvidenceAssembleParts {
            report_id,
            evidence_kind,
            model,
            property_scope,
            verifier_kind,
            verifier_identity,
            input_corpus_hash,
            capability_labels,
            non_claims,
        });
    }
    Err(ModelEvidenceError { diagnostics })
}

struct ModelEvidenceAssembleParts {
    report_id: Option<String>,
    evidence_kind: Option<ModelEvidenceKind>,
    model: Option<ModelArtifactIdentity>,
    property_scope: Option<String>,
    verifier_kind: Option<ModelEvidenceVerifierKind>,
    verifier_identity: Option<String>,
    input_corpus_hash: Option<crate::ArtifactHash>,
    capability_labels: Option<Vec<ModelEvidenceCapabilityLabel>>,
    non_claims: Option<Vec<ModelEvidenceNonClaim>>,
}

fn assemble_model_evidence_report(
    draft: ModelEvidenceReportDraft<'_>,
    parts: ModelEvidenceAssembleParts,
) -> Result<ModelEvidenceReport, ModelEvidenceError> {
    let (
        Some(report_id),
        Some(evidence_kind),
        Some(model),
        Some(property_scope),
        Some(verifier_kind),
        Some(verifier_identity),
        Some(input_corpus_hash),
        Some(capability_labels),
        Some(non_claims),
    ) = (
        parts.report_id,
        parts.evidence_kind,
        parts.model,
        parts.property_scope,
        parts.verifier_kind,
        parts.verifier_identity,
        parts.input_corpus_hash,
        parts.capability_labels,
        parts.non_claims,
    )
    else {
        return Err(ModelEvidenceError {
            diagnostics: alloc::vec![ModelEvidenceDiagnostic::new(
                ModelEvidenceValidationIssue::InvalidField,
                "model_evidence_report",
                "model evidence validation ended without required fields",
            )],
        });
    };
    let receipt = build_model_evidence_report_receipt(&ModelEvidenceReportReceiptInput {
        draft,
        report_id: report_id.as_str(),
        evidence_kind,
        model: &model,
        property_scope: property_scope.as_str(),
        verifier_kind,
        verifier_identity: verifier_identity.as_str(),
        input_corpus_hash: &input_corpus_hash,
        capability_labels: capability_labels.as_slice(),
        non_claims: non_claims.as_slice(),
    })?;
    Ok(ModelEvidenceReport {
        report_id,
        evidence_kind,
        model,
        property_scope,
        verifier_kind,
        verifier_identity,
        input_corpus_hash,
        result_status: draft.result_status,
        freshness: draft.freshness,
        supporting_evidence_count: draft.supporting_evidence_count,
        diagnostics: draft.diagnostics.to_vec(),
        capability_labels,
        non_claims,
        receipt,
    })
}

#[derive(Clone, Copy)]
struct ModelEvidenceReportReceiptInput<'a> {
    draft: ModelEvidenceReportDraft<'a>,
    report_id: &'a str,
    evidence_kind: ModelEvidenceKind,
    model: &'a ModelArtifactIdentity,
    property_scope: &'a str,
    verifier_kind: ModelEvidenceVerifierKind,
    verifier_identity: &'a str,
    input_corpus_hash: &'a crate::ArtifactHash,
    capability_labels: &'a [ModelEvidenceCapabilityLabel],
    non_claims: &'a [ModelEvidenceNonClaim],
}

fn build_model_evidence_report_receipt(
    input: &ModelEvidenceReportReceiptInput<'_>,
) -> Result<crate::safe::receipt::Record, ModelEvidenceError> {
    build_receipt(
        &ReceiptInput {
            report_id: input.report_id,
            evidence_kind: input.evidence_kind,
            model: input.model,
            property_scope: input.property_scope,
            verifier_kind: input.verifier_kind,
            verifier_identity: input.verifier_identity,
            input_corpus_hash: input.input_corpus_hash,
            result_status: input.draft.result_status,
            freshness: input.draft.freshness,
            supporting_evidence_count: input.draft.supporting_evidence_count,
            diagnostic_count: len_to_u32(input.draft.diagnostics.len()),
            capability_count: len_to_u32(input.capability_labels.len()),
            non_claim_count: len_to_u32(input.non_claims.len()),
            capabilities: input.capability_labels,
            non_claims: input.non_claims,
            extra_label: None,
        },
        crate::safe::receipt::Operation::AdapterBoundary,
    )
    .map_err(|diagnostic| ModelEvidenceError {
        diagnostics: alloc::vec![diagnostic],
    })
}

fn validate_property_scope(value: &str, diagnostics: &mut Vec<ModelEvidenceDiagnostic>) -> Option<String> {
    let scope = validate_safe_label(PROPERTY_SCOPE_FIELD, value, diagnostics);
    if scope.is_none() {
        diagnostics.push(ModelEvidenceDiagnostic::new(
            ModelEvidenceValidationIssue::MissingPropertyScope,
            PROPERTY_SCOPE_FIELD.as_str(),
            "model evidence requires a property scope",
        ));
    }
    scope
}

fn validate_status_freshness_and_benchmark(
    draft: ModelEvidenceReportDraft<'_>,
    evidence_kind: Option<ModelEvidenceKind>,
    diagnostics: &mut Vec<ModelEvidenceDiagnostic>,
) {
    if draft.freshness == ModelEvidenceFreshness::Stale {
        diagnostics.push(ModelEvidenceDiagnostic::new(
            ModelEvidenceValidationIssue::StaleFreshness,
            "model_evidence_freshness",
            "model evidence freshness envelope is stale",
        ));
    }
    if evidence_kind == Some(ModelEvidenceKind::BenchmarkSummary)
        && draft.supporting_evidence_count < MIN_SUPPORTING_EVIDENCE_COUNT
    {
        diagnostics.push(ModelEvidenceDiagnostic::new(
            ModelEvidenceValidationIssue::BenchmarkOnly,
            FIELD_SUPPORTING_EVIDENCE_COUNT,
            "benchmark-only evidence remains insufficient for admission or routing promotion",
        ));
    }
    if draft.result_status != ModelEvidenceResultStatus::Passed && draft.diagnostics.is_empty() {
        diagnostics.push(ModelEvidenceDiagnostic::new(
            ModelEvidenceValidationIssue::MissingDiagnostics,
            DIAGNOSTIC_FIELD.as_str(),
            "non-passing model evidence requires safe diagnostics",
        ));
    }
}

fn validate_supplied_diagnostics(supplied: &[ModelEvidenceDiagnostic], diagnostics: &mut Vec<ModelEvidenceDiagnostic>) {
    if supplied.is_empty() {
        diagnostics.push(ModelEvidenceDiagnostic::new(
            ModelEvidenceValidationIssue::MissingDiagnostics,
            DIAGNOSTIC_FIELD.as_str(),
            "model evidence requires at least one safe diagnostic",
        ));
    }
    if supplied.len() > MAX_DIAGNOSTICS {
        diagnostics.push(ModelEvidenceDiagnostic::new(
            ModelEvidenceValidationIssue::TooManyItems,
            DIAGNOSTIC_FIELD.as_str(),
            "model evidence diagnostic count exceeds DTO bound",
        ));
    }
    for diagnostic in supplied.iter().take(MAX_DIAGNOSTICS) {
        validate_bounded_safe_text(DIAGNOSTIC_FIELD, diagnostic.message(), MAX_DIAGNOSTIC_CHARS, diagnostics);
    }
}

fn validate_capability_labels(
    values: &[ModelEvidenceCapabilityLabel],
    diagnostics: &mut Vec<ModelEvidenceDiagnostic>,
) -> Option<Vec<ModelEvidenceCapabilityLabel>> {
    if values.is_empty() {
        diagnostics.push(ModelEvidenceDiagnostic::new(
            ModelEvidenceValidationIssue::InvalidField,
            "model_evidence_capabilities",
            "model evidence requires an advisory capability label",
        ));
    }
    if values.len() > MAX_CAPABILITY_LABELS {
        diagnostics.push(ModelEvidenceDiagnostic::new(
            ModelEvidenceValidationIssue::TooManyItems,
            "model_evidence_capabilities",
            "model evidence capability count exceeds DTO bound",
        ));
    }
    if values.iter().any(|label| label.is_overclaim()) {
        diagnostics.push(ModelEvidenceDiagnostic::new(
            ModelEvidenceValidationIssue::CapabilityOverclaim,
            "model_evidence_capabilities",
            "model evidence capability labels must remain advisory",
        ));
    }
    if diagnostics.iter().any(|diagnostic| diagnostic.field() == "model_evidence_capabilities") {
        return None;
    }
    let mut capabilities = values.to_vec();
    capabilities.sort();
    capabilities.dedup();
    Some(capabilities)
}

fn validate_non_claims(
    values: &[ModelEvidenceNonClaim],
    diagnostics: &mut Vec<ModelEvidenceDiagnostic>,
) -> Option<Vec<ModelEvidenceNonClaim>> {
    for required in required_model_non_claims() {
        if !contains_non_claim(values, *required) {
            diagnostics.push(ModelEvidenceDiagnostic::new(
                ModelEvidenceValidationIssue::MissingNonClaim,
                "model_evidence_non_claims",
                "model evidence report is missing a required non-claim",
            ));
            break;
        }
    }
    if diagnostics.iter().any(|diagnostic| diagnostic.field() == "model_evidence_non_claims") {
        return None;
    }
    let mut non_claims = values.to_vec();
    non_claims.sort();
    non_claims.dedup();
    Some(non_claims)
}

fn contains_non_claim(values: &[ModelEvidenceNonClaim], target: ModelEvidenceNonClaim) -> bool {
    values.contains(&target)
}

fn required_model_non_claims() -> &'static [ModelEvidenceNonClaim; REQUIRED_MODEL_NON_CLAIMS] {
    &[
        ModelEvidenceNonClaim::ArbitraryRobustness,
        ModelEvidenceNonClaim::OutputCorrectness,
        ModelEvidenceNonClaim::HallucinationPrevention,
        ModelEvidenceNonClaim::VerifierSoundness,
        ModelEvidenceNonClaim::BenchmarkSuperiority,
        ModelEvidenceNonClaim::ReleaseEligibility,
        ModelEvidenceNonClaim::GpuCorrectness,
        ModelEvidenceNonClaim::WholeSystemSafety,
    ]
}

struct ReceiptInput<'a> {
    report_id: &'a str,
    evidence_kind: ModelEvidenceKind,
    model: &'a ModelArtifactIdentity,
    property_scope: &'a str,
    verifier_kind: ModelEvidenceVerifierKind,
    verifier_identity: &'a str,
    input_corpus_hash: &'a crate::ArtifactHash,
    result_status: ModelEvidenceResultStatus,
    freshness: ModelEvidenceFreshness,
    supporting_evidence_count: u32,
    diagnostic_count: u32,
    capability_count: u32,
    non_claim_count: u32,
    capabilities: &'a [ModelEvidenceCapabilityLabel],
    non_claims: &'a [ModelEvidenceNonClaim],
    extra_label: Option<&'a str>,
}

fn build_receipt(
    input: &ReceiptInput<'_>,
    operation: crate::safe::receipt::Operation,
) -> Result<crate::safe::receipt::Record, ModelEvidenceDiagnostic> {
    let label_storage = receipt_labels(input);
    let label_refs = label_storage.iter().map(String::as_str).collect::<Vec<_>>();
    let counts = [
        crate::safe::receipt::BoundedCountDraft::new(FIELD_RESULT_COUNT, RESULT_COUNT),
        crate::safe::receipt::BoundedCountDraft::new(FIELD_SUPPORTING_EVIDENCE_COUNT, input.supporting_evidence_count),
        crate::safe::receipt::BoundedCountDraft::new(FIELD_DIAGNOSTIC_COUNT, input.diagnostic_count),
        crate::safe::receipt::BoundedCountDraft::new(FIELD_CAPABILITY_COUNT, input.capability_count),
        crate::safe::receipt::BoundedCountDraft::new(FIELD_NON_CLAIM_COUNT, input.non_claim_count),
    ];
    let hashes = [input.model.model_hash().clone(), input.input_corpus_hash.clone()];
    let redactions = [
        crate::safe::receipt::Redaction::OutputBounded,
        crate::safe::receipt::Redaction::PayloadHashOnly,
        crate::safe::receipt::Redaction::ProviderPayloadOmitted,
    ];
    crate::safe::receipt::Record::new(
        crate::safe::receipt::Draft::new(
            crate::safe::receipt::Status::Completed,
            crate::safe::receipt::IssueCode::ProvenanceObservation,
            operation,
        )
        .with_labels(label_refs.as_slice())
        .with_counts(&counts)
        .with_artifact_hashes(&hashes)
        .with_backend_label(RECEIPT_BACKEND_LABEL)
        .with_interchange_label(RECEIPT_INTERCHANGE_LABEL)
        .with_redactions(&redactions),
    )
    .map_err(|_error| {
        ModelEvidenceDiagnostic::new(
            ModelEvidenceValidationIssue::SafeReceiptRejected,
            "model_evidence_receipt",
            "safe receipt rejected model evidence metadata",
        )
    })
}

fn receipt_labels(input: &ReceiptInput<'_>) -> Vec<String> {
    let mut labels = alloc::vec![
        String::from(input.report_id),
        String::from(input.evidence_kind.as_str()),
        String::from(input.model.model_label()),
        String::from(input.property_scope),
        String::from(input.verifier_kind.as_str()),
        String::from(input.verifier_identity),
        String::from(input.result_status.as_str()),
        String::from(input.freshness.as_str()),
    ];
    labels.extend(input.capabilities.iter().map(|label| String::from(label.as_str())));
    labels.extend(input.non_claims.iter().map(|claim| String::from(claim.as_str())));
    if let Some(extra_label) = input.extra_label {
        labels.push(String::from(extra_label));
    }
    labels
}

fn validate_hash(
    field: &'static str,
    value: &str,
    issue: ModelEvidenceValidationIssue,
    diagnostics: &mut Vec<ModelEvidenceDiagnostic>,
) -> Option<crate::ArtifactHash> {
    match crate::ArtifactHash::new(value) {
        Ok(hash) => Some(hash),
        Err(_error) => {
            diagnostics.push(ModelEvidenceDiagnostic::new(
                issue,
                field,
                "model evidence hash must be lowercase BLAKE3 hex",
            ));
            None
        }
    }
}

fn validate_safe_label(
    field: FieldName,
    value: &str,
    diagnostics: &mut Vec<ModelEvidenceDiagnostic>,
) -> Option<String> {
    match normalize_label(field, value) {
        Ok(label) if is_safe_text(label.as_str()) => Some(label),
        Ok(_unsafe) => {
            diagnostics.push(ModelEvidenceDiagnostic::new(
                ModelEvidenceValidationIssue::UnsafeMetadata,
                field.as_str(),
                "model evidence metadata contains unsafe raw material marker",
            ));
            None
        }
        Err(issue) => {
            diagnostics.push(ModelEvidenceDiagnostic::new(
                issue,
                field.as_str(),
                "model evidence metadata field is invalid",
            ));
            None
        }
    }
}

fn normalize_label(field: FieldName, value: &str) -> Result<String, ModelEvidenceValidationIssue> {
    match validation::normalized_adapter_field(field, value) {
        Ok(label) => Ok(label),
        Err(
            AdapterContractError::EmptyField { .. }
            | AdapterContractError::FieldTooLong { .. }
            | AdapterContractError::NulCharacter { .. },
        ) => Err(ModelEvidenceValidationIssue::InvalidField),
        Err(_error) => Err(ModelEvidenceValidationIssue::InvalidField),
    }
}

fn validate_bounded_safe_text(
    field: FieldName,
    value: &str,
    max_chars: usize,
    diagnostics: &mut Vec<ModelEvidenceDiagnostic>,
) -> Option<String> {
    let text = validate_safe_label(field, value, diagnostics)?;
    if text.chars().count() > max_chars {
        diagnostics.push(ModelEvidenceDiagnostic::new(
            ModelEvidenceValidationIssue::InvalidField,
            field.as_str(),
            "model evidence diagnostic exceeds DTO character budget",
        ));
        return None;
    }
    Some(text)
}

fn is_safe_text(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    !UNSAFE_TEXT_MARKERS.iter().any(|marker| lower.contains(marker))
}

fn len_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(crate::safe::receipt::MAX_COUNT)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODEL_HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const MODEL_HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const CORPUS_HASH: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const REPORT_ID: &str = "model-evidence:alpha";
    const MODEL_LABEL: &str = "model:alpha";
    const PROPERTY_SCOPE: &str = "classification-bound:local";
    const VERIFIER_IDENTITY: &str = "gamma-crown-fixture";
    const ADMISSION_LABEL: &str = "admission:advisory";

    const EXPECTED_ITEM_COUNT: usize = 1;

    struct Fixture {
        diagnostics: [ModelEvidenceDiagnostic; EXPECTED_ITEM_COUNT],
        capability_labels: [ModelEvidenceCapabilityLabel; EXPECTED_ITEM_COUNT],
        non_claims: [ModelEvidenceNonClaim; REQUIRED_MODEL_NON_CLAIMS],
    }

    impl Fixture {
        fn new() -> Self {
            Self {
                diagnostics: [ModelEvidenceDiagnostic::new(
                    ModelEvidenceValidationIssue::InvalidField,
                    "fixture",
                    "safe fixture diagnostic",
                )],
                capability_labels: [ModelEvidenceCapabilityLabel::AdvisoryRouting],
                non_claims: *required_model_non_claims(),
            }
        }

        fn draft(&self) -> ModelEvidenceReportDraft<'_> {
            ModelEvidenceReportDraft {
                report_id: REPORT_ID,
                evidence_kind: "bound-report",
                model: ModelArtifactIdentityDraft {
                    model_label: MODEL_LABEL,
                    model_hash: MODEL_HASH_A,
                    observed_model_hash: MODEL_HASH_A,
                },
                property_scope: PROPERTY_SCOPE,
                verifier_kind: "gamma-crown",
                verifier_identity: VERIFIER_IDENTITY,
                input_corpus_hash: CORPUS_HASH,
                result_status: ModelEvidenceResultStatus::Passed,
                freshness: ModelEvidenceFreshness::Current,
                supporting_evidence_count: MIN_SUPPORTING_EVIDENCE_COUNT,
                diagnostics: &self.diagnostics,
                capability_labels: &self.capability_labels,
                non_claims: &self.non_claims,
            }
        }
    }

    #[test]
    fn model_evidence_accepts_bounded_positive_fixtures() {
        // r[verify model-evidence-adapters.positive]
        // r[verify model-evidence-adapters.report-dto]
        let fixture = Fixture::new();
        let bound = validate_model_evidence_report(fixture.draft()).expect("valid bound report should pass");
        assert_eq!(bound.evidence_kind(), ModelEvidenceKind::BoundReport);
        assert_eq!(bound.model().model_hash().as_str(), MODEL_HASH_A);

        let mut conversion = fixture.draft();
        conversion.evidence_kind = "conversion-report";
        conversion.verifier_kind = "mly-conversion";
        let conversion = validate_model_evidence_report(conversion).expect("valid conversion report should pass");
        assert_eq!(conversion.evidence_kind(), ModelEvidenceKind::ConversionReport);

        let mut kernel = fixture.draft();
        kernel.evidence_kind = "kernel-proof-summary";
        kernel.verifier_kind = "kernel-proof";
        let kernel = validate_model_evidence_report(kernel).expect("valid kernel proof report should pass");
        assert_eq!(kernel.evidence_kind(), ModelEvidenceKind::KernelProofSummary);
        assert!(PROHIBITED_MODEL_EVIDENCE_EFFECTS.contains(&"GPU access"));
    }

    #[test]
    fn model_evidence_advisory_receipt_preserves_non_claim_labels() {
        // r[verify model-evidence-adapters.validation]
        let fixture = Fixture::new();
        let report = validate_model_evidence_report(fixture.draft()).expect("valid model evidence report should pass");
        let receipt = report
            .advisory_decision_receipt(ADMISSION_LABEL)
            .expect("advisory receipt should preserve non-claims");

        assert!(receipt.labels().iter().any(|label| label == ModelEvidenceNonClaim::OutputCorrectness.as_str()));
        assert!(receipt.labels().iter().any(|label| label == ADMISSION_LABEL));
    }

    #[test]
    fn model_evidence_rejects_negative_fixtures() {
        // r[verify model-evidence-adapters.negative]
        // r[verify model-evidence-adapters.core-validation]
        let fixture = Fixture::new();
        let mut stale = fixture.draft();
        stale.freshness = ModelEvidenceFreshness::Stale;
        assert_issue(validate_model_evidence_report(stale), ModelEvidenceValidationIssue::StaleFreshness);

        let mut mismatch = fixture.draft();
        mismatch.model.observed_model_hash = MODEL_HASH_B;
        assert_issue(validate_model_evidence_report(mismatch), ModelEvidenceValidationIssue::MismatchedModel);

        let mut unsupported_verifier = fixture.draft();
        unsupported_verifier.verifier_kind = "unsupported-verifier";
        assert_issue(
            validate_model_evidence_report(unsupported_verifier),
            ModelEvidenceValidationIssue::UnsupportedVerifier,
        );

        let mut missing_property = fixture.draft();
        missing_property.property_scope = "";
        assert_issue(
            validate_model_evidence_report(missing_property),
            ModelEvidenceValidationIssue::MissingPropertyScope,
        );

        let mut benchmark_only = fixture.draft();
        benchmark_only.evidence_kind = "benchmark-summary";
        benchmark_only.verifier_kind = "benchmark-harness";
        benchmark_only.supporting_evidence_count = ZERO_COUNT;
        assert_issue(validate_model_evidence_report(benchmark_only), ModelEvidenceValidationIssue::BenchmarkOnly);

        let mut weakened_non_claim = fixture.draft();
        weakened_non_claim.non_claims = &[];
        assert_issue(validate_model_evidence_report(weakened_non_claim), ModelEvidenceValidationIssue::MissingNonClaim);

        let overclaim_capabilities = [ModelEvidenceCapabilityLabel::ProductionSafety];
        let mut overclaim = fixture.draft();
        overclaim.capability_labels = &overclaim_capabilities;
        assert_issue(validate_model_evidence_report(overclaim), ModelEvidenceValidationIssue::CapabilityOverclaim);
    }

    fn assert_issue(result: Result<ModelEvidenceReport, ModelEvidenceError>, issue: ModelEvidenceValidationIssue) {
        let error = result.expect_err("fixture should fail");
        assert!(
            error.diagnostics().iter().any(|diagnostic| diagnostic.issue() == issue),
            "expected issue {issue:?}, got {:?}",
            error.diagnostics()
        );
    }
}
