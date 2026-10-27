use super::support::*;

#[test]
fn missing_credential_blocks_before_side_effects() {
    let request = request_with(None, Some(HASH_B)).expect("request should validate");

    let decision = super::super::admit(&request).expect("decision should evaluate");

    assert_eq!(decision.verdict(), super::super::AdmissionVerdict::Deny);
    assert_eq!(
        decision.reason(),
        Some(super::super::AdmissionReason::MissingCredential)
    );
    assert!(!decision.is_allowed());
}

#[test]
fn wrong_audience_fails_closed() {
    let reason = denied_reason(
        Some(credential(WRONG_AUDIENCE_CREDENTIAL)),
        Some(HASH_B),
        super::super::AdmissionCaveat::Allow,
    );

    assert_eq!(reason, Some(super::super::AdmissionReason::WrongAudience));
}

#[test]
fn expired_credential_fails_closed() {
    let reason = denied_reason(
        Some(super::super::CredentialEvidenceDraft {
            expiry: super::super::CredentialExpiry::Expired,
            ..credential(VALID_CREDENTIAL)
        }),
        Some(HASH_B),
        super::super::AdmissionCaveat::Allow,
    );

    assert_eq!(
        reason,
        Some(super::super::AdmissionReason::ExpiredCredential)
    );
}

#[test]
fn revoked_credential_fails_closed() {
    let reason = denied_reason(
        Some(super::super::CredentialEvidenceDraft {
            verifier_status: super::super::CredentialVerifierStatus::Revoked,
            ..credential(VALID_CREDENTIAL)
        }),
        Some(HASH_B),
        super::super::AdmissionCaveat::Allow,
    );

    assert_eq!(
        reason,
        Some(super::super::AdmissionReason::RevokedCredential)
    );
}

#[test]
fn missing_grant_fails_closed() {
    let reason = denied_reason(
        Some(credential(OTHER_RESOURCE_CREDENTIAL)),
        Some(HASH_B),
        super::super::AdmissionCaveat::Allow,
    );

    assert_eq!(reason, Some(super::super::AdmissionReason::MissingGrant));
}

#[test]
fn caveat_denial_fails_closed() {
    let reason = denied_reason(
        Some(credential(VALID_CREDENTIAL)),
        Some(HASH_B),
        super::super::AdmissionCaveat::Deny,
    );

    assert_eq!(reason, Some(super::super::AdmissionReason::CaveatDenied));
}

#[test]
fn verifier_unavailable_fails_closed() {
    let reason = denied_reason(
        Some(super::super::CredentialEvidenceDraft {
            verifier_status: super::super::CredentialVerifierStatus::Unavailable,
            ..credential(VALID_CREDENTIAL)
        }),
        Some(HASH_B),
        super::super::AdmissionCaveat::Allow,
    );

    assert_eq!(
        reason,
        Some(super::super::AdmissionReason::VerifierUnavailable)
    );
}

#[test]
fn missing_policy_fails_closed() {
    let reason = denied_reason(
        Some(credential(VALID_CREDENTIAL)),
        None,
        super::super::AdmissionCaveat::Allow,
    );

    assert_eq!(reason, Some(super::super::AdmissionReason::MissingPolicy));
}

#[test]
fn unsupported_entrypoint_fails_closed() {
    let request = super::super::AdmissionRequest::new(&super::super::AdmissionRequestDraft {
        entrypoint: super::super::Entrypoint::Unsupported,
        ..draft(
            Some(credential(VALID_CREDENTIAL)),
            Some(HASH_B),
            super::super::AdmissionCaveat::Allow,
        )
    })
    .expect("request should validate");

    let reason = super::super::admit(&request)
        .expect("decision should evaluate")
        .reason();

    assert_eq!(
        reason,
        Some(super::super::AdmissionReason::UnsupportedEntrypoint)
    );
}

#[test]
fn receipt_leak_fixtures_are_rejected() {
    for marker in super::super::LEAK_MARKERS {
        assert_eq!(
            super::super::validate_safe_text(marker),
            Err(super::super::AdmissionError::ReceiptLeak { marker })
        );
    }
}

