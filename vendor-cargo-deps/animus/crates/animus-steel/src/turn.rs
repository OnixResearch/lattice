const SUMMARY_FIELD: FieldName = FieldName::new("summary");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FieldName {
    value: &'static str,
}

impl FieldName {
    const fn new(value: &'static str) -> Self {
        Self { value }
    }

    const fn as_str(self) -> &'static str {
        self.value
    }
}

/// Inputs made available to the Steel loop for one turn.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Input {
    summary: String,
    last_model_message: Option<String>,
    last_tool_output: Option<String>,
}

impl Input {
    /// Builds one loop turn from a required summary and optional prior outputs.
    ///
    /// # Errors
    ///
    /// Returns [`crate::LoopError`] when the summary is empty.
    pub fn new(
        summary: &str,
        last_model_message: Option<&str>,
        last_tool_output: Option<&str>,
    ) -> Result<Self, crate::LoopError> {
        validate_required_text(SUMMARY_FIELD, summary)?;
        Ok(Self {
            summary: String::from(summary),
            last_model_message: last_model_message.map(String::from),
            last_tool_output: last_tool_output.map(String::from),
        })
    }

    /// Builds one loop turn from deterministic core state.
    #[must_use]
    pub fn from_state(state: &animus_core::AnimusState) -> Self {
        Self {
            summary: state.summary(),
            last_model_message: None,
            last_tool_output: None,
        }
    }

    /// Returns the deterministic core summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.summary.as_str()
    }

    /// Returns the previous model message, if present.
    #[must_use]
    pub fn last_model_message(&self) -> Option<&str> {
        self.last_model_message.as_deref()
    }

    /// Returns the previous tool output, if present.
    #[must_use]
    pub fn last_tool_output(&self) -> Option<&str> {
        self.last_tool_output.as_deref()
    }

    pub(crate) fn bindings_program(&self) -> String {
        format!(
            "(set! animus-summary {})\n(set! animus-last-model-message {})\n(set! animus-last-tool-output {})\n",
            crate::quoting::scheme_string(self.summary()),
            option_binding(self.last_model_message()),
            option_binding(self.last_tool_output()),
        )
    }
}

fn option_binding(value: Option<&str>) -> String {
    value.map_or_else(|| String::from("#f"), crate::quoting::scheme_string)
}

fn validate_required_text(field: FieldName, value: &str) -> Result<(), crate::LoopError> {
    if value.trim().is_empty() {
        return Err(crate::LoopError::EmptyInput {
            field: field.as_str(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_SUMMARY: &str = "intent: build reliable systems";
    const MODEL_MESSAGE: &str = "model answer";
    const TOOL_OUTPUT: &str = "tool result";
    const BLANK_TEXT: &str = "   ";

    #[test]
    fn accepts_valid_inputs() {
        let input = Input::new(VALID_SUMMARY, Some(MODEL_MESSAGE), Some(TOOL_OUTPUT))
            .expect("valid input should parse");

        assert_eq!(input.summary(), VALID_SUMMARY);
        assert_eq!(input.last_model_message(), Some(MODEL_MESSAGE));
        assert_eq!(input.last_tool_output(), Some(TOOL_OUTPUT));
    }

    #[test]
    fn builds_from_core_state() {
        let intent =
            animus_core::Intent::new("build reliable systems").expect("valid intent should parse");
        let state = animus_core::AnimusState::new(intent);
        let input = Input::from_state(&state);

        assert_eq!(input.summary(), VALID_SUMMARY);
        assert_eq!(input.last_model_message(), None);
        assert_eq!(input.last_tool_output(), None);
    }

    #[test]
    fn rejects_empty_summary() {
        let error = Input::new(BLANK_TEXT, None, None).expect_err("blank summary should fail");

        assert!(matches!(error, crate::LoopError::EmptyInput { .. }));
    }
}
