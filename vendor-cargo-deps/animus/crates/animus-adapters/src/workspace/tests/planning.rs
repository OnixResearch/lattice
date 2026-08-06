use super::support::*;

#[test]
fn plan_accepts_leaf_commit_and_receipt_hashes_are_stable() {
    let parent = descriptor(PARENT_ID, None, OP_CREATE, &[CAP_NESTED, CAP_COMMIT]);
    let child = descriptor(WORKSPACE_ID, Some(PARENT_ID), OP_COMMIT, &[CAP_NESTED, CAP_COMMIT]);
    let adapter = branchfs_available(&[
        CAP_BRANCHFS,
        CAP_NESTED,
        CAP_COMMIT,
        CAP_ABORT,
        CAP_CONFLICT,
        CAP_CLEANUP,
    ]);
    let descriptors = [parent, child];
    let first_plan = crate::evaluate_workspace_plan(crate::WorkspacePlanRequest {
        descriptors: &descriptors,
        selected_workspace_id: WORKSPACE_ID,
        adapter: &adapter,
    });
    let second_plan = crate::evaluate_workspace_plan(crate::WorkspacePlanRequest {
        descriptors: &descriptors,
        selected_workspace_id: WORKSPACE_ID,
        adapter: &adapter,
    });
    let outcome = crate::WorkspaceAdapterOutcome::new(crate::WorkspaceAdapterOutcomeDraft {
        kind: OUTCOME_COMMITTED,
        workspace_id: WORKSPACE_ID,
        issue_code: None,
    })
    .expect("committed outcome should parse");
    let receipt = crate::WorkspaceReceipt::new(crate::WorkspaceReceiptDraft {
        plan: &first_plan,
        outcome: Some(&outcome),
        manifest_labels: &[SAFE_MANIFEST_LABEL],
    });

    assert!(first_plan.dispatch_allowed());
    assert_eq!(first_plan.status(), crate::WorkspacePlanStatus::Ready);
    assert_eq!(first_plan.issue(), crate::WorkspacePlanIssue::None);
    assert_eq!(first_plan.parent_workspace_id(), Some(PARENT_ID));
    assert_eq!(first_plan.descriptor_hash(), second_plan.descriptor_hash());
    assert_eq!(receipt.status(), crate::WorkspacePlanStatus::Ready);
    assert_eq!(receipt.outcome_kind(), Some(crate::WorkspaceAdapterOutcomeKind::Committed));
    assert_eq!(receipt.manifest_count(), 1);
    assert_eq!(receipt.capability_labels().len(), first_plan.required_capabilities().len());
    assert_eq!(receipt.descriptor_hash(), first_plan.descriptor_hash());
}

#[test]
fn plan_accepts_leaf_abort_without_touching_siblings() {
    let parent = descriptor(PARENT_ID, None, OP_CREATE, &[CAP_NESTED, CAP_ABORT]);
    let child = descriptor(WORKSPACE_ID, Some(PARENT_ID), OP_ABORT, &[CAP_NESTED, CAP_ABORT]);
    let sibling = descriptor("ws:sibling", Some(PARENT_ID), OP_CREATE, &[CAP_NESTED]);
    let adapter = branchfs_available(&[CAP_BRANCHFS, CAP_NESTED, CAP_ABORT, CAP_CLEANUP]);
    let descriptors = [parent, child, sibling];
    let plan = crate::evaluate_workspace_plan(crate::WorkspacePlanRequest {
        descriptors: &descriptors,
        selected_workspace_id: WORKSPACE_ID,
        adapter: &adapter,
    });
    let outcome = crate::WorkspaceAdapterOutcome::new(crate::WorkspaceAdapterOutcomeDraft {
        kind: OUTCOME_ABORTED,
        workspace_id: WORKSPACE_ID,
        issue_code: None,
    })
    .expect("aborted outcome should parse");
    let receipt = crate::WorkspaceReceipt::new(crate::WorkspaceReceiptDraft {
        plan: &plan,
        outcome: Some(&outcome),
        manifest_labels: &[],
    });

    assert!(plan.dispatch_allowed());
    assert_eq!(plan.issue(), crate::WorkspacePlanIssue::None);
    assert_eq!(receipt.outcome_kind(), Some(crate::WorkspaceAdapterOutcomeKind::Aborted));
}