#[test]
fn plain_http_remote_server_fails_closed_when_tls_required() {
    // r[verify agent_harness_runtime.goose_remote_server_admission.fail_closed]
    let reason = remote_denial_reason(RemoteDenialFixture {
        transport: super::super::RemoteServerTransportKind::Http,
        handshake: Some(RemoteHandshakeFixture {
            tls: super::super::RemoteServerObservedTls::Disabled,
            ..RemoteHandshakeFixture::pinned()
        }),
        ..RemoteDenialFixture::default()
    });

    assert_eq!(
        reason,
        Some(super::super::RemoteServerAdmissionReason::MissingTls)
    );
}

#[test]
fn missing_remote_server_handshake_fails_closed_without_probe_effects() {
    // r[verify agent_harness_runtime.goose_remote_server_admission.handshake.host_owned]
    let descriptor = remote_descriptor().expect("descriptor should validate");
    let policy = remote_policy().expect("policy should validate");
    let decision = remote_decision(&descriptor, None, &policy).expect("admission should evaluate");

    assert_eq!(
        decision.reason(),
        Some(super::super::RemoteServerAdmissionReason::MissingHandshake)
    );
    assert!(!decision.is_admitted());
    assert_eq!(
        decision.receipt().tls_status(),
        super::super::RemoteServerObservedTls::NotObserved
    );
}

#[test]
fn remote_server_fingerprint_mismatch_fails_closed() {
    let reason = remote_denial_reason(RemoteDenialFixture {
        handshake: Some(RemoteHandshakeFixture {
            fingerprint_status: super::super::RemoteServerFingerprintStatus::Mismatched,
            ..RemoteHandshakeFixture::pinned()
        }),
        ..RemoteDenialFixture::default()
    });

    assert_eq!(
        reason,
        Some(super::super::RemoteServerAdmissionReason::FingerprintMismatch)
    );
}

#[test]
fn remote_server_tofu_pending_without_approval_fails_closed() {
    let reason = remote_denial_reason(RemoteDenialFixture {
        fingerprint_policy: super::super::RemoteServerFingerprintPolicy::TrustOnFirstUsePending,
        handshake: Some(RemoteHandshakeFixture {
            fingerprint_status: super::super::RemoteServerFingerprintStatus::TofuPending,
            ..RemoteHandshakeFixture::pinned()
        }),
        ..RemoteDenialFixture::default()
    });

    assert_eq!(
        reason,
        Some(super::super::RemoteServerAdmissionReason::TofuPending)
    );
}

#[test]
fn remote_server_secret_unavailable_fails_closed() {
    let reason = remote_denial_reason(RemoteDenialFixture {
        handshake: Some(RemoteHandshakeFixture {
            auth_outcome: super::super::RemoteServerAuthOutcome::SecretUnavailable,
            ..RemoteHandshakeFixture::pinned()
        }),
        ..RemoteDenialFixture::default()
    });

    assert_eq!(
        reason,
        Some(super::super::RemoteServerAdmissionReason::SecretUnavailable)
    );
}

#[test]
fn remote_server_unauthorized_fails_closed() {
    let reason = remote_denial_reason(RemoteDenialFixture {
        handshake: Some(RemoteHandshakeFixture {
            auth_outcome: super::super::RemoteServerAuthOutcome::Unauthorized,
            ..RemoteHandshakeFixture::pinned()
        }),
        ..RemoteDenialFixture::default()
    });

    assert_eq!(
        reason,
        Some(super::super::RemoteServerAdmissionReason::Unauthorized)
    );
}

#[test]
fn remote_server_unsupported_protocol_fails_closed() {
    let reason = remote_denial_reason(RemoteDenialFixture {
        handshake: Some(RemoteHandshakeFixture {
            protocol_status: super::super::RemoteServerProtocolVersionStatus::Unsupported,
            ..RemoteHandshakeFixture::pinned()
        }),
        ..RemoteDenialFixture::default()
    });

    assert_eq!(
        reason,
        Some(super::super::RemoteServerAdmissionReason::UnsupportedProtocol)
    );
}

#[test]
fn remote_server_wrong_audience_fails_closed() {
    let reason = remote_denial_reason(RemoteDenialFixture {
        handshake: Some(RemoteHandshakeFixture {
            audience_hash: HASH_B,
            ..RemoteHandshakeFixture::pinned()
        }),
        ..RemoteDenialFixture::default()
    });

    assert_eq!(
        reason,
        Some(super::super::RemoteServerAdmissionReason::AudienceMismatch)
    );
}

