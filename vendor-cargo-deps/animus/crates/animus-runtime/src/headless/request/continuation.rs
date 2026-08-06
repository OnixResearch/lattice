/// Borrowed resume identity before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ThreadResumeDraft<'a> {
    /// Thread identity to resume.
    pub thread_id: &'a str,
    /// Ledger hash expected by the controller.
    pub ledger_hash: &'a str,
    /// Optional guard hash that must equal `ledger_hash`.
    pub expected_ledger_hash: Option<&'a str>,
}

/// Validated resume identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThreadResume {
    thread_id: super::super::Text,
    ledger_hash: super::super::Text,
}

impl ThreadResume {
    /// Validates a resume identity before any runtime action continues.
    ///
    /// # Errors
    ///
    /// Returns [`super::super::AutomationError`] when thread or ledger identity
    /// fields are empty, or when the supplied ledger hash does not match its
    /// guard hash.
    pub fn validate(draft: &ThreadResumeDraft<'_>) -> Result<Self, super::super::AutomationError> {
        let thread_id = super::super::required_text(super::super::TextField {
            field: super::super::THREAD_ID_FIELD,
            value: draft.thread_id,
        })?;
        let ledger_hash = super::super::required_text(super::super::TextField {
            field: super::super::LEDGER_HASH_FIELD,
            value: draft.ledger_hash,
        })?;
        if let Some(expected) = draft.expected_ledger_hash {
            let expected_hash = super::super::required_text(super::super::TextField {
                field: super::super::LEDGER_HASH_FIELD,
                value: expected,
            })?;
            if ledger_hash != expected_hash {
                return Err(super::super::AutomationError::ResumeMismatch {
                    thread_id,
                    expected_ledger_hash: expected_hash,
                    actual_ledger_hash: ledger_hash,
                });
            }
        }
        Ok(Self { thread_id, ledger_hash })
    }

    /// Returns the thread identity.
    #[must_use]
    pub fn thread_id(&self) -> &str {
        self.thread_id.as_str()
    }

    /// Returns the ledger hash.
    #[must_use]
    pub fn ledger_hash(&self) -> &str {
        self.ledger_hash.as_str()
    }
}
