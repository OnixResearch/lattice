const MAX_CONTRACT_ENTRIES: usize = 64;
const MAX_CONTRACT_TEXT_BYTES: usize = 128;
const MAX_CONTRACT_ISSUES: usize = 80;

/// Ownership policy for accepted component state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadlessStateOwnership {
    /// Ratcore reducer commits accepted state.
    ReducerOwned,
    /// The caller commits proposed state changes.
    CallerControlled,
    /// The contract names separately owned state fields.
    Split,
}

/// Required facet of a headless component declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HeadlessContractFacet {
    /// Accepted model and construction invariants.
    AcceptedModel,
    /// Explicit configuration and policy choices.
    Configuration,
    /// Stable semantic identities.
    StableIdentities,
    /// Typed renderer-neutral messages and intents.
    TypedMessages,
    /// Deterministic transitions and outputs.
    DeterministicTransition,
    /// Immutable named-part projection.
    NamedParts,
    /// Caller-owned arbitrary content slots.
    CallerOwnedSlots,
    /// Shared focus and semantic projection.
    FocusAndSemantics,
    /// Renderer capability and fallback metadata.
    RendererCapabilities,
    /// Stable positive and negative diagnostics evidence.
    DiagnosticsAndEvidence,
}

impl HeadlessContractFacet {
    fn all() -> [Self; 10] {
        [
            Self::AcceptedModel,
            Self::Configuration,
            Self::StableIdentities,
            Self::TypedMessages,
            Self::DeterministicTransition,
            Self::NamedParts,
            Self::CallerOwnedSlots,
            Self::FocusAndSemantics,
            Self::RendererCapabilities,
            Self::DiagnosticsAndEvidence,
        ]
    }
}

/// Executable declaration for one headless component family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadlessContractDeclaration {
    /// Stable component-family identity.
    pub component_id: String,
    /// Accepted-state ownership policy.
    pub ownership: Option<HeadlessStateOwnership>,
    /// Explicitly implemented contract facets.
    pub facets: std::collections::BTreeSet<HeadlessContractFacet>,
    /// Public shared type paths reviewed for framework leakage.
    pub shared_type_paths: Vec<String>,
    /// Whether an adapter independently recomputes shared semantics.
    pub adapter_recomputes_semantics: bool,
    /// Whether rejected paths expose typed diagnostics.
    pub diagnostics_exposed: bool,
    /// Named positive and negative evidence.
    pub evidence: Vec<String>,
}

/// Stable contract declaration diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HeadlessContractIssueCode {
    /// Component identity is empty or malformed.
    InvalidComponentIdentity,
    /// State ownership is unspecified.
    MissingOwnership,
    /// A required contract facet is absent.
    MissingFacet,
    /// Shared API names a renderer or framework type.
    FrameworkTypeLeak,
    /// Adapter duplicates semantic decisions.
    AdapterRecomputesSemantics,
    /// Rejected behavior hides diagnostics.
    DiagnosticsHidden,
    /// Positive and negative evidence is absent.
    MissingEvidence,
    /// A bounded declaration field is too large.
    DeclarationTooLarge,
}

/// One deterministic contract declaration diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HeadlessContractIssue {
    /// Stable diagnostic code.
    pub code: HeadlessContractIssueCode,
    /// Stable component or facet subject.
    pub subject: String,
}

/// Validates one headless component declaration.
#[must_use]
pub fn validate_headless_contract(declaration: &HeadlessContractDeclaration) -> Vec<HeadlessContractIssue> {
    let mut issues = Vec::with_capacity(MAX_CONTRACT_ISSUES);
    if !valid_contract_text(&declaration.component_id) {
        issues.push(contract_issue(HeadlessContractIssueCode::InvalidComponentIdentity, &declaration.component_id));
    }
    if declaration.ownership.is_none() {
        issues.push(contract_issue(HeadlessContractIssueCode::MissingOwnership, &declaration.component_id));
    }
    for facet in HeadlessContractFacet::all() {
        if !declaration.facets.contains(&facet) {
            issues.push(contract_issue(
                HeadlessContractIssueCode::MissingFacet,
                &format!("{}:{facet:?}", declaration.component_id),
            ));
        }
    }
    if declaration.shared_type_paths.len() > MAX_CONTRACT_ENTRIES || declaration.evidence.len() > MAX_CONTRACT_ENTRIES {
        issues.push(contract_issue(HeadlessContractIssueCode::DeclarationTooLarge, &declaration.component_id));
    }
    for path in declaration.shared_type_paths.iter().take(MAX_CONTRACT_ENTRIES) {
        if framework_path(path) {
            issues.push(contract_issue(HeadlessContractIssueCode::FrameworkTypeLeak, path));
        }
    }
    if declaration.adapter_recomputes_semantics {
        issues.push(contract_issue(HeadlessContractIssueCode::AdapterRecomputesSemantics, &declaration.component_id));
    }
    if !declaration.diagnostics_exposed {
        issues.push(contract_issue(HeadlessContractIssueCode::DiagnosticsHidden, &declaration.component_id));
    }
    if declaration.evidence.is_empty() {
        issues.push(contract_issue(HeadlessContractIssueCode::MissingEvidence, &declaration.component_id));
    }
    issues.sort();
    issues.dedup();
    debug_assert!(issues.len() <= MAX_CONTRACT_ISSUES);
    debug_assert!(issues.iter().all(|issue| issue.subject.chars().count() <= MAX_CONTRACT_TEXT_BYTES));
    issues
}

/// Returns the complete declaration used by the tabs pilot.
#[must_use]
pub fn tabs_contract_declaration() -> HeadlessContractDeclaration {
    HeadlessContractDeclaration {
        component_id: "headless.tabs.v1".to_string(),
        ownership: Some(HeadlessStateOwnership::Split),
        facets: HeadlessContractFacet::all().into_iter().collect(),
        shared_type_paths: vec![
            "TabsState".to_string(),
            "TabsMessage".to_string(),
            "TabsTransition".to_string(),
            "TabsProjection".to_string(),
        ],
        adapter_recomputes_semantics: false,
        diagnostics_exposed: true,
        evidence: vec![
            "headless-tabs.normal-v1".to_string(),
            "headless-tabs.invalid-v1".to_string(),
        ],
    }
}

fn valid_contract_text(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_CONTRACT_TEXT_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_'))
}

fn framework_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    [
        "react",
        "radix",
        "dioxus",
        "web_sys",
        "html",
        "dom",
        "ratatui",
        "frame",
        "buffer",
        "crossterm",
        "css",
    ]
    .iter()
    .any(|token| lower.contains(token))
}

fn contract_issue(code: HeadlessContractIssueCode, subject: &str) -> HeadlessContractIssue {
    HeadlessContractIssue {
        code,
        subject: subject.chars().take(MAX_CONTRACT_TEXT_BYTES).collect(),
    }
}
