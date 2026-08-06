// Dense adversarial fixtures intentionally share owner-qualified DTOs and grouped assertions.
#![allow(unknown_lints, assertion_density, non_trait_imports)]
#![allow(clippy::too_many_arguments)]

use super::*;
use crate::conversation_projection::ConversationProjectionCursor;
use crate::conversation_projection::ConversationProjectionSnapshot;

const ROOT_SEQUENCE: u32 = 1;
const INTERNAL_SEQUENCE: u32 = 2;
const LEFT_SEQUENCE: u32 = 3;
const RIGHT_SEQUENCE: u32 = 4;
const LEDGER_GENERATION: u64 = 7;
const ROOT_USAGE: u64 = 2;
const INTERNAL_USAGE: u64 = 3;
const LEAF_USAGE: u64 = 5;
const MAX_NODES: u32 = 64;
const MAX_DEPTH: u32 = 16;
const MAX_LABEL_BYTES: u32 = 64;
const MAX_LABELS_PER_NODE: u32 = 4;
const MAX_ORDERED_CHILDREN: u32 = 16;
const MAX_COMPARISON_SUFFIX: u32 = 16;
const MAX_SAFE_REFERENCES: u32 = 16;

const SESSION_ID: &str = "session-alpha";
const REVISION_ID: &str = "revision-seven";
const STALE_REVISION_ID: &str = "revision-six";
const ROOT_ID: &str = "entry-root";
const INTERNAL_ID: &str = "entry-internal";
const LEFT_ID: &str = "entry-left";
const RIGHT_ID: &str = "entry-right";
const UNKNOWN_ID: &str = "entry-unknown";
const POLICY_ID: &str = "policy-profile";
const PARAMETER_ID: &str = "parameter-profile";
const TOOL_PLAN_ID: &str = "tool-plan";
const EFFECT_ID: &str = "effect-receipt";
const OTHER_EFFECT_ID: &str = "effect-receipt-other";
const RESPONSE_ID: &str = "response-artifact";
const ISOLATION_ID: &str = "isolation-evidence";

#[test]
fn projects_multiple_leaves_active_path_bookmark_and_partial_order() {
    let policy = policy();
    let projection = project_branch_snapshot(&policy, &source()).expect("projection");
    let repeated = project_branch_snapshot(&policy, &source()).expect("repeated");

    assert_eq!(projection, repeated);
    assert_eq!(projection.presentation_order, vec![ROOT_ID, INTERNAL_ID, RIGHT_ID, LEFT_ID]);
    assert!(node(&projection, ROOT_ID).is_active_path);
    assert!(node(&projection, INTERNAL_ID).is_active_path);
    assert!(node(&projection, LEFT_ID).is_active_path);
    assert!(!node(&projection, RIGHT_ID).is_active_path);
    assert!(node(&projection, RIGHT_ID).is_bookmarked);
    assert_eq!(node(&projection, RIGHT_ID).labels, vec!["candidate"]);
    assert_eq!(node(&projection, INTERNAL_ID).child_count, 2);
    assert_eq!(projection.truncation.omitted_node_count, 0);
}

#[test]
fn plans_inspect_leaf_activation_append_only_fork_and_stale_blocks() {
    let projection = projection();
    let inspect =
        plan_branch_operation(&projection, BranchExplorerOperation::Inspect, REVISION_ID, LEFT_ID, INTERNAL_ID);
    assert_eq!(inspect.status, BranchPlanStatus::Admitted);
    assert_eq!(inspect.append_only_branch_operation, None);

    let activate =
        plan_branch_operation(&projection, BranchExplorerOperation::ActivateLeaf, REVISION_ID, LEFT_ID, RIGHT_ID);
    assert_eq!(activate.status, BranchPlanStatus::Admitted);

    let blocked_internal =
        plan_branch_operation(&projection, BranchExplorerOperation::ActivateLeaf, REVISION_ID, LEFT_ID, INTERNAL_ID);
    assert_eq!(blocked_internal.status, BranchPlanStatus::Blocked);
    assert_eq!(blocked_internal.diagnostic, Some(BranchExplorerIssue::NonLeafActivation));

    let fork =
        plan_branch_operation(&projection, BranchExplorerOperation::ForkFromNode, REVISION_ID, LEFT_ID, INTERNAL_ID);
    assert_eq!(fork.status, BranchPlanStatus::Admitted);
    assert_eq!(fork.append_only_branch_operation, Some(crate::branching::SessionBranchOperation::ForkFromMessage));
    assert!(fork.continuation_independent);

    let stale =
        plan_branch_operation(&projection, BranchExplorerOperation::ActivateLeaf, STALE_REVISION_ID, LEFT_ID, RIGHT_ID);
    assert_eq!(stale.status, BranchPlanStatus::Blocked);
    assert_eq!(stale.diagnostic, Some(BranchExplorerIssue::StaleRevision));
}

