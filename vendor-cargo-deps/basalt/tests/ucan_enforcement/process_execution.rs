use crate::process_support as process;
use crate::support::*;

// r[verify basalt.ucan_token_flow.process_execution]
#[test]
fn signed_execution_requires_explicit_policy_and_one_replay_commit() {
    let fixture = process::fixture(process::RESOURCE);
    let request = process::request();
    let signature = process::signature(&fixture, &request);
    let replay = RecordingReplay::new();
    let services = services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay);
    let live = process::verify(
        process::POLICY_JSON,
        &fixture,
        &request,
        &signature,
        &services,
    )
    .expect("explicit execution policy and exact signed grant");
    assert!(live.receipt().is_allowed());
    assert_eq!(live.holder_authorization().resource(), process::RESOURCE);
    assert_eq!(live.holder_authorization().ability(), "process/execute");
    assert_eq!(replay.calls.get(), 1);
    assert!(replay.committed.get());
    drop(live);
    assert_denied(
        process::verify(
            process::POLICY_JSON,
            &fixture,
            &request,
            &signature,
            &services,
        ),
        UcanEnforcementReason::ReplayDenied,
    );
}

// r[verify basalt.ucan_token_flow.process_policy]
#[test]
fn packaged_policy_does_not_enable_execution() {
    let fixture = process::fixture(process::RESOURCE);
    let request = process::request();
    let signature = process::signature(&fixture, &request);
    let replay = RecordingReplay::new();
    assert_denied(
        process::verify(
            basalt::DEFAULT_POLICY_JSON,
            &fixture,
            &request,
            &signature,
            &services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::PolicyDenied,
    );
    assert_eq!(replay.calls.get(), 0);
}

#[test]
fn document_write_capability_cannot_authorize_a_process() {
    let fixture = InvocationFixture::direct(document_capability(process::RESOURCE));
    let request = process::request();
    let signature = process::signature(&fixture, &request);
    let replay = RecordingReplay::new();
    assert_denied(
        process::verify(
            process::POLICY_JSON,
            &fixture,
            &request,
            &signature,
            &services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::ScopeDenied,
    );
    assert_eq!(replay.calls.get(), 0);
}

#[test]
fn changed_context_cannot_reuse_the_holder_signature() {
    let action = EnforcementAction::ProcessExecute;
    let fixture = process::fixture(action.resource_prefix());
    let request = process::request();
    let signature = process::signature(&fixture, &request);
    let changed = EnforcementRequest::new(
        action.contract_id(),
        format!("{}{}", action.resource_prefix(), "b".repeat(64)),
        action.ability(),
    );
    let replay = RecordingReplay::new();
    assert_denied(
        process::verify(
            process::POLICY_JSON,
            &fixture,
            &changed,
            &signature,
            &services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::HolderDenied,
    );
    assert_eq!(replay.calls.get(), 0);
    assert_ne!(
        bind_enforcement_action(action, &request).expect("original"),
        bind_enforcement_action(action, &changed).expect("changed")
    );
}

#[test]
fn revoked_execution_grant_never_commits_replay() {
    let fixture = process::fixture(process::RESOURCE);
    let request = process::request();
    let signature = process::signature(&fixture, &request);
    let replay = RecordingReplay::new();
    let revoked = RevocationList::from_references(vec![proof_reference(&fixture.token)]);
    assert_denied(
        process::verify(
            process::POLICY_JSON,
            &fixture,
            &request,
            &signature,
            &services(&fixture, &revoked, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::Revoked,
    );
    assert_eq!(replay.calls.get(), 0);
}

#[test]
fn binding_rejects_malformed_contexts_and_wrong_purpose() {
    let action = EnforcementAction::ProcessExecute;
    for suffix in [
        String::new(),
        "a".repeat(63),
        "a".repeat(65),
        "A".repeat(64),
        "g".repeat(64),
        "../a".into(),
        format!("{}?x", "a".repeat(64)),
    ] {
        let request = EnforcementRequest::new(
            action.contract_id(),
            format!("{}{suffix}", action.resource_prefix()),
            action.ability(),
        );
        assert_eq!(
            bind_enforcement_action(action, &request),
            Err(UcanEnforcementReason::ScopeDenied)
        );
    }
    let request = process::request();
    assert!(bind_enforcement_action(action, &request).is_ok());
    assert_eq!(
        bind_enforcement_action(EnforcementAction::DocumentWrite, &request),
        Err(UcanEnforcementReason::OperationMismatch)
    );
    assert_ne!(
        action.operation_profile_id(),
        EnforcementAction::DocumentWrite.operation_profile_id()
    );
    assert_ne!(
        action.operation_profile_id(),
        EnforcementAction::DocumentRead.operation_profile_id()
    );
}
