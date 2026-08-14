mod common;

use std::collections::HashMap;

use lattice::engine::protocol::ActionRequest;
use lattice::engine::protocol::HttpAdapter;
use lattice::engine::protocol::LabelId;
use lattice::engine::protocol::create_session;
use lattice::engine::protocol::plan_action;
use lattice::engine::protocol::plan_session;
use lattice::engine::protocol::prepare_attempt;
use lattice::engine::protocol::shell::AdapterDispatch;
use lattice::engine::protocol::shell::ProtocolAdapter;

fn envelope() -> (lattice::engine::protocol::MessageEnvelope, lattice::engine::protocol::RolePlacement) {
    let cohort = common::protocol::cohort(common::protocol::native_source(common::protocol::transfer(
        common::protocol::ROLE_A,
        common::protocol::ROLE_B,
        common::protocol::LABEL_A,
        common::protocol::end(),
    )));
    let placements = common::protocol::native_placements(&cohort);
    let destination = placements[1].clone();
    let plan = match plan_session(cohort, placements, common::protocol::identity("transport-session")) {
        Ok(plan) => plan,
        Err(error) => panic!("plan failed: {error:?}"),
    };
    let session = match create_session(plan) {
        Ok(session) => session,
        Err(error) => panic!("session failed: {error:?}"),
    };
    let action = match plan_action(&session, ActionRequest {
        source: common::protocol::lattice_role(common::protocol::ROLE_A),
        label: LabelId(common::protocol::LABEL_A.0),
        payload_reference: common::protocol::identity("transport-payload"),
    }) {
        Ok(action) => action,
        Err(error) => panic!("action failed: {error:?}"),
    };
    let prepared = match prepare_attempt(&session, action) {
        Ok(prepared) => prepared,
        Err(error) => panic!("prepare failed: {error:?}"),
    };
    (prepared.attempts[0].messages[0].clone(), destination)
}

fn route(placement: &lattice::engine::protocol::RolePlacement) -> lattice::engine::protocol::Identity {
    match &placement.kind {
        lattice::engine::protocol::PlacementKind::Native { runtime_route } => runtime_route.clone(),
        lattice::engine::protocol::PlacementKind::External { .. } => panic!("native fixture became external"),
    }
}

// r[verify lattice.role_protocol.external_roles.denied]
#[test]
fn bounded_http_transport_accepts_success_and_rejects_oversized_envelope() {
    let (envelope, placement) = envelope();
    let endpoint = "https://protocol.invalid/receive";
    let client = lattice::http::StaticHttpClient::new(HashMap::from([(
        endpoint.to_owned(),
        lattice::http::StaticHttpResult::Response(lattice::http::HttpResponse {
            status: 202,
            headers: HashMap::new(),
            body: String::new(),
        }),
    )]));
    let transport =
        HttpAdapter::native(placement.adapter_cohort.clone(), route(&placement), endpoint.to_owned(), &client);
    assert_eq!(transport.dispatch(&envelope, &placement.limits), AdapterDispatch::Started);
    let tiny_limits = lattice::engine::protocol::AdapterLimits {
        max_envelope_bytes: 1,
        timeout_ms: placement.limits.timeout_ms,
    };
    assert_eq!(transport.dispatch(&envelope, &tiny_limits), AdapterDispatch::RejectedBeforeStart);
}

#[test]
fn transport_error_remains_unknown_after_start_and_invalid_url_is_rejected_before_start() {
    let (envelope, placement) = envelope();
    let endpoint = "https://protocol.invalid/receive";
    let client = lattice::http::StaticHttpClient::new(HashMap::from([(
        endpoint.to_owned(),
        lattice::http::StaticHttpResult::Error(lattice::http::HttpError::Timeout("bounded timeout".to_owned())),
    )]));
    let timeout =
        HttpAdapter::native(placement.adapter_cohort.clone(), route(&placement), endpoint.to_owned(), &client);
    assert_eq!(timeout.dispatch(&envelope, &placement.limits), AdapterDispatch::UnknownAfterStart);
    let invalid = HttpAdapter::native(
        placement.adapter_cohort.clone(),
        route(&placement),
        "file:///ambient/path".to_owned(),
        &client,
    );
    assert_eq!(invalid.dispatch(&envelope, &placement.limits), AdapterDispatch::RejectedBeforeStart);
}
