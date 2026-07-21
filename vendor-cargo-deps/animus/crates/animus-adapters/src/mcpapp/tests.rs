use super::*;

const SAFE_RESOURCE: &str = "ui://tool/result/summary";
const SAFE_MIME: &str = "text/html+skybridge";
const SAFE_TITLE: &str = "Safe summary";
const SAFE_DOMAIN: &str = "static.example.invalid";
const SAFE_PERMISSION: &str = "clipboard-read";
const SAFE_CONTEXT: &str = "theme";
const SAFE_CALLBACK: &str = "bounded-selection";

#[test]
fn evaluation_matrix_accepts_safe_metadata_fixture() {
    let evaluation = Descriptor::evaluate(&safe_descriptor()).expect("safe fixture should parse");

    assert_eq!(
        evaluation.adoption_decision(),
        AdoptionDecision::DataOnlyAdoptable
    );
    assert_eq!(evaluation.risk_reason(), RiskReason::SafeMetadata);
    assert_eq!(Evaluation::covered_axes(), MCP_APP_EVALUATION_AXES);
    assert_eq!(evaluation.descriptor().resource_uri(), SAFE_RESOURCE);
    assert_eq!(evaluation.descriptor().size_policy(), SizePolicy::Bounded);
    assert_eq!(
        evaluation.descriptor().lifecycle_policy(),
        LifecyclePolicy::BoundedCallbacks
    );
}

#[test]
fn evaluation_matrix_classifies_remote_domains_as_policy_gated() {
    let evaluation = Descriptor::evaluate(&DescriptorDraft {
        policy_admission: PolicyAdmissionDraft {
            remote_domains: PolicyAdmission::Denied,
            browser_permissions: PolicyAdmission::Admitted,
            host_callbacks: PolicyAdmission::Admitted,
            direct_chat_messages: PolicyAdmission::Denied,
        },
        ..safe_descriptor()
    })
    .expect("policy-gated fixture should parse");

    assert_eq!(
        evaluation.adoption_decision(),
        AdoptionDecision::NeedsHostPolicy
    );
    assert_eq!(
        evaluation.risk_reason(),
        RiskReason::RemoteDomainRequiresAdmission
    );
}

#[test]
fn evaluation_matrix_rejects_unsafe_fixtures() {
    let cases = [
        (
            DescriptorDraft {
                csp_domains: &["*"],
                ..safe_descriptor()
            },
            RiskReason::BroadDomain,
        ),
        (
            DescriptorDraft {
                permission_requests: &[CAMERA_PERMISSION],
                policy_admission: PolicyAdmissionDraft::default(),
                ..safe_descriptor()
            },
            RiskReason::BroadPermission,
        ),
        (
            DescriptorDraft {
                callback_channels: &[SAFE_CALLBACK],
                lifecycle_policy: Some(LIFECYCLE_POLICY_UNBOUNDED_CALLBACKS),
                ..safe_descriptor()
            },
            RiskReason::UnboundedCallback,
        ),
        (
            DescriptorDraft {
                host_context_sync: &[AMBIENT_HOST_CONTROL_CONTEXT],
                ..safe_descriptor()
            },
            RiskReason::AmbientHostControl,
        ),
        (
            DescriptorDraft {
                callback_channels: &[DIRECT_CHAT_CALLBACK],
                ..safe_descriptor()
            },
            RiskReason::DirectMessageInjection,
        ),
    ];

    for (draft, expected_reason) in cases {
        let evaluation = Descriptor::evaluate(&draft).expect("unsafe fixture should parse");
        assert_eq!(
            evaluation.adoption_decision(),
            AdoptionDecision::RejectDefault
        );
        assert_eq!(evaluation.risk_reason(), expected_reason);
    }
}

#[test]
fn descriptor_validation_is_data_only() {
    let evaluation = Descriptor::evaluate(&DescriptorDraft {
        claims_runtime_execution: true,
        ..safe_descriptor()
    })
    .expect("runtime-claim fixture should parse as data");
    let rendered_effects = alloc::format!("{PROHIBITED_MCP_APP_VALIDATION_EFFECTS:?}");

    assert_eq!(
        evaluation.adoption_decision(),
        AdoptionDecision::RejectDefault
    );
    assert_eq!(evaluation.risk_reason(), RiskReason::RuntimeExecutionClaim);
    assert!(rendered_effects.contains("HTML rendering"));
    assert!(rendered_effects.contains("JavaScript execution"));
    assert!(rendered_effects.contains("chat callback dispatch"));
}

#[test]
fn descriptor_validation_rejects_malformed_data() {
    let missing_uri = Descriptor::evaluate(&DescriptorDraft {
        resource_uri: None,
        ..safe_descriptor()
    })
    .expect_err("resource identity is required");
    let bad_policy_error = Descriptor::evaluate(&DescriptorDraft {
        size_policy: Some("unbounded-window"),
        ..safe_descriptor()
    })
    .expect_err("unknown size policy is rejected");

    assert!(matches!(
        missing_uri,
        EvaluationError::Contract(crate::AdapterContractError::EmptyField { .. })
    ));
    assert!(matches!(
        bad_policy_error,
        EvaluationError::UnsupportedValue { .. }
    ));
}

fn safe_descriptor() -> DescriptorDraft<'static> {
    DescriptorDraft {
        resource_uri: Some(SAFE_RESOURCE),
        mime_profile: Some(SAFE_MIME),
        title: Some(SAFE_TITLE),
        csp_domains: &[SAFE_DOMAIN],
        permission_requests: &[SAFE_PERMISSION],
        host_context_sync: &[SAFE_CONTEXT],
        callback_channels: &[SAFE_CALLBACK],
        size_policy: Some(SIZE_POLICY_BOUNDED),
        lifecycle_policy: Some(LIFECYCLE_POLICY_BOUNDED_CALLBACKS),
        policy_admission: PolicyAdmissionDraft {
            remote_domains: PolicyAdmission::Admitted,
            browser_permissions: PolicyAdmission::Admitted,
            host_callbacks: PolicyAdmission::Admitted,
            direct_chat_messages: PolicyAdmission::Denied,
        },
        claims_runtime_execution: false,
    }
}
