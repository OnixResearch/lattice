use crate::support::*;
use basalt::ucan;

struct Offline;

impl RevocationChecker for Offline {
    fn is_revoked(&self, reference: &ucan::ProofReference) -> Result<bool, ucan::RevocationError> {
        Err(ucan::RevocationError::Backend {
            reference: reference.clone(),
            message: "test revocation outage".to_owned(),
        })
    }
}

#[test]
fn unavailable_revocation_fails_without_another_replay_commit() {
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
    .expect("initial admission");
    let profile = Profile(EnforcementAction::DocumentWrite.operation_profile_id());
    let policies = ucan::HolderRecheckPolicies {
        caveats: &DenyUnknownCaveats,
        profiles: &profile,
    };
    assert!(matches!(
        live.observe_holder(
            fixture.token.as_str(),
            &context(&fixture, &Offline, VERIFY_AT_SECONDS + 1),
            &policies
        ),
        Err(ucan::HolderRecheckError::Verification(
            ucan::HolderInvocationError::Token { .. }
        ))
    ));
    assert_eq!(replay.calls.get(), FIRST_CALL);
}

struct Profile(OperationProfileId);

impl ucan::OperationProfileAdmission for Profile {
    fn admit_operation_profile(
        &self,
        profile: &OperationProfileId,
    ) -> Result<(), ucan::OperationProfileAdmissionError> {
        if profile == &self.0 {
            Ok(())
        } else {
            Err(ucan::OperationProfileAdmissionError::UnknownProfile)
        }
    }
}

fn context<'a>(
    fixture: &'a InvocationFixture,
    revocations: &'a dyn RevocationChecker,
    seconds: u64,
) -> ucan::HolderTokenVerificationContext<'a> {
    ucan::HolderTokenVerificationContext::new(
        VerificationTime::from_unix_seconds(seconds),
        &fixture.keys,
        &fixture.proofs,
        revocations,
        VerificationLimits::default(),
    )
}

#[test]
fn direct_and_delegated_observations_preserve_replay_and_receipt() {
    for fixture in [
        InvocationFixture::direct(document_capability(PARENT_RESOURCE)),
        InvocationFixture::delegated(),
    ] {
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
        .expect("initial admission");
        let receipt = live.receipt().clone();
        let profile = Profile(EnforcementAction::DocumentWrite.operation_profile_id());
        let policies = ucan::HolderRecheckPolicies {
            caveats: &DenyUnknownCaveats,
            profiles: &profile,
        };
        for seconds in [VERIFY_AT_SECONDS, VERIFY_AT_SECONDS + 1] {
            let observed = live
                .observe_holder(
                    fixture.token.as_str(),
                    &context(&fixture, &NoRevocations, seconds),
                    &policies,
                )
                .expect("current holder");
            assert_eq!(
                std::ptr::from_ref(observed.authorization()),
                std::ptr::from_ref(live.holder_authorization())
            );
            assert_eq!(observed.observed_at().as_unix_seconds(), seconds);
            assert_eq!(replay.calls.get(), FIRST_CALL);
            assert_eq!(live.receipt(), &receipt);
        }
        let denied = evaluate(
            fixture.token.as_str(),
            &request,
            &signature,
            nonce(),
            services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
        )
        .expect_err("nonce remains spent");
        assert_eq!(denied.reason(), UcanEnforcementReason::ReplayDenied);
    }
}

#[test]
fn substitution_and_time_regression_remain_typed_denials() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let other = InvocationFixture::direct_with_expiration(
        document_capability(PARENT_RESOURCE),
        TOKEN_EXPIRES_SECONDS + 1,
    );
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
    .expect("initial admission");
    let profile = Profile(EnforcementAction::DocumentWrite.operation_profile_id());
    let policies = ucan::HolderRecheckPolicies {
        caveats: &DenyUnknownCaveats,
        profiles: &profile,
    };
    assert!(matches!(
        live.observe_holder(
            other.token.as_str(),
            &context(&fixture, &NoRevocations, VERIFY_AT_SECONDS),
            &policies
        ),
        Err(ucan::HolderRecheckError::Binding(
            ucan::core::holder_invocation::recheck::Error::TokenMismatch
        ))
    ));
    assert!(matches!(
        live.observe_holder(
            fixture.token.as_str(),
            &context(&fixture, &NoRevocations, VERIFY_AT_SECONDS - 1),
            &policies
        ),
        Err(ucan::HolderRecheckError::Binding(
            ucan::core::holder_invocation::recheck::Error::TimeRegression
        ))
    ));
    assert_eq!(replay.calls.get(), FIRST_CALL);
}

#[test]
fn revoked_leaf_parent_and_expiry_deny_without_readmission() {
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
    .expect("initial admission");
    let profile = Profile(EnforcementAction::DocumentWrite.operation_profile_id());
    let policies = ucan::HolderRecheckPolicies {
        caveats: &DenyUnknownCaveats,
        profiles: &profile,
    };
    for reference in [
        proof_reference(&fixture.token),
        proof_reference(&fixture.proofs.as_slice()[0]),
    ] {
        let revoked = RevocationList::from_references(vec![reference]);
        assert!(matches!(
            live.observe_holder(
                fixture.token.as_str(),
                &context(&fixture, &revoked, VERIFY_AT_SECONDS + 1),
                &policies
            ),
            Err(ucan::HolderRecheckError::Verification(
                ucan::HolderInvocationError::Token { .. }
            ))
        ));
    }
    assert!(
        live.observe_holder(
            fixture.token.as_str(),
            &context(&fixture, &NoRevocations, EXPIRED_VERIFY_AT_SECONDS),
            &policies
        )
        .is_err()
    );
    assert_eq!(replay.calls.get(), FIRST_CALL);
}

#[test]
fn current_profile_and_caveat_observations_can_deny() {
    let fixture = InvocationFixture::direct(caveated_document_capability());
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &request, nonce());
    let replay = RecordingReplay::new();
    let allowed = CountingCaveatSet::new(true);
    let denied = CountingCaveatSet::new(false);
    let live = evaluate(
        fixture.token.as_str(),
        &request,
        &signature,
        nonce(),
        services(&fixture, &NoRevocations, &allowed, &replay),
    )
    .expect("initial admission");
    let profile = Profile(EnforcementAction::DocumentWrite.operation_profile_id());
    let unknown = Profile(EnforcementAction::DocumentRead.operation_profile_id());
    let verification = context(&fixture, &NoRevocations, VERIFY_AT_SECONDS + 1);
    assert!(
        live.observe_holder(
            fixture.token.as_str(),
            &verification,
            &ucan::HolderRecheckPolicies {
                caveats: &allowed,
                profiles: &profile
            }
        )
        .is_ok()
    );
    assert!(matches!(
        live.observe_holder(
            fixture.token.as_str(),
            &verification,
            &ucan::HolderRecheckPolicies {
                caveats: &denied,
                profiles: &profile
            }
        ),
        Err(ucan::HolderRecheckError::Verification(
            ucan::HolderInvocationError::Authorization { .. }
        ))
    ));
    assert!(matches!(
        live.observe_holder(
            fixture.token.as_str(),
            &verification,
            &ucan::HolderRecheckPolicies {
                caveats: &allowed,
                profiles: &unknown
            }
        ),
        Err(ucan::HolderRecheckError::Verification(
            ucan::HolderInvocationError::OperationProfile { .. }
        ))
    ));
    assert_eq!(replay.calls.get(), FIRST_CALL);
}
