/// Tool-contract validation and planning errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractError {
    /// A common adapter field check failed.
    Adapter(crate::AdapterContractError),
    /// A required optional draft field was absent.
    MissingField {
        /// Boundary field name.
        field: &'static str,
    },
    /// A collection exceeded the tool contract bound.
    TooManyItems {
        /// Boundary field name.
        field: &'static str,
        /// Maximum accepted item count.
        max_items: usize,
        /// Actual item count.
        actual_items: usize,
    },
    /// A prepared argument collided with a previous argument.
    DuplicateArgument {
        /// Duplicate argument name.
        name: super::Text,
    },
    /// A prepared argument is not part of the current strict contract.
    UnknownArgument {
        /// Unknown argument name.
        name: super::Text,
    },
    /// Runtime tool calls no longer apply legacy argument compatibility shims.
    LegacyCompatibilityDisabled,
    /// Approval posture was not one of the stable current-shape labels.
    UnsupportedApprovalPosture {
        /// Rejected approval posture.
        posture: super::Text,
    },
    /// A safe output reference looked like a raw path, URL, or host locator.
    UnsafeReference {
        /// Rejected reference text.
        reference: super::Text,
    },
    /// The visible-output budget was outside the supported range.
    InvalidVisibleBudget {
        /// Smallest accepted budget.
        min_chars: u32,
        /// Largest accepted budget.
        max_chars: u32,
        /// Actual requested budget.
        actual_chars: u32,
    },
    /// Preserved bounded-output counts contradicted visible text metadata.
    InconsistentOutputMetadata,
    /// A bounded planning ceiling or count was outside the supported range.
    InvalidPlanningBound {
        /// Boundary field name.
        field: &'static str,
        /// Smallest accepted item count.
        min_items: usize,
        /// Largest accepted item count.
        max_items: usize,
        /// Actual item count.
        actual_items: usize,
    },
    /// A planning reference pointed at an unknown tool, step, or binding.
    UnknownPlanningReference {
        /// Boundary field name.
        field: &'static str,
        /// Rejected reference.
        reference: super::Text,
    },
    /// A planning reference was duplicated after normalization.
    DuplicatePlanningReference {
        /// Boundary field name.
        field: &'static str,
    },
    /// A planning enum-like text field used an unsupported label.
    UnsupportedPlanningValue {
        /// Boundary field name.
        field: &'static str,
        /// Rejected value.
        value: super::Text,
    },
    /// Safe receipt validation rejected generated tool-planning evidence.
    SafeReceiptMetadata(crate::SafeReceiptError),
}

impl From<crate::AdapterContractError> for ContractError {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Adapter(error)
    }
}
