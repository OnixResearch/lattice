// Public root reexports intentionally keep worker-report naming for embedders.
#![allow(unknown_lints)]
#![allow(path_segment_repetition)]

/// Evidence reference included in a worker report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceReference {
    /// Safe evidence id.
    pub evidence_id: String,
    /// Safe bounded evidence summary.
    pub summary: String,
}

/// Worker claim with safe evidence references.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Claim {
    /// Claim subject label.
    pub subject: String,
    /// Claim value label or summary.
    pub value: String,
    /// Evidence ids supporting the claim.
    pub evidence_ids: Vec<String>,
}

/// Worker mutation intent that still requires normal policy arbitration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MutationIntent {
    /// Safe mutation target label.
    pub target: String,
    /// Safe mutation kind label.
    pub mutation_kind: String,
    /// Evidence id supporting the intent.
    pub evidence_id: String,
}

/// Report produced by a worker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Report {
    /// Safe worker identity.
    pub worker_id: String,
    /// Schema name this report claims to satisfy.
    pub schema_name: String,
    /// Required and optional schema fields.
    pub fields: std::collections::BTreeMap<String, String>,
    /// Safe evidence references.
    pub evidence: Vec<EvidenceReference>,
    /// Safe worker claims.
    pub claims: Vec<Claim>,
    /// Mutation intents requiring arbitration before any effect.
    pub mutation_intents: Vec<MutationIntent>,
}

/// Successful deterministic merge of validated worker reports.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerMergedReport {
    /// Preserved evidence references.
    pub evidence: Vec<EvidenceReference>,
    /// Compatible worker claims.
    pub claims: Vec<Claim>,
    /// Allowed but unresolved mutation intents.
    pub unresolved_mutation_intents: Vec<MutationIntent>,
    /// Denied mutation intents outside worker capabilities.
    pub denied_mutation_intents: Vec<MutationIntent>,
}

/// Structured conflict requiring operator or policy arbitration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConflictReport {
    /// Structured conflicts.
    pub conflicts: Vec<Conflict>,
    /// Preserved evidence references for arbitration.
    pub evidence: Vec<EvidenceReference>,
}

/// One conflict detected during worker report merge.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Conflict {
    /// Stable conflict code.
    pub code: String,
    /// Safe conflict subject label.
    pub subject: String,
    /// Evidence ids relevant to the conflict.
    pub evidence_ids: Vec<String>,
}

/// Worker report merge outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReportMergeOutcome {
    /// Reports merged without conflicts.
    Merged(WorkerMergedReport),
    /// Reports require arbitration before finalization.
    Conflict(ConflictReport),
}
