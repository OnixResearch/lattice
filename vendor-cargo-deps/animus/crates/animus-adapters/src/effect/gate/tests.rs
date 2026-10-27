use super::*;
const HASH_A: &str = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HASH_B: &str = "b3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const HASH_C: &str = "b3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const AUDIENCE_DID: &str = "did:key:z6Mktestholder";
const RESOURCE: &str = "did:key:z6Mkresource/docs/report";
const MAX_IDENTITY_BYTES: usize = 160;
const MAX_RESOURCE_BYTES: usize = 512;
const MAX_CHECKED_FACTS: u32 = 32;
const CHECKED_FACTS: u32 = 7;

fn bounds() -> HostAuthorityBounds {
    HostAuthorityBounds {
        identity_capacity_bytes: MAX_IDENTITY_BYTES,
        resource_ceiling_bytes: MAX_RESOURCE_BYTES,
        checked_fact_allowance: MAX_CHECKED_FACTS,
    }
}

fn binding() -> AgentAuthorityBinding {
    admit_agent_authority_binding(AgentAuthorityBindingDraft {
        display_label: Some("reviewer"),
        audience_did: AUDIENCE_DID,
        agent_definition_identity: HASH_A,
        binding_policy_identity: HASH_B,
        freshness_identity: HASH_C,
        subject_identity: HASH_A,
        task_identity: HASH_B,
        actor_identity: HASH_C,
        role_plan_identity: HASH_A,
        session_identity: HASH_B,
        freshness: BindingFreshness::Current,
        bounds: bounds(),
    })
    .expect("valid binding")
}

fn request(binding: &AgentAuthorityBinding) -> HostAuthorityRequest {
    admit_host_authority_request(HostAuthorityRequestDraft {
        action: crate::GuardedAction::Filesystem,
        contract: "document-write",
        resource: RESOURCE,
        ability: "document/write",
        binding,
        agent_definition_identity: HASH_A,
        subject_identity: HASH_A,
        task_identity: HASH_B,
        actor_identity: HASH_C,
        role_plan_identity: HASH_A,
        session_identity: HASH_B,
        policy_identity: HASH_C,
        grant_identity: HASH_A,
        operation_profile_identity: HASH_B,
        correlation_identity: "dispatch-a",
        bounds: bounds(),
    })
    .expect("valid request")
}

fn allowed(request: &HostAuthorityRequest) -> HostAuthorityDecision {
    build_host_authority_decision(HostAuthorityDecisionDraft {
        request,
        provider_identity: HASH_A,
        policy_identity: HASH_C,
        status: HostAuthorityStatus::Allowed,
        issue: HostAuthorityIssue::None,
        principal_hash: HASH_B,
        resource_hash: HASH_C,
        checked_fact_count: CHECKED_FACTS,
        freshness: HostAuthorityFreshnessClass::Current,
        replay: HostAuthorityReplayClass::Admitted,
        provider_receipt_identity: HASH_A,
    })
    .expect("valid decision")
}

fn all_gates() -> RequiredHostGates {
    RequiredHostGates {
        agent_definition: Status::Allowed,
        semantic_role: Status::Allowed,
        parent: Status::Allowed,
        task: Status::Allowed,
        model: Status::Allowed,
        tool: Status::Allowed,
        workspace: Status::Allowed,
        approval: Status::Allowed,
        sandbox: Status::Allowed,
        cancellation: Status::Allowed,
        timeout: Status::Allowed,
        budget: Status::Allowed,
        runtime_authority_seal: Status::Allowed,
    }
}

#[test]
fn binding_request_provider_replacement_and_receipt_are_deterministic() {
    let binding = binding();
    let admitted_request = request(&binding);
    let decision = allowed(&admitted_request);
    let receipt = build_host_authority_receipt(HostAuthorityReceiptDraft {
        decision: &decision,
        dispatch_class: "committed",
        non_claims: &HOST_AUTHORITY_NON_CLAIMS,
    })
    .expect("safe receipt");
    let mut provider = FakeProvider {
        allow: true,
        calls: 0,
    };
    let replacement = provider
        .authorize(&admitted_request)
        .expect("replaceable provider");

    assert_eq!(binding.display_label(), Some("reviewer"));
    assert_ne!(binding.display_label(), Some(binding.audience_did()));
    assert_eq!(
        admitted_request.request_identity(),
        request(&binding).request_identity()
    );
    assert_eq!(receipt.status, HostAuthorityStatus::Allowed);
    assert_eq!(provider.calls, 1);
    assert!(replacement.is_allowed());
    assert_eq!(
        intersect_host_authority(&decision, all_gates()).status,
        HostAuthorityStatus::Allowed
    );
}