#[test]
fn shared_world_fork_never_claims_independent_continuation() {
    let mut input = source();
    input.external_state = Some(ExternalStatePosture::SharedWorld);
    input.isolation_evidence_id = None;
    let projection = project_branch_snapshot(&policy(), &input).expect("shared projection");
    let fork =
        plan_branch_operation(&projection, BranchExplorerOperation::ForkFromNode, REVISION_ID, LEFT_ID, INTERNAL_ID);
    assert_eq!(fork.status, BranchPlanStatus::Admitted);
    assert!(!fork.continuation_independent);
}

#[test]
fn compares_exact_common_ancestor_suffix_counts_and_usage() {
    let comparison = compare_branches(&policy(), &projection(), LEFT_ID, RIGHT_ID).expect("comparison");
    assert_eq!(comparison.common_ancestor_id, INTERNAL_ID);
    assert_eq!(comparison.left_suffix_ids, vec![LEFT_ID]);
    assert_eq!(comparison.right_suffix_ids, vec![RIGHT_ID]);
    assert_eq!(comparison.left_usage_units, Some(LEAF_USAGE));
    assert_eq!(comparison.right_usage_units, Some(LEAF_USAGE));
    assert!(comparison.unavailable_facts.contains(&String::from("winner")));
    assert!(!comparison.truncated);
}

#[test]
fn exact_generation_identity_marks_equivalence_without_deletion() {
    let projection = projection();
    let left = generation();
    let right = generation();
    let equivalence =
        validate_sibling_equivalence(&projection, LEFT_ID, RIGHT_ID, &left, &right, false).expect("equivalence");
    assert!(equivalence.branches_preserved);
    assert_eq!(equivalence.parent_entry_id, INTERNAL_ID);
    assert!(projection.nodes.iter().any(|entry| entry.entry_id == LEFT_ID));
    assert!(projection.nodes.iter().any(|entry| entry.entry_id == RIGHT_ID));
}

#[test]
fn differing_effects_and_deletion_attempts_reject_equivalence() {
    let projection = projection();
    let left = generation();
    let mut right = generation();
    right.effect_receipt_ids = vec![String::from(OTHER_EFFECT_ID)];
    assert_issue(
        validate_sibling_equivalence(&projection, LEFT_ID, RIGHT_ID, &left, &right, false),
        BranchExplorerIssue::GenerationMismatch,
    );
    assert_issue(
        validate_sibling_equivalence(&projection, LEFT_ID, RIGHT_ID, &left, &left, true),
        BranchExplorerIssue::DestructiveAction,
    );
}

#[test]
fn selected_read_binds_same_session_generation_and_leaf() {
    let projection = projection();
    let current_conversation = conversation(LEFT_ID, LEDGER_GENERATION);
    let binding = BranchSelectedRead::bind(&projection, LEFT_ID, &current_conversation).expect("selected read");
    assert_eq!(binding.revision_id, REVISION_ID);

    let stale = conversation(LEFT_ID, LEDGER_GENERATION.saturating_add(1));
    assert_issue(BranchSelectedRead::bind(&projection, LEFT_ID, &stale), BranchExplorerIssue::StaleRevision);
}

#[test]
fn empty_duplicate_missing_parent_multiple_parent_and_false_leaf_fail() {
    let policy = policy();
    let mut empty = source();
    empty.nodes.clear();
    assert_issue(project_branch_snapshot(&policy, &empty), BranchExplorerIssue::EmptyTree);

    let mut duplicate = source();
    duplicate.nodes.push(duplicate.nodes[0].clone());
    assert_issue(project_branch_snapshot(&policy, &duplicate), BranchExplorerIssue::DuplicateEntry);

    let mut missing_parent = source();
    missing_parent.nodes[1].parent_id = Some(String::from(UNKNOWN_ID));
    assert_issue(project_branch_snapshot(&policy, &missing_parent), BranchExplorerIssue::MissingParent);

    let mut multiple_parent = source();
    multiple_parent.nodes[1].extra_parent_id = Some(String::from(RIGHT_ID));
    assert_issue(project_branch_snapshot(&policy, &multiple_parent), BranchExplorerIssue::MultipleParents);

    let mut false_leaf = source();
    false_leaf.nodes[1].claimed_leaf = true;
    assert_issue(project_branch_snapshot(&policy, &false_leaf), BranchExplorerIssue::FalseLeaf);
}