#[test]
fn remote_server_capability_widening_fails_closed() {
    let reason = remote_denial_reason(RemoteDenialFixture {
        policy: RemotePolicyFixture {
            requested_capabilities: &[CAPABILITY_PROMPT, CAPABILITY_ADMIN],
            ..RemotePolicyFixture::default()
        },
        ..RemoteDenialFixture::default()
    });

    assert_eq!(
        reason,
        Some(super::super::RemoteServerAdmissionReason::CapabilityWidening)
    );
}

#[test]
fn remote_server_fallback_request_fails_closed() {
    let reason = remote_denial_reason(RemoteDenialFixture {
        policy: RemotePolicyFixture {
            fallback_requested: true,
            ..RemotePolicyFixture::default()
        },
        ..RemoteDenialFixture::default()
    });

    assert_eq!(
        reason,
        Some(super::super::RemoteServerAdmissionReason::FallbackDenied)
    );
}

#[test]
fn remote_server_receipt_leak_fixtures_are_rejected() {
    // r[verify agent_harness_runtime.goose_remote_server_admission.receipts.rejects_leaks]
    for marker in super::super::REMOTE_SERVER_RECEIPT_LEAK_MARKERS {
        assert_eq!(
            super::super::validate_remote_server_admission_receipt_text(marker),
            Err(super::super::RemoteServerAdmissionError::UnsafeMaterial { marker })
        );
    }
}

#[test]
fn raw_endpoint_descriptor_is_rejected() {
    let result =
        super::super::RemoteServerDescriptor::new(&super::super::RemoteServerDescriptorDraft {
            backend_identity: RAW_ENDPOINT,
            endpoint_identity_hash: HASH_C,
            endpoint_identity_class: super::super::RemoteServerEndpointIdentityClass::HashOnly,
            transport: super::super::RemoteServerTransportKind::Https,
            protocol_role: super::super::RemoteServerProtocolRole::HeadlessControl,
            platform_role: super::super::RemoteServerPlatformRole::GooseServeBackend,
            tls_requirement: super::super::RemoteServerTlsRequirement::Required,
            fingerprint_policy: super::super::RemoteServerFingerprintPolicy::Pinned,
            secret_source_label: SECRET_SOURCE,
            secret_source_class: super::super::RemoteServerSecretSourceClass::SharedSecret,
            scope: SCOPE,
            capability_ceilings: &[CAPABILITY_PROMPT],
            expected_audience_hash: HASH_A,
            provenance_hash: HASH_D,
        });

    assert!(result.is_err());
    assert_eq!(
        result,
        Err(super::super::RemoteServerAdmissionError::UnsafeMaterial { marker: "https://" })
    );
}

#[derive(Clone, Copy)]
struct RemoteDenialFixture<'a> {
    transport: super::super::RemoteServerTransportKind,
    fingerprint_policy: super::super::RemoteServerFingerprintPolicy,
    protocol_role: super::super::RemoteServerProtocolRole,
    handshake: Option<RemoteHandshakeFixture<'a>>,
    policy: RemotePolicyFixture<'a>,
    expected_audience_hash: &'a str,
}

impl RemoteDenialFixture<'_> {
    const fn default() -> Self {
        Self {
            transport: super::super::RemoteServerTransportKind::Https,
            fingerprint_policy: super::super::RemoteServerFingerprintPolicy::Pinned,
            protocol_role: super::super::RemoteServerProtocolRole::HeadlessControl,
            handshake: Some(RemoteHandshakeFixture::pinned()),
            policy: RemotePolicyFixture::default(),
            expected_audience_hash: HASH_A,
        }
    }
}

fn remote_denial_reason(
    fixture: RemoteDenialFixture<'_>,
) -> Option<super::super::RemoteServerAdmissionReason> {
    let descriptor = remote_descriptor_with(
        fixture.transport,
        fixture.fingerprint_policy,
        fixture.protocol_role,
        fixture.expected_audience_hash,
    )
    .expect("descriptor should validate");
    let handshake = fixture
        .handshake
        .map(remote_handshake_with)
        .map(|value| value.expect("handshake should validate"));
    let policy = remote_policy_with(fixture.policy).expect("policy should validate");
    remote_decision(&descriptor, handshake.as_ref(), &policy)
        .expect("admission should evaluate")
        .reason()
}
