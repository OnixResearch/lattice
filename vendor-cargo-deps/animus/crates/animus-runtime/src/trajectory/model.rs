use animus_adapters::trajectory::{
    DiagnosticDisposition, TrajectoryDiagnosticCode, TrajectoryRecord, TrajectorySource,
};

/// Stable validation or projection failure class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrajectoryIssue {
    /// Policy cohort or helper identity differs from the reviewed cohort.
    Cohort,
    /// Policy has a missing, duplicate, or invalid source.
    SourcePolicy,
    /// A required bound is zero or inconsistent.
    BoundPolicy,
    /// Diagnostic policy is missing, duplicated, or invalid.
    DiagnosticPolicy,
    /// Raw request exceeds a bound or has an invalid identity.
    Request,
    /// Result exceeds a record, content, tool, or diagnostic bound.
    ResultBound,
    /// Leading meta record is missing, duplicated, misplaced, or mismatched.
    Meta,
    /// Record timestamp is malformed or order is invalid.
    Order,
    /// Required content is empty or contains a forbidden shape.
    Content,
    /// Tool arguments are not a JSON object.
    ToolArguments,
    /// Tool call identity is empty or duplicated.
    DuplicateToolCall,
    /// Tool result is orphaned or duplicated.
    ToolResult,
    /// Diagnostic code is unknown or rejected by policy.
    Diagnostic,
    /// Existing Animus ledger draft rejected a projected field.
    LedgerProjection,
    /// A receipt draft contains sensitive material or unsafe claims.
    Receipt,
}

/// Normalized role class used only for bounded counts and projection metadata.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TrajectoryRole {
    /// Leading source metadata.
    Meta,
    /// User message.
    User,
    /// Hidden reasoning.
    Reasoning,
    /// Assistant message.
    Assistant,
    /// Assistant tool-call group.
    AssistantToolCalls,
    /// Tool result.
    Tool,
}

impl TrajectoryRole {
    /// Returns a stable metadata label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Meta => "meta",
            Self::User => "user",
            Self::Reasoning => "reasoning",
            Self::Assistant => "assistant",
            Self::AssistantToolCalls => "assistant-tool-calls",
            Self::Tool => "tool",
        }
    }
}

/// Bounded role counts from validated records.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TrajectoryRoleCounts {
    /// Meta records.
    pub meta: u32,
    /// User records.
    pub user: u32,
    /// Reasoning records.
    pub reasoning: u32,
    /// Assistant message records.
    pub assistant: u32,
    /// Assistant tool-call records.
    pub assistant_tool_calls: u32,
    /// Tool result records.
    pub tool: u32,
}

impl TrajectoryRoleCounts {
    pub(crate) fn add(&mut self, role: TrajectoryRole) -> Result<(), TrajectoryIssue> {
        let target = match role {
            TrajectoryRole::Meta => &mut self.meta,
            TrajectoryRole::User => &mut self.user,
            TrajectoryRole::Reasoning => &mut self.reasoning,
            TrajectoryRole::Assistant => &mut self.assistant,
            TrajectoryRole::AssistantToolCalls => &mut self.assistant_tool_calls,
            TrajectoryRole::Tool => &mut self.tool,
        };
        *target = target.checked_add(1).ok_or(TrajectoryIssue::ResultBound)?;
        Ok(())
    }
}

/// Pure validation result over already-loaded values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrajectoryValidation {
    /// Validated source.
    pub source: TrajectorySource,
    /// Source-generation identity supplied by the caller.
    pub source_generation_identity: String,
    /// Policy identity used for admission.
    pub policy_identity: String,
    /// Ordered validated records.
    pub records: Vec<TrajectoryRecord>,
    /// Role counts.
    pub role_counts: TrajectoryRoleCounts,
    /// Diagnostic code and occurrence count rows.
    pub diagnostic_counts: Vec<(TrajectoryDiagnosticCode, u32)>,
    /// Most conservative admitted diagnostic disposition.
    pub disposition: DiagnosticDisposition,
}