#[test]
fn cycle_and_duplicate_sequence_fail_closed() {
    let policy = policy();
    let mut cycle = source();
    cycle.active_leaf_id = String::from(RIGHT_ID);
    cycle.nodes[1].parent_id = Some(String::from(LEFT_ID));
    cycle.nodes[1].claimed_leaf = false;
    cycle.nodes[2].parent_id = Some(String::from(INTERNAL_ID));
    cycle.nodes[2].claimed_leaf = false;
    cycle.nodes[3].parent_id = Some(String::from(ROOT_ID));
    assert_issue(project_branch_snapshot(&policy, &cycle), BranchExplorerIssue::Cycle);

    let mut duplicate_sequence = source();
    duplicate_sequence.nodes[1].storage_sequence = ROOT_SEQUENCE;
    assert_issue(project_branch_snapshot(&policy, &duplicate_sequence), BranchExplorerIssue::DuplicateSequence);
}

#[test]
fn stale_unknown_cross_parent_repeated_and_unsafe_annotations_fail() {
    let policy = policy();
    let mut stale = source();
    stale.annotations.revision_id = String::from(STALE_REVISION_ID);
    assert_issue(project_branch_snapshot(&policy, &stale), BranchExplorerIssue::StaleRevision);

    let mut unknown = source();
    unknown.annotations.bookmarks[0].target_entry_id = String::from(UNKNOWN_ID);
    assert_issue(project_branch_snapshot(&policy, &unknown), BranchExplorerIssue::UnknownTarget);

    let mut repeated = source();
    repeated.annotations.sibling_orders[0].ordered_child_ids.push(String::from(RIGHT_ID));
    assert_issue(project_branch_snapshot(&policy, &repeated), BranchExplorerIssue::InvalidSiblingOrder);

    let mut cross_parent = source();
    cross_parent.annotations.sibling_orders[0].ordered_child_ids.push(String::from(ROOT_ID));
    assert_issue(project_branch_snapshot(&policy, &cross_parent), BranchExplorerIssue::InvalidSiblingOrder);

    let mut unsafe_label = source();
    unsafe_label.annotations.labels[0].label = String::from("raw_prompt: private");
    assert_issue(project_branch_snapshot(&policy, &unsafe_label), BranchExplorerIssue::UnsafeLabel);
}

#[test]
fn bounded_projection_truncates_and_depth_bound_rejects() {
    let mut input = policy_input();
    input.max_nodes = INTERNAL_SEQUENCE;
    let bounded_policy = validate_branch_explorer_policy(&input).expect("bounded policy");
    let bounded = project_branch_snapshot(&bounded_policy, &source()).expect("bounded projection");
    assert_eq!(bounded.nodes.len(), usize::try_from(INTERNAL_SEQUENCE).expect("size"));
    assert_eq!(bounded.truncation.omitted_node_count, INTERNAL_SEQUENCE);

    let mut shallow_input = policy_input();
    shallow_input.max_depth = ROOT_SEQUENCE;
    let shallow = validate_branch_explorer_policy(&shallow_input).expect("shallow policy");
    assert_issue(project_branch_snapshot(&shallow, &source()), BranchExplorerIssue::BoundExceeded);
}

#[test]
fn missing_external_state_and_isolation_evidence_fail() {
    let policy = policy();
    let mut missing_state = source();
    missing_state.external_state = None;
    assert_issue(project_branch_snapshot(&policy, &missing_state), BranchExplorerIssue::MissingExternalState);

    let mut missing_evidence = source();
    missing_evidence.isolation_evidence_id = None;
    assert_issue(project_branch_snapshot(&policy, &missing_evidence), BranchExplorerIssue::MissingIsolationEvidence);
}

