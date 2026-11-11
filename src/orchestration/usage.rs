//! Versioned command-worker usage observations.
//!
//! Reservations, ceilings, estimates, provider reports, host measurements, and
//! unknown observations remain distinct. This module does not price models or
//! schedule work.

use thiserror::Error;

pub const USAGE_OBSERVATION_SCHEMA: &str = "lattice.command-worker-usage.v1";
pub const MAX_IDENTIFIER_BYTES: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageProvenance {
    HostMeasured,
    ProviderReported,
    WorkerReported,
    Estimated,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageScope {
    Run,
    Node,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CommandWorkerUsageObservation {
    pub schema: String,
    pub scope: UsageScope,
    pub workflow_revision: String,
    pub instance_id: String,
    pub dispatch_id: String,
    pub run_id: u64,
    pub node_id: u64,
    pub invocation_id: String,
    pub worker_profile: String,
    pub attempt_id: String,
    pub agent_invocations: Option<u64>,
    pub model_requests: Option<u64>,
    pub subagent_calls: Option<u64>,
    pub tokens: Option<u64>,
    pub cost_microunits: Option<u64>,
    pub duration_millis: Option<u64>,
    pub provenance: UsageProvenance,
    pub complete: bool,
}

impl CommandWorkerUsageObservation {
    pub fn validate(&self) -> Result<(), UsageObservationError> {
        if self.schema != USAGE_OBSERVATION_SCHEMA {
            return Err(UsageObservationError::InvalidSchema);
        }
        for identifier in [
            &self.workflow_revision,
            &self.instance_id,
            &self.dispatch_id,
            &self.invocation_id,
            &self.worker_profile,
            &self.attempt_id,
        ] {
            validate_identifier(identifier)?;
        }
        let has_any = self.agent_invocations.is_some()
            || self.model_requests.is_some()
            || self.subagent_calls.is_some()
            || self.tokens.is_some()
            || self.cost_microunits.is_some()
            || self.duration_millis.is_some();
        if self.provenance == UsageProvenance::Unknown && (has_any || self.complete) {
            return Err(UsageObservationError::UnknownCarriesValues);
        }
        if self.complete && !has_any {
            return Err(UsageObservationError::EmptyCompleteObservation);
        }
        Ok(())
    }

    /// Only complete, non-estimated, non-unknown observations expose measured tokens.
    pub fn measured_tokens(&self) -> Option<u64> {
        if self.validate().is_ok() && self.complete && !self.is_estimated_or_unknown() {
            self.tokens
        } else {
            None
        }
    }

    /// Only complete, non-estimated, non-unknown observations expose measured cost.
    pub fn measured_cost_microunits(&self) -> Option<u64> {
        if self.validate().is_ok() && self.complete && !self.is_estimated_or_unknown() {
            self.cost_microunits
        } else {
            None
        }
    }

    pub fn is_estimated_or_unknown(&self) -> bool {
        matches!(self.provenance, UsageProvenance::Estimated | UsageProvenance::Unknown)
    }

    pub fn to_json(&self) -> Result<String, UsageObservationError> {
        self.validate()?;
        serde_json::to_string(self).map_err(|error| UsageObservationError::InvalidJson(error.to_string()))
    }

    pub fn from_json(text: &str) -> Result<Self, UsageObservationError> {
        let observation: Self =
            serde_json::from_str(text).map_err(|error| UsageObservationError::InvalidJson(error.to_string()))?;
        observation.validate()?;
        Ok(observation)
    }
}

fn validate_identifier(value: &str) -> Result<(), UsageObservationError> {
    if value.is_empty() || value.len() > MAX_IDENTIFIER_BYTES || value.chars().any(char::is_control) {
        return Err(UsageObservationError::InvalidIdentifier);
    }
    Ok(())
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum UsageObservationError {
    #[error("usage observation schema is not supported")]
    InvalidSchema,
    #[error("usage observation identifier is empty, unbounded, or contains control characters")]
    InvalidIdentifier,
    #[error("unknown usage cannot carry values or claim completeness")]
    UnknownCarriesValues,
    #[error("complete usage observation must carry at least one value")]
    EmptyCompleteObservation,
    #[error("usage observation JSON is malformed: {0}")]
    InvalidJson(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn observation() -> CommandWorkerUsageObservation {
        CommandWorkerUsageObservation {
            schema: USAGE_OBSERVATION_SCHEMA.to_string(),
            scope: UsageScope::Run,
            workflow_revision: "revision-1".to_string(),
            instance_id: "instance-1".to_string(),
            dispatch_id: "dispatch-1".to_string(),
            run_id: 7,
            node_id: 3,
            invocation_id: "invocation-1".to_string(),
            worker_profile: "profile-1".to_string(),
            attempt_id: "attempt-1".to_string(),
            agent_invocations: Some(1),
            model_requests: Some(2),
            subagent_calls: Some(1),
            tokens: Some(1_234),
            cost_microunits: Some(567),
            duration_millis: Some(42),
            provenance: UsageProvenance::ProviderReported,
            complete: true,
        }
    }

    #[test]
    fn complete_provider_observation_exposes_measured_values() {
        let observation = observation();
        assert_eq!(observation.validate(), Ok(()));
        assert_eq!(observation.measured_tokens(), Some(1_234));
        assert_eq!(observation.measured_cost_microunits(), Some(567));
    }

    #[test]
    fn unknown_and_estimated_observations_do_not_claim_measurements() {
        let mut unknown = observation();
        unknown.provenance = UsageProvenance::Unknown;
        unknown.complete = false;
        unknown.agent_invocations = None;
        unknown.model_requests = None;
        unknown.subagent_calls = None;
        unknown.tokens = None;
        unknown.cost_microunits = None;
        unknown.duration_millis = None;
        assert_eq!(unknown.validate(), Ok(()));
        assert_eq!(unknown.measured_tokens(), None);

        let mut estimated = observation();
        estimated.provenance = UsageProvenance::Estimated;
        assert_eq!(estimated.measured_tokens(), None);
        assert_eq!(estimated.measured_cost_microunits(), None);
    }

    #[test]
    fn invalid_shapes_are_rejected() {
        let mut unknown_with_values = observation();
        unknown_with_values.provenance = UsageProvenance::Unknown;
        assert_eq!(unknown_with_values.validate(), Err(UsageObservationError::UnknownCarriesValues));

        let mut empty = observation();
        empty.agent_invocations = None;
        empty.model_requests = None;
        empty.subagent_calls = None;
        empty.tokens = None;
        empty.cost_microunits = None;
        empty.duration_millis = None;
        assert_eq!(empty.validate(), Err(UsageObservationError::EmptyCompleteObservation));

        let mut identifier = observation();
        identifier.attempt_id.clear();
        assert_eq!(identifier.validate(), Err(UsageObservationError::InvalidIdentifier));
    }

    #[test]
    fn observation_json_round_trips_after_admission() {
        let observation = observation();
        let json = observation.to_json().expect("observation serializes");
        assert_eq!(CommandWorkerUsageObservation::from_json(&json), Ok(observation));
        assert!(CommandWorkerUsageObservation::from_json("{bad").is_err());
    }
}
