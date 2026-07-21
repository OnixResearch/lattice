impl RestoreReceipt {
    /// Validates a safe restore receipt and computes a deterministic BLAKE3 hash.
    ///
    /// # Errors
    ///
    /// Returns [`SessionRestoreError`] when forbidden receipt fields are present.
    pub fn validate(draft: &RestoreReceiptDraft<'_>) -> Result<Self, SessionRestoreError> {
        reject_receipt_leaks(draft)?;
        Ok(Self {
            plan: draft.plan.clone(),
            receipt_hash: restore_receipt_hash(draft.plan),
        })
    }

    /// Returns safe plan metadata.
    #[must_use]
    pub const fn plan(&self) -> &RestorePlan {
        &self.plan
    }

    /// Returns BLAKE3 receipt hash.
    #[must_use]
    pub const fn receipt_hash(&self) -> &crate::artifact::Hash {
        &self.receipt_hash
    }
}

fn reject_receipt_leaks(draft: &RestoreReceiptDraft<'_>) -> Result<(), SessionRestoreError> {
    if draft.raw_history.is_some() {
        return Err(SessionRestoreError::ReceiptLeak(
            RestoreReceiptLeak::RawHistory,
        ));
    }
    if draft.credential.is_some() {
        return Err(SessionRestoreError::ReceiptLeak(
            RestoreReceiptLeak::Credential,
        ));
    }
    if draft.shell_command.is_some() {
        return Err(SessionRestoreError::ReceiptLeak(
            RestoreReceiptLeak::ShellCommand,
        ));
    }
    if draft.full_path.is_some() {
        return Err(SessionRestoreError::ReceiptLeak(
            RestoreReceiptLeak::FullPath,
        ));
    }
    if draft.transcript.is_some() {
        return Err(SessionRestoreError::ReceiptLeak(
            RestoreReceiptLeak::Transcript,
        ));
    }
    Ok(())
}

fn restore_receipt_hash(plan: &RestorePlan) -> crate::artifact::Hash {
    let mut canonical = String::from(CANONICAL_VERSION_PREFIX);
    for target in plan.targets() {
        canonical.push_str(CANONICAL_SEPARATOR);
        canonical.push_str(target.target_label());
        canonical.push_str(CANONICAL_SEPARATOR);
        canonical.push_str(target.requested_mode().as_str());
        canonical.push_str(CANONICAL_SEPARATOR);
        canonical.push_str(target.status().as_str());
        canonical.push_str(CANONICAL_SEPARATOR);
        canonical.push_str(
            target
                .degraded_reason()
                .map_or("none", RestoreDegradedReason::as_str),
        );
        canonical.push_str(CANONICAL_SEPARATOR);
        canonical.push_str(&target.claims().process_survived().to_string());
        canonical.push_str(CANONICAL_SEPARATOR);
        canonical.push_str(&target.claims().layout_reconstructed().to_string());
        canonical.push_str(CANONICAL_SEPARATOR);
        canonical.push_str(&target.claims().display_replayed().to_string());
        canonical.push_str(CANONICAL_SEPARATOR);
        canonical.push_str(&target.claims().native_resume_requested().to_string());
        canonical.push_str(CANONICAL_SEPARATOR);
        canonical.push_str(&target.claims().handoff_requested().to_string());
        canonical.push_str(CANONICAL_SEPARATOR);
        canonical.push_str(
            target
                .native_reference_hash()
                .map_or("none", crate::artifact::Hash::as_str),
        );
    }
    let digest = blake3::hash(canonical.as_bytes());
    crate::artifact::Hash::from_bytes(digest.as_bytes())
}

/// Restore receipt leak class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RestoreReceiptLeak {
    /// Raw display history leaked.
    RawHistory,
    /// Credential-looking value leaked.
    Credential,
    /// Shell command leaked.
    ShellCommand,
    /// Full path leaked.
    FullPath,
    /// Transcript body leaked.
    Transcript,
}

/// Session restore validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionRestoreError {
    /// A safe text field failed adapter-boundary validation.
    Contract(crate::AdapterContractError),
    /// Too many restore targets were supplied.
    TooManyTargets {
        /// Maximum target count.
        max_items: usize,
        /// Actual target count.
        actual_items: usize,
    },
    /// Native session reference was invalid.
    InvalidNativeReference {
        /// Degraded reason.
        reason: RestoreDegradedReason,
    },
    /// History replay metadata was unsafe.
    UnsafeHistoryReplay,
    /// Receipt contained forbidden material.
    ReceiptLeak(RestoreReceiptLeak),
}

impl core::fmt::Display for SessionRestoreError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "{error}"),
            Self::TooManyTargets {
                max_items,
                actual_items,
            } => write!(
                formatter,
                "restore planning supports at most {max_items} targets; got {actual_items}"
            ),
            Self::InvalidNativeReference { reason } => {
                write!(formatter, "invalid native session reference: {reason:?}")
            }
            Self::UnsafeHistoryReplay => formatter.write_str("unsafe history replay metadata"),
            Self::ReceiptLeak(leak) => write!(formatter, "restore receipt leak: {leak:?}"),
        }
    }
}

impl From<crate::AdapterContractError> for SessionRestoreError {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

fn validate_item_count(actual_items: usize) -> Result<(), SessionRestoreError> {
    if actual_items > MAX_RESTORE_TARGETS {
        return Err(SessionRestoreError::TooManyTargets {
            max_items: MAX_RESTORE_TARGETS,
            actual_items,
        });
    }
    Ok(())
}

fn normalize(
    field: crate::validation::FieldName,
    value: &str,
) -> Result<String, SessionRestoreError> {
    crate::validation::normalized_adapter_field(field, value).map_err(SessionRestoreError::Contract)
}