#[test]
fn receipt_is_stable_metadata_only_and_rejects_every_leak_class() {
    let policy = policy();
    let projection = projection();
    let operation =
        plan_branch_operation(&projection, BranchExplorerOperation::ActivateLeaf, REVISION_ID, LEFT_ID, RIGHT_ID);
    let comparison = compare_branches(&policy, &projection, LEFT_ID, RIGHT_ID).expect("comparison");
    let equivalence = validate_sibling_equivalence(&projection, LEFT_ID, RIGHT_ID, &generation(), &generation(), false)
        .expect("equivalence");
    let input = BranchExplorerReceiptInput {
        policy: &policy,
        projection: &projection,
        operation: Some(&operation),
        comparison: Some(&comparison),
        equivalence: Some(&equivalence),
        tui_action_id: Some("activate-action"),
        sensitive: BranchExplorerSensitiveDraft::default(),
        claims: BranchExplorerClaimDraft::default(),
    };
    let receipt = project_branch_explorer_receipt(&input).expect("receipt");
    let repeated = project_branch_explorer_receipt(&input).expect("repeated receipt");
    assert_eq!(receipt, repeated);
    assert_eq!(receipt.node_count, RIGHT_SEQUENCE);
    assert_eq!(receipt.redaction, "metadata_only_redaction_first");

    for sensitive in sensitive_drafts() {
        let leaking = BranchExplorerReceiptInput {
            sensitive,
            ..input.clone()
        };
        assert_issue(project_branch_explorer_receipt(&leaking), BranchExplorerIssue::ReceiptLeak);
    }

    let overclaim = BranchExplorerReceiptInput {
        claims: BranchExplorerClaimDraft {
            rollback: true,
            ..BranchExplorerClaimDraft::default()
        },
        ..input
    };
    assert_issue(project_branch_explorer_receipt(&overclaim), BranchExplorerIssue::Overclaim);
}

fn policy() -> BranchExplorerPolicy {
    validate_branch_explorer_policy(&policy_input()).expect("policy")
}

fn policy_input() -> BranchExplorerPolicyInput {
    BranchExplorerPolicyInput {
        schema: String::from(BRANCH_EXPLORER_POLICY_SCHEMA),
        max_nodes: MAX_NODES,
        max_depth: MAX_DEPTH,
        max_label_bytes: MAX_LABEL_BYTES,
        max_labels_per_node: MAX_LABELS_PER_NODE,
        max_ordered_children: MAX_ORDERED_CHILDREN,
        max_comparison_suffix: MAX_COMPARISON_SUFFIX,
        max_safe_references: MAX_SAFE_REFERENCES,
        redaction_first: true,
        one_replay_parent: true,
        annotations_are_sidecars: true,
        external_state_required: true,
        destructive_actions_allowed: false,
        receipt_metadata_only: true,
        non_claims: vec![
            String::from("transcript_truth"),
            String::from("effect_rollback"),
            String::from("world_isolation"),
            String::from("semantic_quality"),
            String::from("ui_delivery"),
            String::from("accessibility"),
            String::from("production_readiness"),
            String::from("release_eligibility"),
        ],
    }
}

fn projection() -> BranchProjectionSnapshot {
    project_branch_snapshot(&policy(), &source()).expect("projection")
}

fn source() -> BranchExplorerSourceInput {
    BranchExplorerSourceInput {
        session_id: String::from(SESSION_ID),
        revision_id: String::from(REVISION_ID),
        ledger_generation: LEDGER_GENERATION,
        active_leaf_id: String::from(LEFT_ID),
        nodes: vec![
            ledger_node(ROOT_ID, None, ROOT_SEQUENCE, BranchEntryClass::System, false, ROOT_USAGE),
            ledger_node(INTERNAL_ID, Some(ROOT_ID), INTERNAL_SEQUENCE, BranchEntryClass::User, false, INTERNAL_USAGE),
            ledger_node(LEFT_ID, Some(INTERNAL_ID), LEFT_SEQUENCE, BranchEntryClass::Assistant, true, LEAF_USAGE),
            ledger_node(RIGHT_ID, Some(INTERNAL_ID), RIGHT_SEQUENCE, BranchEntryClass::Assistant, true, LEAF_USAGE),
        ],
        annotations: BranchAnnotationsInput {
            revision_id: String::from(REVISION_ID),
            bookmarks: vec![BranchBookmarkInput {
                target_entry_id: String::from(RIGHT_ID),
                label: Some(String::from("candidate")),
            }],
            labels: vec![BranchLabelInput {
                target_entry_id: String::from(INTERNAL_ID),
                label: String::from("fork-point"),
            }],
            sibling_orders: vec![BranchSiblingOrderInput {
                parent_entry_id: String::from(INTERNAL_ID),
                ordered_child_ids: vec![String::from(RIGHT_ID)],
            }],
        },
        external_state: Some(ExternalStatePosture::Isolated),
        isolation_evidence_id: Some(String::from(ISOLATION_ID)),
    }
}