#[test]
fn plan_rejects_invalid_graphs_before_adapter_dispatch() {
    let duplicate = descriptor(WORKSPACE_ID, Some(PARENT_ID), OP_CREATE, &[CAP_NESTED]);
    let duplicate_graph = [duplicate.clone(), duplicate];
    let adapter = branchfs_available(&[CAP_BRANCHFS, CAP_NESTED]);
    let duplicate_plan = crate::evaluate_workspace_plan(crate::WorkspacePlanRequest {
        descriptors: &duplicate_graph,
        selected_workspace_id: WORKSPACE_ID,
        adapter: &adapter,
    });
    assert_eq!(duplicate_plan.issue(), crate::WorkspacePlanIssue::DuplicateWorkspace);
    assert!(!duplicate_plan.dispatch_allowed());

    let missing_parent = descriptor(WORKSPACE_ID, Some(PARENT_ID), OP_CREATE, &[CAP_NESTED]);
    let missing_parent_graph = [missing_parent];
    let missing_parent_plan = crate::evaluate_workspace_plan(crate::WorkspacePlanRequest {
        descriptors: &missing_parent_graph,
        selected_workspace_id: WORKSPACE_ID,
        adapter: &adapter,
    });
    assert_eq!(missing_parent_plan.issue(), crate::WorkspacePlanIssue::MissingParent);

    let cycle_parent = descriptor(PARENT_ID, Some(WORKSPACE_ID), OP_CREATE, &[CAP_NESTED]);
    let cycle_child = descriptor(WORKSPACE_ID, Some(PARENT_ID), OP_CREATE, &[CAP_NESTED]);
    let cycle_graph = [cycle_parent, cycle_child];
    let cycle_plan = crate::evaluate_workspace_plan(crate::WorkspacePlanRequest {
        descriptors: &cycle_graph,
        selected_workspace_id: WORKSPACE_ID,
        adapter: &adapter,
    });
    assert_eq!(cycle_plan.issue(), crate::WorkspacePlanIssue::ParentCycle);
}

#[test]
fn plan_rejects_non_leaf_mutation_and_capability_gaps() {
    let root = descriptor(ROOT_PARENT_ID, None, OP_CREATE, &[CAP_NESTED]);
    let parent = descriptor(PARENT_ID, Some(ROOT_PARENT_ID), OP_COMMIT, &[CAP_NESTED, CAP_COMMIT]);
    let child = descriptor(WORKSPACE_ID, Some(PARENT_ID), OP_CREATE, &[CAP_NESTED]);
    let adapter = branchfs_available(&[CAP_BRANCHFS, CAP_NESTED, CAP_COMMIT]);
    let descriptors = [root, parent, child];
    let non_leaf_plan = crate::evaluate_workspace_plan(crate::WorkspacePlanRequest {
        descriptors: &descriptors,
        selected_workspace_id: PARENT_ID,
        adapter: &adapter,
    });
    assert_eq!(non_leaf_plan.issue(), crate::WorkspacePlanIssue::NonLeafCommit);
    assert!(!non_leaf_plan.dispatch_allowed());

    let requires_cleanup = descriptor(WORKSPACE_ID, None, OP_CREATE, &[CAP_CLEANUP]);
    let capability_gap_graph = [requires_cleanup];
    let missing_capability_adapter = branchfs_available(&[CAP_BRANCHFS]);
    let capability_gap_plan = crate::evaluate_workspace_plan(crate::WorkspacePlanRequest {
        descriptors: &capability_gap_graph,
        selected_workspace_id: WORKSPACE_ID,
        adapter: &missing_capability_adapter,
    });
    let receipt = crate::WorkspaceReceipt::new(crate::WorkspaceReceiptDraft {
        plan: &capability_gap_plan,
        outcome: None,
        manifest_labels: &[SAFE_MANIFEST_LABEL],
    });
    assert_eq!(capability_gap_plan.issue(), crate::WorkspacePlanIssue::UnsupportedCapability);
    assert_eq!(capability_gap_plan.unsupported_capabilities(), &[crate::WorkspaceCapability::CleanupGuarantee]);
    assert_eq!(receipt.unsupported_capability_labels(), &[alloc::string::String::from(CAP_CLEANUP)]);
}

#[test]
fn plan_fails_closed_for_unavailable_partial_or_placeholder_adapters() {
    let descriptor = descriptor(WORKSPACE_ID, None, OP_CREATE, NO_CAPS);
    let descriptors = [descriptor];
    let unavailable = branchfs_unavailable();
    let partial = branchfs_partial();
    let placeholder = placeholder_available();

    let unavailable_plan = crate::evaluate_workspace_plan(crate::WorkspacePlanRequest {
        descriptors: &descriptors,
        selected_workspace_id: WORKSPACE_ID,
        adapter: &unavailable,
    });
    let partial_plan = crate::evaluate_workspace_plan(crate::WorkspacePlanRequest {
        descriptors: &descriptors,
        selected_workspace_id: WORKSPACE_ID,
        adapter: &partial,
    });
    let placeholder_plan = crate::evaluate_workspace_plan(crate::WorkspacePlanRequest {
        descriptors: &descriptors,
        selected_workspace_id: WORKSPACE_ID,
        adapter: &placeholder,
    });

    assert_eq!(unavailable_plan.issue(), crate::WorkspacePlanIssue::AdapterUnavailable);
    assert_eq!(partial_plan.issue(), crate::WorkspacePlanIssue::AdapterPartiallyEnforceable);
    assert_eq!(placeholder_plan.issue(), crate::WorkspacePlanIssue::AdapterUnsupported);
    assert!(!unavailable_plan.dispatch_allowed());
    assert!(!partial_plan.dispatch_allowed());
    assert!(!placeholder_plan.dispatch_allowed());
}
