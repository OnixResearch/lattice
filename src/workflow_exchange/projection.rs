#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    non_trait_imports,
    reason = "the bounded projection preserves every producer observation and exact log-linkage field"
)]

use lattice_workflow_contract::ExecutionObservation;
use lattice_workflow_contract::LogReference;
use lattice_workflow_contract::LogStream;
use lattice_workflow_contract::NodeObservation;
use lattice_workflow_contract::OBSERVATION_SCHEMA;
use lattice_workflow_contract::observation_identity;

use super::DurableExecutionState;
use super::ExecutionRecord;
use super::TransitionError;

pub fn observation(record: &ExecutionRecord) -> Result<ExecutionObservation, TransitionError> {
    let nodes = record
        .nodes
        .iter()
        .map(|node| NodeObservation {
            node_id: node.request_node_id.clone(),
            state: node.state,
            terminal: node.state.is_terminal(),
            successful: node.state.is_successful(),
            adapter_identity: node.adapter_identity.clone(),
            receipt_identity: node.receipt_identity.clone(),
        })
        .collect::<Vec<_>>();
    let logs = record
        .logs
        .iter()
        .map(|log| {
            let bytes = u64::try_from(log.bytes.len()).map_err(|_| TransitionError::LogLimit)?;
            Ok(LogReference {
                node_id: log.node_id.clone(),
                stream: log.stream,
                reference_identity: log.reference_identity.clone(),
                cursor: log.cursor,
                bytes,
                truncated: false,
                gap_before: false,
                retained: true,
            })
        })
        .collect::<Result<Vec<_>, TransitionError>>()?;
    let mut candidate = ExecutionObservation {
        schema: OBSERVATION_SCHEMA.to_string(),
        identity: super::core::zero_identity()?,
        request_identity: record.accepted.request_identity.clone(),
        idempotency_key: record.accepted.idempotency_key.clone(),
        workflow_identity: record.accepted.workflow_identity.clone(),
        execution_identity: record.accepted.execution_identity.clone(),
        sequence: record.sequence,
        state: record.state.as_contract(),
        terminal: record.state.is_terminal(),
        successful: record.state == DurableExecutionState::Completed,
        nodes,
        logs,
        receipt_identity: record.receipt_identity.clone(),
    };
    candidate.identity = observation_identity(&candidate);
    Ok(candidate)
}

pub fn retained_log<'a>(
    record: &'a ExecutionRecord,
    node_id: &lattice_workflow_contract::Label,
    stream: LogStream,
    cursor: u64,
    maximum_bytes: u64,
) -> Result<&'a [u8], TransitionError> {
    let log = record
        .logs
        .iter()
        .find(|log| log.node_id == *node_id && log.stream == stream && log.cursor == cursor)
        .ok_or(TransitionError::LogLimit)?;
    let byte_count = u64::try_from(log.bytes.len()).map_err(|_| TransitionError::LogLimit)?;
    if byte_count > maximum_bytes {
        return Err(TransitionError::LogLimit);
    }
    Ok(&log.bytes)
}
