pub(super) const AUDIENCE: &str = "animus-headless";
pub(super) const OTHER_AUDIENCE: &str = "other-runtime";
pub(super) const ACTOR: &str = "agent@example";
pub(super) const SCOPE: &str = "session-1";
pub(super) const RESOURCE: &str = "session:session-1";
pub(super) const OTHER_RESOURCE: &str = "session:other";
pub(super) const ABILITY: &str = "prompt_submit";
pub(super) const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
pub(super) const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
pub(super) const HASH_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
pub(super) const HASH_D: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
pub(super) const BACKEND: &str = "goose-serve-backend";
pub(super) const SECRET_SOURCE: &str = "goose-shared-secret-source";
pub(super) const PROTOCOL_VERSION: &str = "goose-headless-v1";
pub(super) const FEATURE_PROMPT: &str = "prompt-submit";
pub(super) const FEATURE_EVENTS: &str = "event-stream";
pub(super) const CAPABILITY_PROMPT: &str = "capability.prompt-submit";
pub(super) const CAPABILITY_EVENTS: &str = "capability.event-stream";
pub(super) const CAPABILITY_ADMIN: &str = "capability.admin";
pub(super) const RAW_ENDPOINT: &str = "https://127.0.0.1:3000";
pub(super) const PROMPT_BODY: &str = "prompt_body: raw prompt";

#[derive(Clone, Copy)]
pub(super) struct CredentialFixture<'a> {
    pub(super) audience: &'a str,
    pub(super) resource: &'a str,
    pub(super) ability: &'a str,
}

pub(super) const VALID_CREDENTIAL: CredentialFixture<'static> = CredentialFixture {
    audience: AUDIENCE,
    resource: RESOURCE,
    ability: ABILITY,
};
pub(super) const WRONG_AUDIENCE_CREDENTIAL: CredentialFixture<'static> = CredentialFixture {
    audience: OTHER_AUDIENCE,
    resource: RESOURCE,
    ability: ABILITY,
};
pub(super) const OTHER_RESOURCE_CREDENTIAL: CredentialFixture<'static> = CredentialFixture {
    audience: AUDIENCE,
    resource: OTHER_RESOURCE,
    ability: ABILITY,
};

pub(super) fn denied_reason(
    credential: Option<super::super::CredentialEvidenceDraft<'_>>,
    policy_hash: Option<&str>,
    caveat: super::super::AdmissionCaveat,
) -> Option<super::super::AdmissionReason> {
    let request = super::super::AdmissionRequest::new(&super::support::draft(
        credential,
        policy_hash,
        caveat,
    ))
    .expect("request should validate");
    super::super::admit(&request)
        .expect("decision should evaluate")
        .reason()
}

pub(super) fn request_with(
    credential: Option<super::super::CredentialEvidenceDraft<'_>>,
    policy_hash: Option<&str>,
) -> Result<super::super::AdmissionRequest, super::super::AdmissionError> {
    super::super::AdmissionRequest::new(&draft(
        credential,
        policy_hash,
        super::super::AdmissionCaveat::Allow,
    ))
}

pub(super) fn draft<'a>(
    credential: Option<super::super::CredentialEvidenceDraft<'a>>,
    policy_hash: Option<&'a str>,
    caveat: super::super::AdmissionCaveat,
) -> super::super::AdmissionRequestDraft<'a> {
    super::super::AdmissionRequestDraft {
        audience: AUDIENCE,
        actor: ACTOR,
        entrypoint: super::super::Entrypoint::HeadlessRpc,
        scope: SCOPE,
        action: super::super::Action::PromptSubmit,
        resource: RESOURCE,
        ability: ABILITY,
        credential,
        caveat,
        policy_hash,
    }
}

pub(super) fn credential(
    fixture: CredentialFixture<'_>,
) -> super::super::CredentialEvidenceDraft<'_> {
    super::super::CredentialEvidenceDraft {
        audience: fixture.audience,
        actor: ACTOR,
        resource: fixture.resource,
        ability: fixture.ability,
        credential_hash: HASH_A,
        expiry: super::super::CredentialExpiry::Current,
        verifier_status: super::super::CredentialVerifierStatus::Verified,
    }
}

pub(super) fn remote_descriptor_with(
    transport: super::super::RemoteServerTransportKind,
    fingerprint_policy: super::super::RemoteServerFingerprintPolicy,
    protocol_role: super::super::RemoteServerProtocolRole,
    expected_audience_hash: &str,
) -> Result<super::super::RemoteServerDescriptor, super::super::RemoteServerAdmissionError> {
    super::super::RemoteServerDescriptor::new(&super::super::RemoteServerDescriptorDraft {
        backend_identity: BACKEND,
        endpoint_identity_hash: HASH_C,
        endpoint_identity_class: super::super::RemoteServerEndpointIdentityClass::HashOnly,
        transport,
        protocol_role,
        platform_role: super::super::RemoteServerPlatformRole::GooseServeBackend,
        tls_requirement: super::super::RemoteServerTlsRequirement::Required,
        fingerprint_policy,
        secret_source_label: SECRET_SOURCE,
        secret_source_class: super::super::RemoteServerSecretSourceClass::SharedSecret,
        scope: SCOPE,
        capability_ceilings: &[CAPABILITY_PROMPT, CAPABILITY_EVENTS],
        expected_audience_hash,
        provenance_hash: HASH_D,
    })
}

