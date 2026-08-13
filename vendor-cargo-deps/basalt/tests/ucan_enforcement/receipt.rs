use crate::support::*;

#[test]
fn identical_evaluations_have_identical_redacted_receipts() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &request, nonce());
    let first_replay = RecordingReplay::new();
    let second_replay = RecordingReplay::new();

    let first = evaluate(
        fixture.token.as_str(),
        &request,
        &signature,
        nonce(),
        services(&fixture, &NoRevocations, &DenyUnknownCaveats, &first_replay),
    )
    .expect("first evaluation must pass");
    let second = evaluate(
        fixture.token.as_str(),
        &request,
        &signature,
        nonce(),
        services(&fixture, &NoRevocations, &DenyUnknownCaveats, &second_replay),
    )
    .expect("second evaluation must pass");

    assert_eq!(first.receipt(), second.receipt());
    assert_eq!(first_replay.calls.get(), FIRST_CALL);
    assert_eq!(second_replay.calls.get(), FIRST_CALL);
}

#[test]
fn compact_bearer_fixtures_keep_zeroizing_ownership() {
    let sources = [
        include_str!("support/mod.rs"),
        include_str!("support/evaluate.rs"),
        include_str!("support/fixtures.rs"),
        include_str!("support/services.rs"),
        include_str!("positive.rs"),
        include_str!("negative/credentials.rs"),
        include_str!("negative/authority.rs"),
        include_str!("negative/substitution.rs"),
        include_str!("order.rs"),
        include_str!("receipt.rs"),
    ]
    .concat();
    let docs = include_str!("../../docs/ucan-enforcement.md");
    let forbidden = [
        ["token.as_str().as_bytes()", ".to_vec()"].concat(),
        ["token.as_str()", ".to_owned()"].concat(),
        ["token.as_str()", ".to_string()"].concat(),
        ["serde_json::to_string(&fixture.", "token"].concat(),
        [".arg(fixture.", "token.as_str())"].concat(),
        ["println!(fixture.", "token"].concat(),
        ["eprintln!(fixture.", "token"].concat(),
        ["dbg!(&fixture.", "token"].concat(),
        ["tampered_", "token"].concat(),
    ];

    for pattern in forbidden {
        assert!(!sources.contains(pattern.as_str()), "test source copies or exposes compact bearer text: {pattern}");
    }
    assert!(docs.contains("Pass only borrowed `CompactToken::as_str()` views"));
    assert!(docs.contains("Do not copy a compact bearer into `String` or `Vec` values"));
}

#[test]
fn persisted_receipt_cannot_recreate_live_authority() {
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
    .expect("valid invocation must pass");
    let persisted = serde_json::to_string(live.receipt()).expect("receipt must serialize");
    let restored: UcanEnforcementReceipt =
        serde_json::from_str(&persisted).expect("receipt must deserialize as evidence");
    let live_source = include_str!("../../src/ucan_enforcement/shell.rs");
    let error_source = include_str!("../../src/ucan_enforcement/error.rs");
    let struct_start = live_source.find("pub struct LiveUcanEnforcement").expect("live type source must exist");
    let struct_end = live_source[struct_start..]
        .find("impl fmt::Debug for LiveUcanEnforcement")
        .map(|offset| struct_start + offset)
        .expect("live debug implementation must follow its declaration");
    let struct_block = &live_source[struct_start..struct_end];
    let failure_start = error_source.find("pub struct UcanEnforcementFailure").expect("failure type source must exist");
    let failure_end = error_source[failure_start..]
        .find("impl UcanEnforcementFailure")
        .map(|offset| failure_start + offset)
        .expect("failure implementation must follow its declaration");
    let failure_block = &error_source[failure_start..failure_end];
    let live_impl = live_source
        .split("impl<'request> LiveUcanEnforcement")
        .nth(1)
        .expect("live implementation must exist")
        .split("struct ProspectiveAuthority")
        .next()
        .expect("live implementation must end before prospective authority");

    assert_eq!(&restored, live.receipt());
    assert!(!persisted.contains("did:key"));
    assert!(!persisted.contains(DOCUMENT_WRITE_ABILITY));
    assert!(!struct_block.lines().skip(1).any(|line| line.trim_start().starts_with("pub ")));
    assert!(!failure_block.contains("AuthorityDecision"));
    assert!(!failure_block.contains("LiveUcanEnforcement"));
    assert!(!live_impl.contains("pub fn new"));
    assert!(!live_impl.contains("authority_decision"));
    assert!(!live_source.contains("impl Clone for LiveUcanEnforcement"));
    assert!(!live_source.contains("impl serde::Serialize for LiveUcanEnforcement"));
    assert!(!live_source.contains("impl serde::Deserialize for LiveUcanEnforcement"));
}
