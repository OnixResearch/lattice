/// Classification of an existing component-family contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadlessComponentClassification {
    /// Family satisfies the executable contract for its declared scope.
    Conforming,
    /// Family is a reviewed shared-behavior candidate.
    Candidate,
    /// Behavior intentionally belongs to one renderer.
    RendererOwned,
    /// Renderers intentionally expose different behavior.
    IntentionallyDivergent,
    /// Extraction or parity is blocked.
    Blocked,
}

impl HeadlessComponentClassification {
    /// Returns the stable inventory spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Conforming => "conforming",
            Self::Candidate => "candidate",
            Self::RendererOwned => "renderer-owned",
            Self::IntentionallyDivergent => "intentionally-divergent",
            Self::Blocked => "blocked",
        }
    }

    pub(super) fn parse(value: &str) -> Option<Self> {
        match value {
            "conforming" => Some(Self::Conforming),
            "candidate" => Some(Self::Candidate),
            "renderer-owned" => Some(Self::RendererOwned),
            "intentionally-divergent" => Some(Self::IntentionallyDivergent),
            "blocked" => Some(Self::Blocked),
            _ => None,
        }
    }
}

/// Renderer capability required by a component family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HeadlessComponentCapability {
    /// Semantic keyboard intent normalization.
    Keyboard,
    /// Semantic pointer intent normalization.
    Pointer,
    /// Shared focus state or renderer-owned physical focus integration for the declared scope.
    Focus,
    /// Accessibility semantic projection.
    Semantics,
    /// Dynamic collection replacement.
    DynamicCollection,
    /// Deterministic behavioral fixture evidence.
    Fixtures,
}

impl HeadlessComponentCapability {
    pub(super) const fn all() -> [Self; 6] {
        [
            Self::Keyboard,
            Self::Pointer,
            Self::Focus,
            Self::Semantics,
            Self::DynamicCollection,
            Self::Fixtures,
        ]
    }
}

/// Truthful capability support classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadlessCapabilitySupport {
    /// Capability is implemented and evidenced for the declared scope.
    Direct,
    /// Capability has a bounded fallback or incomplete path.
    Partial,
    /// Capability is unavailable with a named reason.
    Unsupported,
}

/// One renderer capability inventory cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadlessCapabilityEntry {
    /// Capability being classified.
    pub capability: HeadlessComponentCapability,
    /// Support classification.
    pub support: HeadlessCapabilitySupport,
    /// Named executable or checked evidence.
    pub evidence: Option<String>,
    /// Required reason for non-direct support.
    pub reason: Option<String>,
    /// Required next action for non-direct support.
    pub next_action: Option<String>,
}

/// One component-family and renderer inventory row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadlessComponentInventoryRow {
    /// Stable component-family identity.
    pub component_id: String,
    /// Stable renderer or shared-core identity.
    pub owner: String,
    /// Raw classification spelling validated by the inventory.
    pub classification: String,
    /// Reason for renderer ownership or intentional divergence.
    pub reason: Option<String>,
    /// Next action for candidate, blocked, or otherwise incomplete work.
    pub next_action: Option<String>,
    /// Bounded renderer capability cells.
    pub capabilities: Vec<HeadlessCapabilityEntry>,
}

/// Stable component inventory diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HeadlessInventoryIssueCode {
    /// Inventory contains too many rows or capability cells.
    InventoryTooLarge,
    /// Stable component or owner identity is malformed.
    InvalidIdentity,
    /// Component and owner pair occurs more than once.
    DuplicateRow,
    /// Classification spelling is unknown.
    UnknownClassification,
    /// Conforming row lacks complete direct evidence.
    ConformanceEvidenceMissing,
    /// Renderer ownership or divergence lacks a reason.
    OwnershipReasonMissing,
    /// Candidate, blocked, partial, or unsupported work lacks a next action.
    NextActionMissing,
    /// A capability occurs more than once.
    DuplicateCapability,
    /// Partial or unsupported capability lacks a reason.
    CapabilityReasonMissing,
}

/// One deterministic component inventory diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HeadlessInventoryIssue {
    /// Stable diagnostic code.
    pub code: HeadlessInventoryIssueCode,
    /// Stable component, owner, or capability subject.
    pub subject: String,
}