pub(super) fn remote_descriptor()
-> Result<super::super::RemoteServerDescriptor, super::super::RemoteServerAdmissionError> {
    remote_descriptor_with(
        super::super::RemoteServerTransportKind::Https,
        super::super::RemoteServerFingerprintPolicy::Pinned,
        super::super::RemoteServerProtocolRole::HeadlessControl,
        HASH_A,
    )
}

#[derive(Clone, Copy)]
pub(super) struct RemoteHandshakeFixture<'a> {
    pub(super) tls: super::super::RemoteServerObservedTls,
    pub(super) fingerprint_status: super::super::RemoteServerFingerprintStatus,
    pub(super) auth_outcome: super::super::RemoteServerAuthOutcome,
    pub(super) protocol_status: super::super::RemoteServerProtocolVersionStatus,
    pub(super) audience_hash: &'a str,
}

impl RemoteHandshakeFixture<'_> {
    pub(super) const fn pinned() -> Self {
        Self {
            tls: super::super::RemoteServerObservedTls::Enabled,
            fingerprint_status: super::super::RemoteServerFingerprintStatus::Matched,
            auth_outcome: super::super::RemoteServerAuthOutcome::Authorized,
            protocol_status: super::super::RemoteServerProtocolVersionStatus::Supported,
            audience_hash: HASH_A,
        }
    }
}

pub(super) fn remote_handshake_with(
    fixture: RemoteHandshakeFixture<'_>,
) -> Result<super::super::RemoteServerHandshakeSummary, super::super::RemoteServerAdmissionError> {
    super::super::RemoteServerHandshakeSummary::new(
        &super::super::RemoteServerHandshakeSummaryDraft {
            observed_tls: fixture.tls,
            fingerprint_status: fixture.fingerprint_status,
            auth_outcome: fixture.auth_outcome,
            protocol_version: PROTOCOL_VERSION,
            protocol_version_status: fixture.protocol_status,
            accepted_features: &[FEATURE_PROMPT, FEATURE_EVENTS],
            audience_hash: fixture.audience_hash,
            safe_failure_class: super::super::RemoteServerFailureClass::None,
        },
    )
}

pub(super) fn remote_handshake()
-> Result<super::super::RemoteServerHandshakeSummary, super::super::RemoteServerAdmissionError> {
    remote_handshake_with(RemoteHandshakeFixture::pinned())
}

#[derive(Clone, Copy)]
pub(super) struct RemotePolicyFixture<'a> {
    pub(super) requested_scope: &'a str,
    pub(super) requested_capabilities: &'a [&'a str],
    pub(super) credential_source_availability:
        super::super::RemoteServerCredentialSourceAvailability,
    pub(super) tofu_approved: bool,
    pub(super) fallback_requested: bool,
}

impl RemotePolicyFixture<'_> {
    pub(super) const fn default() -> Self {
        Self {
            requested_scope: SCOPE,
            requested_capabilities: &[CAPABILITY_PROMPT],
            credential_source_availability:
                super::super::RemoteServerCredentialSourceAvailability::Available,
            tofu_approved: false,
            fallback_requested: false,
        }
    }
}

pub(super) fn remote_policy_with(
    fixture: RemotePolicyFixture<'_>,
) -> Result<super::super::RemoteServerAdmissionPolicy, super::super::RemoteServerAdmissionError> {
    super::super::RemoteServerAdmissionPolicy::new(
        &super::super::RemoteServerAdmissionPolicyDraft {
            policy_hash: HASH_B,
            requested_scope: fixture.requested_scope,
            requested_capabilities: fixture.requested_capabilities,
            credential_source_availability: fixture.credential_source_availability,
            tofu_approved: fixture.tofu_approved,
            fallback_requested: fixture.fallback_requested,
        },
    )
}

pub(super) fn remote_policy()
-> Result<super::super::RemoteServerAdmissionPolicy, super::super::RemoteServerAdmissionError> {
    remote_policy_with(RemotePolicyFixture::default())
}

pub(super) fn remote_decision(
    descriptor: &super::super::RemoteServerDescriptor,
    handshake: Option<&super::super::RemoteServerHandshakeSummary>,
    policy: &super::super::RemoteServerAdmissionPolicy,
) -> Result<super::super::RemoteServerAdmissionDecision, super::super::RemoteServerAdmissionError> {
    super::super::evaluate_remote_server_admission(super::super::RemoteServerAdmissionRequest {
        descriptor,
        handshake,
        policy,
    })
}
