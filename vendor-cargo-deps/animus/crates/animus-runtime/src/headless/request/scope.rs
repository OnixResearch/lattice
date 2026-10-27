/// Session and/or workspace identity for a headless run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunIdentity {
    session_id: Option<super::super::Text>,
    workspace_id: Option<super::super::Text>,
}

impl RunIdentity {
    /// Validates that at least one stable host identity is present.
    ///
    /// # Errors
    ///
    /// Returns [`super::super::AutomationError`] when both session and workspace
    /// identities are empty.
    pub fn validate(
        session_id: &str,
        workspace_id: &str,
    ) -> Result<Self, super::super::AutomationError> {
        let session_id = super::super::optional_text(super::super::OptionalTextField {
            field: super::super::SESSION_ID_FIELD,
            value: Some(session_id),
        });
        let workspace_id = super::super::optional_text(super::super::OptionalTextField {
            field: super::super::WORKSPACE_ID_FIELD,
            value: Some(workspace_id),
        });
        if session_id.is_none() && workspace_id.is_none() {
            return Err(super::super::AutomationError::EmptyField {
                field: super::super::RUN_IDENTITY_FIELD,
            });
        }
        Ok(Self {
            session_id,
            workspace_id,
        })
    }

    /// Returns the optional session identity.
    #[must_use]
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Returns the optional workspace identity.
    #[must_use]
    pub fn workspace_id(&self) -> Option<&str> {
        self.workspace_id.as_deref()
    }
}