/// One deterministic batch-owner draft. It does not dispatch or persist work.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrajectoryBatchDraft {
    /// Domain-separated projection identity.
    pub identity: String,
    /// Zero-based validated record position.
    pub position: u32,
    /// Record role.
    pub role: TrajectoryRole,
    /// Descriptive timestamp, never freshness or causality evidence.
    pub descriptive_timestamp: Option<String>,
    /// Whether record content was omitted.
    pub content_omitted: bool,
}

/// Explicit omission class.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TrajectoryOmission {
    /// Hidden reasoning was omitted.
    Reasoning,
    /// Tool arguments were omitted.
    ToolArguments,
    /// Tool results were omitted.
    ToolResults,
    /// Path, branch, or model metadata was omitted.
    SourceMetadata,
}

impl TrajectoryOmission {
    /// Returns a stable omission label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Reasoning => "reasoning-omitted",
            Self::ToolArguments => "tool-arguments-omitted",
            Self::ToolResults => "tool-results-omitted",
            Self::SourceMetadata => "source-metadata-omitted",
        }
    }
}

/// Deterministic projection into existing ledger entries and batch drafts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrajectoryProjection {
    /// Existing neutral ledger entries. No persistence occurs.
    pub ledger_entries: Vec<animus_adapters::LedgerEntry>,
    /// Batch-owner drafts. No batch execution occurs.
    pub batch_drafts: Vec<TrajectoryBatchDraft>,
    /// Explicit omission classes.
    pub omissions: Vec<TrajectoryOmission>,
}

/// Sensitive receipt fields accepted only as explicit `None` values.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TrajectoryReceiptSensitiveDraft<'a> {
    /// Raw transcript.
    pub transcript: Option<&'a str>,
    /// Hidden reasoning.
    pub reasoning: Option<&'a str>,
    /// Tool arguments or result.
    pub tool_body: Option<&'a str>,
    /// Diagnostic prose.
    pub diagnostic_message: Option<&'a str>,
    /// Raw path, cwd, or branch.
    pub path_metadata: Option<&'a str>,
    /// Credential or environment value.
    pub credential: Option<&'a str>,
    /// Provider payload.
    pub provider_payload: Option<&'a str>,
    /// Unapproved content-derived fingerprint.
    pub content_fingerprint: Option<&'a str>,
    /// Unbounded helper output.
    pub helper_output: Option<&'a str>,
    /// Unsafe authenticity, completeness, privacy, or execution claim.
    pub claims_truth: bool,
}

/// Inputs for metadata-only receipt projection.
#[derive(Clone, Copy, Debug)]
pub struct TrajectoryReceiptInput<'a> {
    /// Validated normalized records.
    pub validation: &'a TrajectoryValidation,
    /// Existing-owner projection.
    pub projection: &'a TrajectoryProjection,
    /// Exact helper identity.
    pub helper_identity: &'a str,
    /// Injected service identity.
    pub service_identity: &'a str,
    /// Explicitly absent sensitive fields and unsafe claims.
    pub sensitive: TrajectoryReceiptSensitiveDraft<'a>,
}

/// Metadata-only adapter receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrajectoryReceipt {
    /// Receipt schema.
    pub schema: &'static str,
    /// Exact cohort identity.
    pub cohort_identity: String,
    /// Policy identity.
    pub policy_identity: String,
    /// Helper identity.
    pub helper_identity: String,
    /// Injected service identity.
    pub service_identity: String,
    /// Safe source class.
    pub source: TrajectorySource,
    /// Role counts.
    pub role_counts: TrajectoryRoleCounts,
    /// Diagnostic counts.
    pub diagnostic_counts: Vec<(TrajectoryDiagnosticCode, u32)>,
    /// Diagnostic disposition.
    pub disposition: DiagnosticDisposition,
    /// Omission classes.
    pub omissions: Vec<TrajectoryOmission>,
    /// Projection target class.
    pub projection_target: &'static str,
    /// Required non-claims.
    pub non_claims: [&'static str; 9],
    /// Domain-separated BLAKE3 receipt identity.
    pub receipt_identity: String,
}
