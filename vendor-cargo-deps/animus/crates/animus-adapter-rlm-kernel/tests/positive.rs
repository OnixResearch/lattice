#![allow(missing_docs)]

mod support;

use animus_adapter_rlm_kernel::ChildResult;
use animus_adapter_rlm_kernel::ChildSpawnRequest;
use animus_adapter_rlm_kernel::CompactionCause;
use animus_adapter_rlm_kernel::CompactionRequest;
use animus_adapter_rlm_kernel::ContextBinding;
use animus_adapter_rlm_kernel::EnvironmentState;
use animus_adapter_rlm_kernel::EvaluationMetrics;
use animus_adapter_rlm_kernel::EvaluationRequest;
use animus_adapter_rlm_kernel::HostOperationClass;
use animus_adapter_rlm_kernel::HostRequest;
use animus_adapter_rlm_kernel::Lineage;
use animus_adapter_rlm_kernel::Receipt;
use animus_adapter_rlm_kernel::ReceiptCount;
use animus_adapter_rlm_kernel::ReceiptDraft;
use animus_adapter_rlm_kernel::RlmKernelStatus;
use animus_adapter_rlm_kernel::bind_prompt_context;
use animus_adapter_rlm_kernel::commit_evaluation;
use animus_adapter_rlm_kernel::plan_compaction;
use animus_adapter_rlm_kernel::plan_evaluation;

const PROMPT_BINDING: &str = "task_context";
const PROMPT_VALUE: &str = "inspect adapter boundaries";
const FIRST_SOURCE: &str = "(define (double value) (* value 2))";
const SECOND_SOURCE: &str = "(double 21)";
const EVALUATION_FUEL: u64 = 1_000;
const EVALUATION_DEPTH: u32 = 4;
const EVALUATION_ALLOCATION_BYTES: u64 = 4_096;
const CHILD_CONTEXT: &str = "Review the adapter boundary";
const CHILD_RESULT_VALUE: &str = "No ambient effect paths found";

#[test]
fn policy_environment_host_request_child_and_receipt_flow_is_bounded() {
    let policy = support::policy();
    assert_eq!(policy.cohort_revision(), support::PRIME_AGENT_REVISION);
    assert_eq!(policy.identity().len(), 64);

    let binding = ContextBinding::new(PROMPT_BINDING, PROMPT_VALUE, support::ENVIRONMENT_CHAR_LIMIT)
        .expect("prompt binding must be valid");
    let initial = EnvironmentState::default();
    let bound = bind_prompt_context(&initial, &[binding], &policy).expect("prompt context must bind atomically");
    assert_eq!(bound.bindings().len(), 1);

    let first = plan_evaluation(
        &bound,
        &EvaluationRequest {
            source: FIRST_SOURCE,
            metrics: metrics(),
        },
        &policy,
    )
    .expect("first evaluation must be admitted");
    let after_first = commit_evaluation(&bound, &first, &policy).expect("successful first evaluation must commit");
    let second = plan_evaluation(
        &after_first,
        &EvaluationRequest {
            source: SECOND_SOURCE,
            metrics: metrics(),
        },
        &policy,
    )
    .expect("second evaluation must observe persisted definitions");
    let after_second =
        commit_evaluation(&after_first, &second, &policy).expect("successful second evaluation must commit");
    assert_eq!(after_second.definitions().len(), 2);

    let request = HostRequest::context_read(PROMPT_BINDING).expect("context request must parse");
    let admission =
        animus_adapter_rlm_kernel::admit_host_request(&policy, &request).expect("context request must be admitted");
    assert_eq!(admission.operation(), HostOperationClass::ContextRead);

    let child_request = ChildSpawnRequest::new(
        support::CHILD_PROFILE,
        CHILD_CONTEXT,
        support::CHILD_DEPTH_LIMIT,
        support::CHILD_FAN_OUT_LIMIT,
        support::CHILD_TOKEN_LIMIT,
        support::CHILD_WALL_TIME_LIMIT_MS,
        support::CHILD_RESULT_CHAR_LIMIT,
        false,
        false,
    )
    .expect("child request must parse");
    animus_adapter_rlm_kernel::admit_child_spawn(&policy, &child_request).expect("bounded child must be admitted");
    let lineage = Lineage::new(support::HASH_A, support::HASH_B, support::CHILD_PROFILE).expect("lineage must parse");
    let child_result =
        ChildResult::new(&child_request, lineage, CHILD_RESULT_VALUE).expect("bounded child result must parse");

    let compaction = plan_compaction(&after_second, CompactionRequest {
        cause: CompactionCause::Compaction,
        clear_kernel: false,
    })
    .expect("compaction must preserve kernel state");
    assert!(compaction.kernel_state_preserved());
    assert_eq!(compaction.definition_count(), 2);

    let counts = [
        ReceiptCount::new("bindings", u64::from(compaction.binding_count())).expect("count must parse"),
        ReceiptCount::new("definitions", u64::from(compaction.definition_count())).expect("count must parse"),
    ];
    let receipt = Receipt::from_draft(&ReceiptDraft {
        status: RlmKernelStatus::Preserved,
        policy_identity: policy.identity(),
        operation_classes: &[HostOperationClass::ContextRead, HostOperationClass::Subagent],
        counts: &counts,
        child_lineage_hashes: &[child_result.lineage().child_id_hash()],
        artifact_hashes: &[first.source_hash(), second.source_hash()],
        kernel_state_preserved: true,
        non_claims: animus_adapter_rlm_kernel::REQUIRED_NON_CLAIMS,
        variable_value: None,
        code_body: None,
        child_transcript: None,
        credential: None,
        raw_path: None,
        claims: &["bounded_processing"],
    })
    .expect("metadata-only receipt must parse");
    assert_eq!(Receipt::schema(), animus_adapter_rlm_kernel::RLM_KERNEL_RECEIPT_SCHEMA);
    assert_eq!(receipt.identity().len(), 64);
}

const fn metrics() -> EvaluationMetrics {
    EvaluationMetrics {
        fuel: EVALUATION_FUEL,
        recursion_depth: EVALUATION_DEPTH,
        allocation_bytes: EVALUATION_ALLOCATION_BYTES,
    }
}
