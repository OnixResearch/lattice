use super::{
    ACTION_FIELD, CONTROL_CANCEL, CONTROL_EDIT, CONTROL_PAUSE, CONTROL_RESUME, CONTROL_RUN_NOW,
    ID_FIELD, ScheduleError, Text, parse_value, required_text,
};

/// Schedule control action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScheduleControlAction {
    /// Pause future runs.
    Pause,
    /// Resume future runs.
    Resume,
    /// Edit schedule metadata.
    Edit,
    /// Request immediate run.
    RunNow,
    /// Cancel queued or active run.
    Cancel,
}

impl ScheduleControlAction {
    /// Parses a stable schedule control action label.
    ///
    /// # Errors
    ///
    /// Returns [`ScheduleError`] when the label is empty or unsupported.
    pub fn parse(value: &str) -> Result<Self, ScheduleError> {
        parse_value(ACTION_FIELD, value, Self::from_normalized)
    }

    fn from_normalized(value: &str) -> Option<Self> {
        match value {
            CONTROL_PAUSE => Some(Self::Pause),
            CONTROL_RESUME => Some(Self::Resume),
            CONTROL_EDIT => Some(Self::Edit),
            CONTROL_RUN_NOW => Some(Self::RunNow),
            CONTROL_CANCEL => Some(Self::Cancel),
            _ => None,
        }
    }
}

/// Untrusted schedule control request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScheduleControlDraft<'a> {
    /// Schedule identity.
    pub schedule_id: &'a str,
    /// Action label.
    pub action: &'a str,
    /// Whether host policy admitted the control.
    pub host_admitted: bool,
    /// Whether confirmation was supplied.
    pub confirmed: bool,
}

/// Validated host-mediated schedule control request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduleControlRequest {
    schedule_id: Text,
    action: ScheduleControlAction,
}

impl ScheduleControlRequest {
    /// Validates a host-mediated schedule control request.
    ///
    /// # Errors
    ///
    /// Returns [`ScheduleError`] when admission or confirmation is missing.
    pub fn validate(draft: &ScheduleControlDraft<'_>) -> Result<Self, ScheduleError> {
        if !draft.host_admitted || !draft.confirmed {
            return Err(ScheduleError::ControlDenied);
        }
        let schedule_id = required_text(ID_FIELD, draft.schedule_id)?;
        let action = ScheduleControlAction::parse(draft.action)?;
        Ok(Self {
            schedule_id,
            action,
        })
    }

    /// Returns schedule identity.
    #[must_use]
    pub fn schedule_id(&self) -> &str {
        self.schedule_id.as_str()
    }

    /// Returns control action.
    #[must_use]
    pub const fn action(&self) -> ScheduleControlAction {
        self.action
    }
}
