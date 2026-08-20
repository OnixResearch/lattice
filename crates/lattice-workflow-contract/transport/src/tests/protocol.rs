use super::*;

// r[verify lattice.workflow_interchange.transport_response.scenario.log]
#[test]
fn linked_log_bytes_are_returned_with_the_admitted_observation() {
    let profile = contract_profile();
    let request = workflow_request(&profile);
    let accepted = accepted(&request);
    let node_id = request.graph.nodes.first().expect("test node exists").id.clone();
    let log_read = LogReadRequest {
        schema: LOG_READ_SCHEMA.to_string(),
        request_identity: accepted.request_identity.clone(),
        workflow_identity: accepted.workflow_identity.clone(),
        execution_identity: accepted.execution_identity.clone(),
        node_id,
        stream: LogStream::Stdout,
        cursor: 0,
        maximum_bytes: TEST_MAXIMUM_LOG_READ_BYTES,
    };
    let exchange = new_request(ExchangeCommand::ReadLog {
        profile,
        request,
        accepted,
        log_read,
    })
    .expect("log request is encodable");
    let mut service = WorkflowExchangeService::new(FixtureHandler { calls: 0 });
    let response = service.exchange(&exchange).expect("log response is encodable");

    assert!(matches!(
        response.result,
        ExchangeResult::Success(success)
            if matches!(success.as_ref(), ExchangeSuccess::Log { bytes, .. } if bytes == TEST_LOG_BYTES)
    ));
}

// r[verify lattice.workflow_interchange.transport_response.scenario.oversized]
#[test]
fn log_bytes_must_match_the_admitted_reference() {
    struct MismatchedHandler;

    impl WorkflowExchangeHandler for MismatchedHandler {
        fn handle(&mut self, command: &ExchangeCommand) -> Result<ExchangeSuccess, HandlerError> {
            let ExchangeCommand::ReadLog { request, accepted, .. } = command else {
                return Err(HandlerError::Rejected);
            };
            let mut bytes = TEST_LOG_BYTES.to_vec();
            bytes.push(0);
            Ok(ExchangeSuccess::Log {
                observation: observation(request, accepted),
                bytes,
            })
        }
    }

    let profile = contract_profile();
    let request = workflow_request(&profile);
    let accepted = accepted(&request);
    let node_id = request.graph.nodes.first().expect("test node exists").id.clone();
    let log_read = LogReadRequest {
        schema: LOG_READ_SCHEMA.to_string(),
        request_identity: accepted.request_identity.clone(),
        workflow_identity: accepted.workflow_identity.clone(),
        execution_identity: accepted.execution_identity.clone(),
        node_id,
        stream: LogStream::Stdout,
        cursor: 0,
        maximum_bytes: TEST_MAXIMUM_LOG_READ_BYTES,
    };
    let exchange = new_request(ExchangeCommand::ReadLog {
        profile,
        request,
        accepted,
        log_read,
    })
    .expect("log request is encodable");
    let mut service = WorkflowExchangeService::new(MismatchedHandler);
    let response = service.exchange(&exchange).expect("safe rejection is encodable");

    assert!(matches!(
        response.result,
        ExchangeResult::Rejected(Rejection {
            code: RejectionCode::InvalidOperationResult,
            ..
        })
    ));
}
