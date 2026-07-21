fn add_section_usage(usages: &mut Vec<SectionUsage>, section: PromptSectionKind, units: u32) {
    if let Some(usage) = usages.iter_mut().find(|usage| usage.section == section) {
        usage.used_units = usage.used_units.saturating_add(units);
        return;
    }
    usages.push(SectionUsage {
        section,
        used_units: units,
    });
}

/// Borrowed prompt assembly receipt fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PromptAssemblyReceiptDraft<'a> {
    /// Safe source label.
    pub source_label: &'a str,
    /// Source kind.
    pub source_kind: PromptSourceKind,
    /// Trust status.
    pub trust_status: PromptSourceTrustStatus,
    /// Prompt section.
    pub section: PromptSectionKind,
    /// Consumed budget units.
    pub consumed_units: u32,
    /// Whether the source was truncated.
    pub truncated: bool,
    /// Optional safe conflict reason label.
    pub conflict_reason: Option<&'a str>,
    /// Lowercase BLAKE3 policy hash.
    pub policy_hash: &'a str,
    /// Lowercase BLAKE3 body hash.
    pub body_hash: &'a str,
    /// Raw prompt body, which is forbidden in receipts.
    pub raw_body: Option<&'a str>,
    /// Credential-looking value, which is forbidden in receipts.
    pub credential: Option<&'a str>,
    /// Full host path, which is forbidden in receipts.
    pub full_path: Option<&'a str>,
    /// Provider payload, which is forbidden in receipts.
    pub provider_payload: Option<&'a str>,
    /// Hidden memory body, which is forbidden in receipts.
    pub hidden_memory_body: Option<&'a str>,
}

/// Safe prompt assembly receipt metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptAssemblyReceipt {
    source_label: String,
    source_kind: PromptSourceKind,
    trust_status: PromptSourceTrustStatus,
    section: PromptSectionKind,
    consumed_units: u32,
    truncated: bool,
    conflict_reason: Option<String>,
    policy_hash: crate::artifact::Hash,
    body_hash: crate::artifact::Hash,
}

impl PromptAssemblyReceipt {
    /// Validates safe prompt assembly receipt metadata.
    ///
    /// # Errors
    ///
    /// Returns [`PromptAssemblyError`] when unsafe receipt fields are present or
    /// safe labels/hashes fail validation.
    // r[impl agent_harness_runtime.prompt_assembly.receipts]
    pub fn validate(draft: &PromptAssemblyReceiptDraft<'_>) -> Result<Self, PromptAssemblyError> {
        reject_receipt_leaks(draft)?;
        Ok(Self {
            source_label: normalize(SOURCE_LABEL_FIELD, draft.source_label)?,
            source_kind: draft.source_kind,
            trust_status: draft.trust_status,
            section: draft.section,
            consumed_units: draft.consumed_units,
            truncated: draft.truncated,
            conflict_reason: normalize_optional(CONFLICT_REASON_FIELD, draft.conflict_reason)?,
            policy_hash: crate::artifact::Hash::new(draft.policy_hash)
                .map_err(PromptAssemblyError::InvalidHash)?,
            body_hash: crate::artifact::Hash::new(draft.body_hash)
                .map_err(PromptAssemblyError::InvalidHash)?,
        })
    }

    /// Returns safe source label.
    #[must_use]
    pub fn source_label(&self) -> &str {
        self.source_label.as_str()
    }

    /// Returns source kind.
    #[must_use]
    pub const fn source_kind(&self) -> PromptSourceKind {
        self.source_kind
    }

    /// Returns trust status.
    #[must_use]
    pub const fn trust_status(&self) -> PromptSourceTrustStatus {
        self.trust_status
    }

    /// Returns prompt section.
    #[must_use]
    pub const fn section(&self) -> PromptSectionKind {
        self.section
    }

    /// Returns consumed units.
    #[must_use]
    pub const fn consumed_units(&self) -> u32 {
        self.consumed_units
    }

