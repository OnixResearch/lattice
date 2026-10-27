/// Bounded ATIF issue class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AtifIssueClass {
    /// Profile or policy is stale or unsafe.
    Policy,
    /// JSON is malformed, duplicated, oversized, or unsupported.
    Codec,
    /// Schema version does not match the reviewed cohort.
    Version,
    /// Required identity is absent or unsafe.
    Identity,
    /// A local or aggregate bound was exceeded.
    Bound,
    /// Step order is invalid.
    StepSequence,
    /// Timestamp syntax or range is invalid.
    Timestamp,
    /// A source-specific field is misused.
    SourceField,
    /// Text or multimodal content is malformed.
    Content,
    /// Tool-call or observation correlation is invalid.
    ToolCorrelation,
    /// Metric values or relationships are invalid.
    Metric,
    /// No-LLM dispatch semantics are contradicted.
    NoLlm,
    /// Copied-context data was offered to an ineligible target.
    CopiedContext,
    /// Subagent or external reference semantics are invalid.
    Reference,
    /// Caller-supplied resolved graph contains a cycle.
    Cycle,
    /// Extension data is unsafe or exceeds policy.
    Extension,
    /// A caller requested ambient authority or automatic promotion.
    Authority,
    /// Receipt material is private or overclaiming.
    Evidence,
}

/// Bounded schema location.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AtifLocation {
    /// Checked profile.
    Profile,
    /// Root trajectory.
    Root,
    /// Agent object.
    Agent,
    /// One-based step.
    Step(u32),
    /// Content part within one step.
    Content { step: u32, part: u32 },
    /// Tool call within one step.
    ToolCall { step: u32, call: u32 },
    /// Observation result within one step.
    Observation { step: u32, result: u32 },
    /// Metrics on one step.
    Metrics(u32),
    /// Embedded subagent at a bounded depth and index.
    Subagent { depth: u32, index: u32 },
    /// Extension object.
    Extension,
    /// Receipt draft.
    Receipt,
    /// Caller-supplied resolved graph.
    Graph,
}

/// Safe field class associated with a diagnostic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AtifField {
    /// Schema version.
    Schema,
    /// Identity or name.
    Identity,
    /// Collection or byte bound.
    Bound,
    /// Step ID.
    StepId,
    /// Timestamp.
    Timestamp,
    /// Source.
    Source,
    /// Message or content.
    Content,
    /// Tool metadata.
    Tool,
    /// Observation metadata.
    Observation,
    /// Metrics.
    Metrics,
    /// LLM call count.
    LlmCallCount,
    /// Copied-context marker.
    CopiedContext,
    /// Embedded or external reference.
    Reference,
    /// Extension key or value.
    Extension,
    /// Claim or private receipt field.
    Evidence,
    /// Projection target.
    Projection,
}

/// Deterministic error without sensitive values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AtifDiagnostic {
    /// Issue class.
    pub class: AtifIssueClass,
    /// Bounded schema location.
    pub location: AtifLocation,
    /// Safe field class.
    pub field: AtifField,
}

impl AtifDiagnostic {
    pub(crate) const fn new(
        class: AtifIssueClass,
        location: AtifLocation,
        field: AtifField,
    ) -> Self {
        Self {
            class,
            location,
            field,
        }
    }
}
