#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "run evidence uses its local serialization and operation observation vocabulary"
)]
#![allow(
    path_segment_repetition,
    reason = "public run DTO names keep durable schema meaning explicit at external call sites"
)]

use serde::Deserialize;
use serde::Serialize;

pub const RUN_SCHEMA: &str = "lattice.operation-run.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunState {
    Started,
    Completed,
    Failed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunRecord {
    pub schema: String,
    pub invocation_identity: String,
    pub run_id: crate::engine::executor::RunId,
    pub state: RunState,
    pub started_epoch_millis: i64,
    pub finished_epoch_millis: Option<i64>,
    pub type_revision: String,
    pub instance_identity: String,
    pub method_identity: String,
    pub output_identity: Option<String>,
    pub output_bytes: usize,
    pub stderr_bytes: usize,
    pub diagnostic: String,
    pub redacted: bool,
    pub required_non_claims: Vec<String>,
}

impl RunRecord {
    pub fn started(
        plan: &super::model::InvocationPlan,
        run_id: crate::engine::executor::RunId,
        observed_epoch_millis: i64,
    ) -> Self {
        Self {
            schema: RUN_SCHEMA.to_string(),
            invocation_identity: plan.identity.clone(),
            run_id,
            state: RunState::Started,
            started_epoch_millis: observed_epoch_millis,
            finished_epoch_millis: None,
            type_revision: plan.type_revision.clone(),
            instance_identity: plan.instance_identity.clone(),
            method_identity: plan.method_identity.clone(),
            output_identity: None,
            output_bytes: 0,
            stderr_bytes: 0,
            diagnostic: "operation run started before effect dispatch".to_string(),
            redacted: true,
            required_non_claims: plan.required_non_claims.clone(),
        }
    }

    pub fn finished(&self, observation: &super::runtime::Observation, observed_epoch_millis: i64) -> Self {
        debug_assert_eq!(self.state, RunState::Started);
        let state = match observation.class {
            super::runtime::ObservationClass::Completed => RunState::Completed,
            super::runtime::ObservationClass::DeterministicFailure => RunState::Failed,
            super::runtime::ObservationClass::Unknown => RunState::Unknown,
        };
        Self {
            state,
            finished_epoch_millis: Some(observed_epoch_millis),
            output_identity: observation.output_identity.clone(),
            output_bytes: observation.output_bytes,
            stderr_bytes: observation.stderr_bytes,
            diagnostic: observation.diagnostic.clone(),
            ..self.clone()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DirectRunResult {
    pub record: RunRecord,
    pub output: Option<serde_json::Value>,
}