#[test]
fn stale_name_only_drift_duplicate_wildcard_and_local_denial_fail_closed() {
    let mut stale = AgentAuthorityBindingDraft {
        display_label: Some("reviewer"),
        audience_did: AUDIENCE_DID,
        agent_definition_identity: HASH_A,
        binding_policy_identity: HASH_B,
        freshness_identity: HASH_C,
        subject_identity: HASH_A,
        task_identity: HASH_B,
        actor_identity: HASH_C,
        role_plan_identity: HASH_A,
        session_identity: HASH_B,
        freshness: BindingFreshness::Stale,
        bounds: bounds(),
    };
    assert_eq!(
        admit_agent_authority_binding(stale).expect_err("stale binding"),
        HostAuthorityIssue::StaleBinding
    );
    stale.freshness = BindingFreshness::Current;
    stale.audience_did = "";
    assert_eq!(
        admit_agent_authority_binding(stale).expect_err("name cannot replace DID"),
        HostAuthorityIssue::InvalidPrincipal
    );

    let binding = binding();
    assert_eq!(
        validate_agent_authority_binding_set(&[binding.clone(), binding.clone()])
            .expect_err("duplicate binding"),
        HostAuthorityIssue::DuplicateBinding
    );
    let mut drift = HostAuthorityRequestDraft {
        action: crate::GuardedAction::Tool,
        contract: "tool-call",
        resource: "tool:read",
        ability: "tool/invoke",
        binding: &binding,
        agent_definition_identity: HASH_A,
        subject_identity: HASH_A,
        task_identity: HASH_B,
        actor_identity: HASH_C,
        role_plan_identity: HASH_A,
        session_identity: HASH_C,
        policy_identity: HASH_C,
        grant_identity: HASH_A,
        operation_profile_identity: HASH_B,
        correlation_identity: "dispatch-a",
        bounds: bounds(),
    };
    assert_eq!(
        admit_host_authority_request(drift).expect_err("cross-session drift"),
        HostAuthorityIssue::BindingMismatch
    );
    drift.session_identity = HASH_B;
    drift.resource = "*";
    assert_eq!(
        admit_host_authority_request(drift).expect_err("wildcard"),
        HostAuthorityIssue::InvalidResource
    );

    let request = request(&binding);
    let decision = allowed(&request);
    let mut gates = all_gates();
    gates.sandbox = Status::Denied;
    let effective = intersect_host_authority(&decision, gates);
    assert_eq!(effective.status, HostAuthorityStatus::Denied);
    assert_eq!(effective.issue, HostAuthorityIssue::LocalGateDenied);
}

#[test]
fn inconsistent_decisions_and_private_receipt_material_are_rejected() {
    let binding = binding();
    let request = request(&binding);
    assert_eq!(
        build_host_authority_decision(HostAuthorityDecisionDraft {
            request: &request,
            provider_identity: HASH_A,
            policy_identity: HASH_C,
            status: HostAuthorityStatus::Allowed,
            issue: HostAuthorityIssue::PolicyDenied,
            principal_hash: HASH_B,
            resource_hash: HASH_C,
            checked_fact_count: CHECKED_FACTS,
            freshness: HostAuthorityFreshnessClass::Current,
            replay: HostAuthorityReplayClass::Admitted,
            provider_receipt_identity: HASH_A,
        })
        .expect_err("denial cannot become allow"),
        HostAuthorityIssue::DecisionMismatch
    );
    assert_eq!(
        validate_host_authority_receipt_text("compact_token=header.payload.signature")
            .expect_err("bearer leak"),
        HostAuthorityReceiptLeak::Credential
    );
    assert_eq!(
        validate_host_authority_receipt_text("private_path=/secret").expect_err("private leak"),
        HostAuthorityReceiptLeak::PrivateMaterial
    );
    assert_eq!(
        validate_host_authority_receipt_text("agent is safe").expect_err("overclaim"),
        HostAuthorityReceiptLeak::Overclaim
    );
}

struct FakeProvider {
    allow: bool,
    calls: u32,
}

impl HostAuthorityService for FakeProvider {
    fn authorize(
        &mut self,
        request: &HostAuthorityRequest,
    ) -> Result<HostAuthorityDecision, HostAuthorityIssue> {
        self.calls = self
            .calls
            .checked_add(1)
            .ok_or(HostAuthorityIssue::InvalidBound)?;
        let (status, issue, replay) = if self.allow {
            (
                HostAuthorityStatus::Allowed,
                HostAuthorityIssue::None,
                HostAuthorityReplayClass::Admitted,
            )
        } else {
            (
                HostAuthorityStatus::Denied,
                HostAuthorityIssue::PolicyDenied,
                HostAuthorityReplayClass::Denied,
            )
        };
        build_host_authority_decision(HostAuthorityDecisionDraft {
            request,
            provider_identity: HASH_A,
            policy_identity: request.policy_identity(),
            status,
            issue,
            principal_hash: HASH_B,
            resource_hash: HASH_C,
            checked_fact_count: CHECKED_FACTS,
            freshness: HostAuthorityFreshnessClass::Current,
            replay,
            provider_receipt_identity: HASH_A,
        })
    }
}
