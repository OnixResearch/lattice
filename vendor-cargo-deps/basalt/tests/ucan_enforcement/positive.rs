use crate::support::*;

#[test]
fn direct_holder_invocation_returns_live_exact_request_authority() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &request, nonce());
    let replay = RecordingReplay::new();

    let live = evaluate(
        fixture.token.as_str(),
        &request,
        &signature,
        nonce(),
        services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
    )
    .expect("valid direct holder invocation must pass");

    assert_eq!(std::ptr::from_ref(live.request()), std::ptr::from_ref(&request));
    assert!(live.receipt().is_allowed());
    assert_eq!(live.holder_authorization().resource(), DOCUMENT_RESOURCE);
    assert_eq!(replay.calls.get(), FIRST_CALL);
    assert!(replay.committed.get());
}

#[test]
fn delegated_holder_invocation_returns_live_authority() {
    let fixture = InvocationFixture::delegated();
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &request, nonce());
    let replay = RecordingReplay::new();

    let live = evaluate(
        fixture.token.as_str(),
        &request,
        &signature,
        nonce(),
        services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
    )
    .expect("valid delegated holder invocation must pass");

    assert!(live.receipt().is_allowed());
    assert_eq!(live.holder_authorization().ability(), DOCUMENT_WRITE_ABILITY);
    assert_eq!(replay.calls.get(), FIRST_CALL);
}