fn ledger_node(
    entry_id: &str,
    parent_id: Option<&str>,
    storage_sequence: u32,
    entry_class: BranchEntryClass,
    claimed_leaf: bool,
    usage_units: u64,
) -> BranchLedgerNodeInput {
    BranchLedgerNodeInput {
        entry_id: String::from(entry_id),
        parent_id: parent_id.map(String::from),
        extra_parent_id: None,
        entry_class,
        storage_sequence,
        claimed_leaf,
        artifact_reference_ids: vec![format!("artifact-{entry_id}")],
        effect_receipt_ids: if claimed_leaf {
            vec![String::from(EFFECT_ID)]
        } else {
            Vec::new()
        },
        model_profile_id: Some(String::from(POLICY_ID)),
        usage_units: Some(usage_units),
    }
}

fn generation() -> BranchGenerationInput {
    BranchGenerationInput {
        source_revision_id: String::from(REVISION_ID),
        fork_point_id: String::from(INTERNAL_ID),
        model_profile_id: String::from(POLICY_ID),
        parameter_profile_id: String::from(PARAMETER_ID),
        tool_plan_id: String::from(TOOL_PLAN_ID),
        effect_receipt_ids: vec![String::from(EFFECT_ID)],
        response_artifact_id: String::from(RESPONSE_ID),
    }
}

fn conversation(leaf_hash: &str, generation: u64) -> ConversationProjectionSnapshot {
    ConversationProjectionSnapshot {
        stream_hash: String::from("stream"),
        items: Vec::new(),
        cursor: ConversationProjectionCursor {
            stream_hash: String::from("stream"),
            policy_hash: String::from("conversation-policy"),
            session_id: String::from(SESSION_ID),
            branch_id: String::from("branch"),
            leaf_hash: String::from(leaf_hash),
            generation,
            last_entry_hash: String::from(leaf_hash),
            last_sequence: u64::from(RIGHT_SEQUENCE),
            active_turn_hash: None,
            cursor_hash: String::from("cursor"),
        },
        fold_count: 0,
        steering_count: 0,
        snapshot_hash: String::from("conversation-snapshot"),
    }
}

fn node<'a>(projection: &'a BranchProjectionSnapshot, entry_id: &str) -> &'a BranchProjectionNode {
    projection.nodes.iter().find(|node| node.entry_id == entry_id).expect("projected node")
}

fn sensitive_drafts() -> [BranchExplorerSensitiveDraft<'static>; 8] {
    [
        BranchExplorerSensitiveDraft {
            transcript_text: Some("private transcript"),
            ..BranchExplorerSensitiveDraft::default()
        },
        BranchExplorerSensitiveDraft {
            reasoning: Some("private reasoning"),
            ..BranchExplorerSensitiveDraft::default()
        },
        BranchExplorerSensitiveDraft {
            tool_body: Some("private tool body"),
            ..BranchExplorerSensitiveDraft::default()
        },
        BranchExplorerSensitiveDraft {
            provider_payload: Some("private provider payload"),
            ..BranchExplorerSensitiveDraft::default()
        },
        BranchExplorerSensitiveDraft {
            credential: Some("private credential"),
            ..BranchExplorerSensitiveDraft::default()
        },
        BranchExplorerSensitiveDraft {
            environment_value: Some("private environment"),
            ..BranchExplorerSensitiveDraft::default()
        },
        BranchExplorerSensitiveDraft {
            physical_path: Some("/home/private"),
            ..BranchExplorerSensitiveDraft::default()
        },
        BranchExplorerSensitiveDraft {
            raw_ui_state: Some("private UI state"),
            ..BranchExplorerSensitiveDraft::default()
        },
    ]
}

fn assert_issue<T>(result: Result<T, BranchExplorerError>, issue: BranchExplorerIssue) {
    let error = result.err().expect("expected failure");
    assert_eq!(error.issue(), issue);
}