    /// Returns true when source content was truncated.
    #[must_use]
    pub const fn truncated(&self) -> bool {
        self.truncated
    }

    /// Returns optional safe conflict reason.
    #[must_use]
    pub fn conflict_reason(&self) -> Option<&str> {
        self.conflict_reason.as_deref()
    }

    /// Returns policy hash.
    #[must_use]
    pub const fn policy_hash(&self) -> &crate::artifact::Hash {
        &self.policy_hash
    }

    /// Returns body hash.
    #[must_use]
    pub const fn body_hash(&self) -> &crate::artifact::Hash {
        &self.body_hash
    }
}

fn reject_receipt_leaks(draft: &PromptAssemblyReceiptDraft<'_>) -> Result<(), PromptAssemblyError> {
    if draft.raw_body.is_some() {
        return Err(PromptAssemblyError::ReceiptLeak {
            leak: PromptAssemblyReceiptLeak::RawPromptBody,
        });
    }
    if draft.credential.is_some() {
        return Err(PromptAssemblyError::ReceiptLeak {
            leak: PromptAssemblyReceiptLeak::Credential,
        });
    }
    if draft.full_path.is_some() {
        return Err(PromptAssemblyError::ReceiptLeak {
            leak: PromptAssemblyReceiptLeak::FullPath,
        });
    }
    if draft.provider_payload.is_some() {
        return Err(PromptAssemblyError::ReceiptLeak {
            leak: PromptAssemblyReceiptLeak::ProviderPayload,
        });
    }
    if draft.hidden_memory_body.is_some() {
        return Err(PromptAssemblyError::ReceiptLeak {
            leak: PromptAssemblyReceiptLeak::HiddenMemoryBody,
        });
    }
    Ok(())
}

/// Prompt assembly receipt leak class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromptAssemblyReceiptLeak {
    /// Raw prompt body was present.
    RawPromptBody,
    /// Credential-looking value was present.
    Credential,
    /// Full host path was present.
    FullPath,
    /// Provider payload was present.
    ProviderPayload,
    /// Hidden memory body was present.
    HiddenMemoryBody,
}

/// Prompt assembly validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PromptAssemblyError {
    /// A text field failed adapter-boundary validation.
    Contract(crate::AdapterContractError),
    /// A BLAKE3 content hash was invalid.
    InvalidHash(crate::artifact::Error),
    /// A bounded item list exceeded the allowed item count.
    TooManyItems {
        /// Field containing too many items.
        field: &'static str,
        /// Maximum supported items.
        max_items: usize,
        /// Actual item count.
        actual_items: usize,
    },
    /// Section order was empty.
    EmptySectionOrder,
    /// Section budget was zero.
    ZeroSectionBudget,
    /// Source budget was zero.
    ZeroBudgetUnits {
        /// Source label associated with the invalid budget.
        source_label: String,
    },
    /// Receipt contained forbidden sensitive material.
    ReceiptLeak {
        /// Leak class.
        leak: PromptAssemblyReceiptLeak,
    },
}

impl core::fmt::Display for PromptAssemblyError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "{error}"),
            Self::InvalidHash(error) => write!(formatter, "{error}"),
            Self::TooManyItems {
                field,
                max_items,
                actual_items,
            } => write!(
                formatter,
                "{field} supports at most {max_items} items; got {actual_items}"
            ),
            Self::EmptySectionOrder => {
                formatter.write_str("prompt section order must not be empty")
            }
            Self::ZeroSectionBudget => {
                formatter.write_str("prompt section budget must be non-zero")
            }
            Self::ZeroBudgetUnits { source_label } => {
                write!(formatter, "{source_label} must use non-zero budget units")
            }
            Self::ReceiptLeak { leak } => {
                write!(formatter, "prompt assembly receipt leak: {leak:?}")
            }
        }
    }
}

impl From<crate::AdapterContractError> for PromptAssemblyError {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}
