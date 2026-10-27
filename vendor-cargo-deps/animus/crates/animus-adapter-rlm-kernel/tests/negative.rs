#![allow(missing_docs)]

mod support;

use animus_adapter_rlm_kernel::{
    ChildSpawnRequest, CompactionCause, CompactionRequest, ContextBinding, EnvironmentState, Error,
    EvaluationMetrics, EvaluationRequest, HostOperationClass, HostRequest, Receipt, ReceiptDraft,
    ReceiptLeakClass, RlmKernelPolicy, RlmKernelPolicyDraft, RlmKernelStatus, bind_prompt_context,
    plan_compaction, plan_evaluation,
};

const SAFE_SOURCE: &str = "(+ 1 2)";
const EXCESS_FUEL: u64 = support::FUEL_LIMIT + 1;
const EXCESS_DEPTH: u32 = support::CHILD_DEPTH_LIMIT + 1;

#[test]
fn bound_failures_are_atomic_and_ambient_requests_fail_closed() {
    let policy = support::policy();
    let initial = EnvironmentState::default();
    let binding = ContextBinding::new("task_context", "unchanged", support::ENVIRONMENT_CHAR_LIMIT)
        .expect("binding must parse");
    let bound =
        bind_prompt_context(&initial, &[binding], &policy).expect("binding must be admitted");
    let snapshot = bound.clone();

    let error = plan_evaluation(
        &bound,
        &EvaluationRequest {
            source: SAFE_SOURCE,
            metrics: EvaluationMetrics {
                fuel: EXCESS_FUEL,
                recursion_depth: 1,
                allocation_bytes: 1,
            },
        },
        &policy,
    )
    .expect_err("excess fuel must fail");
    assert!(matches!(
        error,
        Error::BoundExceeded {
            field: "evaluation_fuel",
            ..
        }
    ));
    assert_eq!(bound, snapshot, "failed planning must not mutate state");

    let ambient_child = ChildSpawnRequest::new(
        support::CHILD_PROFILE,
        "ambient child",
        EXCESS_DEPTH,
        support::CHILD_FAN_OUT_LIMIT,
        support::CHILD_TOKEN_LIMIT,
        support::CHILD_WALL_TIME_LIMIT_MS,
        support::CHILD_RESULT_CHAR_LIMIT,
        true,
        true,
    )
    .expect("request shape must parse before admission");
    assert!(matches!(
        animus_adapter_rlm_kernel::admit_child_spawn(&policy, &ambient_child),
        Err(Error::AmbientChildSpawnDenied)
    ));

    let restricted = restricted_policy();
    let shell = HostRequest::shell("cargo test", support::HOST_REQUEST_CHAR_LIMIT)
        .expect("shell request must parse");
    assert!(matches!(
        animus_adapter_rlm_kernel::admit_host_request(&restricted, &shell),
        Err(Error::OperationNotAdmitted {
            operation: HostOperationClass::Shell
        })
    ));
}

#[test]
fn implicit_clears_and_receipt_payloads_or_overclaims_are_rejected() {
    let policy = support::policy();
    let state = EnvironmentState::default();
    assert!(matches!(
        plan_compaction(
            &state,
            CompactionRequest {
                cause: CompactionCause::Compaction,
                clear_kernel: true,
            }
        ),
        Err(Error::ImplicitKernelClearDenied)
    ));

    assert_receipt_leaks_rejected(&policy);
    assert_receipt_overclaim_rejected(&policy);
    assert_schema_drift_rejected();
}

fn assert_receipt_leaks_rejected(policy: &RlmKernelPolicy) {
    for (variable_value, code_body, child_transcript, credential, raw_path, expected) in [
        (
            Some("private value"),
            None,
            None,
            None,
            None,
            ReceiptLeakClass::VariableValue,
        ),
        (
            None,
            Some("model code"),
            None,
            None,
            None,
            ReceiptLeakClass::CodeBody,
        ),
        (
            None,
            None,
            Some("child transcript"),
            None,
            None,
            ReceiptLeakClass::ChildTranscript,
        ),
        (
            None,
            None,
            None,
            Some("credential"),
            None,
            ReceiptLeakClass::Credential,
        ),
        (
            None,
            None,
            None,
            None,
            Some("private/path"),
            ReceiptLeakClass::RawPath,
        ),
    ] {
        let leaked = Receipt::from_draft(&ReceiptDraft {
            status: RlmKernelStatus::Completed,
            policy_identity: policy.identity(),
            operation_classes: &[],
            counts: &[],
            child_lineage_hashes: &[],
            artifact_hashes: &[],
            kernel_state_preserved: false,
            non_claims: animus_adapter_rlm_kernel::REQUIRED_NON_CLAIMS,
            variable_value,
            code_body,
            child_transcript,
            credential,
            raw_path,
            claims: &[],
        });
        assert!(matches!(
            leaked,
            Err(Error::ReceiptLeak { class }) if class == expected
        ));
    }
}

fn assert_receipt_overclaim_rejected(policy: &RlmKernelPolicy) {
    let overclaim = Receipt::from_draft(&ReceiptDraft {
        status: RlmKernelStatus::Completed,
        policy_identity: policy.identity(),
        operation_classes: &[],
        counts: &[],
        child_lineage_hashes: &[],
        artifact_hashes: &[],
        kernel_state_preserved: false,
        non_claims: animus_adapter_rlm_kernel::REQUIRED_NON_CLAIMS,
        variable_value: None,
        code_body: None,
        child_transcript: None,
        credential: None,
        raw_path: None,
        claims: &["security_sandbox"],
    });
    assert!(matches!(overclaim, Err(Error::ProhibitedClaim)));
}

fn assert_schema_drift_rejected() {
    let full = support::policy();
    let schema_drift = RlmKernelPolicy::from_draft(&RlmKernelPolicyDraft {
        schema: "animus.rlm-kernel-policy.unsupported",
        cohort: full.cohort(),
        cohort_revision: full.cohort_revision(),
        profile: full.profile(),
        bounds: full.bounds(),
        admitted_operations: full.admitted_operations(),
        admitted_child_profiles: &[support::CHILD_PROFILE],
        non_claims: animus_adapter_rlm_kernel::REQUIRED_NON_CLAIMS,
    });
    assert!(matches!(
        schema_drift,
        Err(Error::EmptyField { field: "schema" })
    ));
}

fn restricted_policy() -> RlmKernelPolicy {
    let full = support::policy();
    RlmKernelPolicy::from_draft(&RlmKernelPolicyDraft {
        schema: animus_adapter_rlm_kernel::RLM_KERNEL_POLICY_SCHEMA,
        cohort: full.cohort(),
        cohort_revision: full.cohort_revision(),
        profile: "restricted",
        bounds: full.bounds(),
        admitted_operations: &[HostOperationClass::ContextRead],
        admitted_child_profiles: &[support::CHILD_PROFILE],
        non_claims: animus_adapter_rlm_kernel::REQUIRED_NON_CLAIMS,
    })
    .expect("restricted policy must be valid")
}
