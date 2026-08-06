// This archived Cairn adaptation keeps explicit topology facts and bounded validation branches.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    borrowed_argument_types,
    compound_condition,
    fallible_int_fallback,
    function_length,
    no_recursion,
    non_trait_imports,
    numeric_units,
    path_segment_repetition,
    sentinel_fallback,
    too_many_parameters,
    unbounded_collection_growth
)]
#![allow(clippy::too_many_arguments, clippy::missing_errors_doc)]

use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[allow(clippy::wildcard_imports)]
use super::data::*;
use crate::branching::SessionBranchOperation;

const HARD_MAX_NODES: u32 = 16_384;
const HARD_MAX_DEPTH: u32 = 1_024;
const HARD_MAX_LABEL_BYTES: u32 = 256;
const HARD_MAX_LABELS_PER_NODE: u32 = 32;
const HARD_MAX_ORDERED_CHILDREN: u32 = 4_096;
const HARD_MAX_COMPARISON_SUFFIX: u32 = 4_096;
const HARD_MAX_SAFE_REFERENCES: u32 = 256;
const EXPECTED_NON_CLAIM_COUNT: usize = 8;

const REQUIRED_NON_CLAIMS: &[&str] = &[
    "transcript_truth",
    "effect_rollback",
    "world_isolation",
    "semantic_quality",
    "ui_delivery",
    "accessibility",
    "production_readiness",
    "release_eligibility",
];

const UNSAFE_TEXT_MARKERS: &[&str] = &[
    "raw_prompt:",
    "raw transcript",
    "reasoning:",
    "tool_output:",
    "provider payload",
    "provider_json",
    "authorization:",
    "api_key",
    "secret=",
    "/home/",
    "../",
    "environment=",
];

/// Validate generated policy and derive its domain-separated identity.
pub fn validate_branch_explorer_policy(
    input: &BranchExplorerPolicyInput,
) -> Result<BranchExplorerPolicy, BranchExplorerError> {
    let schema_ok = input.schema == BRANCH_EXPLORER_POLICY_SCHEMA;
    let positive_bounds = input.max_nodes > 0
        && input.max_depth > 0
        && input.max_label_bytes > 0
        && input.max_labels_per_node > 0
        && input.max_ordered_children > 0
        && input.max_comparison_suffix > 0
        && input.max_safe_references > 0;
    let bounded = input.max_nodes <= HARD_MAX_NODES
        && input.max_depth <= HARD_MAX_DEPTH
        && input.max_label_bytes <= HARD_MAX_LABEL_BYTES
        && input.max_labels_per_node <= HARD_MAX_LABELS_PER_NODE
        && input.max_ordered_children <= HARD_MAX_ORDERED_CHILDREN
        && input.max_comparison_suffix <= HARD_MAX_COMPARISON_SUFFIX
        && input.max_safe_references <= HARD_MAX_SAFE_REFERENCES;
    let posture_ok = input.redaction_first
        && input.one_replay_parent
        && input.annotations_are_sidecars
        && input.external_state_required
        && !input.destructive_actions_allowed
        && input.receipt_metadata_only;
    let non_claims_ok = REQUIRED_NON_CLAIMS
        .iter()
        .all(|required| input.non_claims.iter().any(|candidate| candidate == required))
        && input.non_claims.len() == EXPECTED_NON_CLAIM_COUNT;
    if !schema_ok || !positive_bounds || !bounded || !posture_ok || !non_claims_ok {
        return Err(BranchExplorerError::new(BranchExplorerIssue::InvalidPolicy, "policy"));
    }
    let mut parts = vec![
        input.schema.clone(),
        input.max_nodes.to_string(),
        input.max_depth.to_string(),
        input.max_label_bytes.to_string(),
        input.max_labels_per_node.to_string(),
        input.max_ordered_children.to_string(),
        input.max_comparison_suffix.to_string(),
        input.max_safe_references.to_string(),
    ];
    parts.extend(input.non_claims.iter().cloned());
    Ok(BranchExplorerPolicy {
        schema: input.schema.clone(),
        policy_id: hash_parts(BRANCH_EXPLORER_POLICY_DOMAIN, parts.iter().map(String::as_str)),
        max_nodes: input.max_nodes,
        max_depth: input.max_depth,
        max_label_bytes: input.max_label_bytes,
        max_labels_per_node: input.max_labels_per_node,
        max_ordered_children: input.max_ordered_children,
        max_comparison_suffix: input.max_comparison_suffix,
        max_safe_references: input.max_safe_references,
        non_claims: input.non_claims.clone(),
    })
}

/// Project one complete validated ledger snapshot into bounded topology facts.
pub fn project_branch_snapshot(
    policy: &BranchExplorerPolicy,
    source: &BranchExplorerSourceInput,
) -> Result<BranchProjectionSnapshot, BranchExplorerError> {
    if source.nodes.is_empty() {
        return Err(BranchExplorerError::new(BranchExplorerIssue::EmptyTree, "nodes"));
    }
    safe_identity(source.session_id.as_str(), "session_id")?;
    safe_identity(source.revision_id.as_str(), "revision_id")?;
    safe_identity(source.active_leaf_id.as_str(), "active_leaf_id")?;
    let external_state = source
        .external_state
        .ok_or_else(|| BranchExplorerError::new(BranchExplorerIssue::MissingExternalState, "external_state"))?;
    validate_isolation(source, external_state)?;

    let indexes = build_indexes(source)?;
    validate_topology(policy, source, &indexes)?;
    let annotations = validate_annotations(policy, source, &indexes)?;
    let active_path = collect_ancestor_set(source.active_leaf_id.as_str(), &indexes.by_id)?;
    let presentation_order = presentation_order(source, &indexes, &annotations.orders)?;
    let admitted_count = usize::try_from(policy.max_nodes)
        .map_err(|_error| BranchExplorerError::new(BranchExplorerIssue::BoundExceeded, "max_nodes"))?;
    let admitted_ids: BTreeSet<&str> = presentation_order.iter().take(admitted_count).map(String::as_str).collect();
    let omitted_node_count = u32::try_from(source.nodes.len().saturating_sub(admitted_ids.len()))
        .map_err(|_error| BranchExplorerError::new(BranchExplorerIssue::BoundExceeded, "nodes"))?;

    let mut nodes = Vec::with_capacity(admitted_ids.len());
    for entry_id in presentation_order.iter().take(admitted_count) {
        let source_node = indexes
            .by_id
            .get(entry_id.as_str())
            .ok_or_else(|| BranchExplorerError::new(BranchExplorerIssue::UnknownTarget, "presentation_order"))?;
        let child_count = indexes.children.get(entry_id.as_str()).map_or(0, Vec::len);
        let child_count = u32::try_from(child_count)
            .map_err(|_error| BranchExplorerError::new(BranchExplorerIssue::BoundExceeded, "child_count"))?;
        nodes.push(BranchProjectionNode {
            entry_id: source_node.entry_id.clone(),
            parent_id: source_node.parent_id.clone(),
            entry_class: source_node.entry_class,
            storage_sequence: source_node.storage_sequence,
            depth: depth_of(source_node.entry_id.as_str(), &indexes.by_id)?,
            child_count,
            is_leaf: child_count == 0,
            is_active_path: active_path.contains(source_node.entry_id.as_str()),
            is_bookmarked: annotations.bookmarks.contains(source_node.entry_id.as_str()),
            labels: annotations.labels.get(source_node.entry_id.as_str()).cloned().unwrap_or_default(),
            artifact_reference_ids: source_node.artifact_reference_ids.clone(),
            effect_receipt_ids: source_node.effect_receipt_ids.clone(),
            model_profile_id: source_node.model_profile_id.clone(),
            usage_units: source_node.usage_units,
        });
    }
    let admitted_order = presentation_order.into_iter().take(admitted_count).collect::<Vec<_>>();
    let projection_id = projection_identity(policy, source, external_state, nodes.as_slice(), omitted_node_count);
    Ok(BranchProjectionSnapshot {
        schema: String::from(BRANCH_EXPLORER_PROJECTION_SCHEMA),
        projection_id,
        policy_id: policy.policy_id.clone(),
        session_id: source.session_id.clone(),
        revision_id: source.revision_id.clone(),
        ledger_generation: source.ledger_generation,
        active_leaf_id: source.active_leaf_id.clone(),
        external_state,
        isolation_evidence_id: source.isolation_evidence_id.clone(),
        nodes,
        presentation_order: admitted_order,
        truncation: BranchProjectionTruncation {
            omitted_node_count,
            node_limit: policy.max_nodes,
        },
    })
}

/// Plan inspect, leaf activation, or append-only fork without host effects.
#[must_use]
pub fn plan_branch_operation(
    projection: &BranchProjectionSnapshot,
    operation: BranchExplorerOperation,
    expected_revision_id: &str,
    expected_active_leaf_id: &str,
    target_entry_id: &str,
) -> BranchOperationPlan {
    let base = PlanBase {
        projection,
        operation,
        expected_revision_id,
        expected_active_leaf_id,
        target_entry_id,
    };
    if projection.revision_id != expected_revision_id || projection.active_leaf_id != expected_active_leaf_id {
        return blocked_plan(base, BranchExplorerIssue::StaleRevision);
    }
    let Some(target) = projection.nodes.iter().find(|node| node.entry_id == target_entry_id) else {
        return blocked_plan(base, BranchExplorerIssue::UnknownTarget);
    };
    if operation == BranchExplorerOperation::ActivateLeaf && !target.is_leaf {
        return blocked_plan(base, BranchExplorerIssue::NonLeafActivation);
    }
    let mut continuation_independent = false;
    if operation == BranchExplorerOperation::ForkFromNode {
        continuation_independent =
            projection.external_state == ExternalStatePosture::Isolated && projection.isolation_evidence_id.is_some();
    }
    let append_only_branch_operation = if operation == BranchExplorerOperation::ForkFromNode {
        Some(SessionBranchOperation::ForkFromMessage)
    } else {
        None
    };
    admitted_plan(base, continuation_independent, append_only_branch_operation)
}

/// Compare two admitted targets through their exact common ancestor.
pub fn compare_branches(
    policy: &BranchExplorerPolicy,
    projection: &BranchProjectionSnapshot,
    left_target_id: &str,
    right_target_id: &str,
) -> Result<BranchComparison, BranchExplorerError> {
    if projection.policy_id != policy.policy_id {
        return Err(BranchExplorerError::new(BranchExplorerIssue::StaleRevision, "policy_id"));
    }
    let by_id = projection.nodes.iter().map(|node| (node.entry_id.as_str(), node)).collect::<BTreeMap<_, _>>();
    let left_path = projection_path(left_target_id, &by_id)?;
    let right_path = projection_path(right_target_id, &by_id)?;
    let common_ancestor_id = left_path
        .iter()
        .zip(right_path.iter())
        .take_while(|(left, right)| left == right)
        .map(|(left, _right)| *left)
        .last()
        .ok_or_else(|| BranchExplorerError::new(BranchExplorerIssue::MissingCommonAncestor, "comparison"))?;
    let left_suffix = suffix_after(common_ancestor_id, left_path.as_slice());
    let right_suffix = suffix_after(common_ancestor_id, right_path.as_slice());
    let suffix_limit = usize::try_from(policy.max_comparison_suffix)
        .map_err(|_error| BranchExplorerError::new(BranchExplorerIssue::BoundExceeded, "comparison"))?;
    let truncated = left_suffix.len() > suffix_limit || right_suffix.len() > suffix_limit;
    let left_suffix_ids = left_suffix.iter().take(suffix_limit).map(|value| String::from(*value)).collect::<Vec<_>>();
    let right_suffix_ids = right_suffix.iter().take(suffix_limit).map(|value| String::from(*value)).collect::<Vec<_>>();
    let left_nodes = nodes_for_ids(left_suffix_ids.as_slice(), &by_id)?;
    let right_nodes = nodes_for_ids(right_suffix_ids.as_slice(), &by_id)?;
    let comparison_id = hash_parts("animus.session-branch-explorer.comparison.v1", [
        projection.projection_id.as_str(),
        left_target_id,
        right_target_id,
        common_ancestor_id,
    ]);
    Ok(BranchComparison {
        comparison_id,
        session_id: projection.session_id.clone(),
        revision_id: projection.revision_id.clone(),
        common_ancestor_id: String::from(common_ancestor_id),
        left_target_id: String::from(left_target_id),
        right_target_id: String::from(right_target_id),
        left_suffix_ids,
        right_suffix_ids,
        left_class_counts: class_counts(left_nodes.as_slice()),
        right_class_counts: class_counts(right_nodes.as_slice()),
        left_usage_units: usage_sum(left_nodes.as_slice()),
        right_usage_units: usage_sum(right_nodes.as_slice()),
        unavailable_facts: vec![
            String::from("semantic_quality"),
            String::from("correctness"),
            String::from("winner"),
        ],
        truncated,
    })
}

/// Derive a complete domain-separated generation identity.
pub fn generation_identity(input: &BranchGenerationInput) -> Result<String, BranchExplorerError> {
    for value in [
        input.source_revision_id.as_str(),
        input.fork_point_id.as_str(),
        input.model_profile_id.as_str(),
        input.parameter_profile_id.as_str(),
        input.tool_plan_id.as_str(),
        input.response_artifact_id.as_str(),
    ] {
        if value.is_empty() {
            return Err(BranchExplorerError::new(
                BranchExplorerIssue::IncompleteGenerationIdentity,
                "generation_identity",
            ));
        }
        safe_identity(value, "generation_identity")?;
    }
    let mut effects = input.effect_receipt_ids.clone();
    effects.sort();
    if effects.windows(2).any(|window| window[0] == window[1]) {
        return Err(BranchExplorerError::new(BranchExplorerIssue::IncompleteGenerationIdentity, "effect_receipt_ids"));
    }
    let mut parts = vec![
        input.source_revision_id.as_str(),
        input.fork_point_id.as_str(),
        input.model_profile_id.as_str(),
        input.parameter_profile_id.as_str(),
        input.tool_plan_id.as_str(),
        input.response_artifact_id.as_str(),
    ];
    parts.extend(effects.iter().map(String::as_str));
    Ok(hash_parts(BRANCH_EXPLORER_GENERATION_DOMAIN, parts))
}

/// Validate exact sibling-leaf equivalence without deletion or rewrite actions.
pub fn validate_sibling_equivalence(
    projection: &BranchProjectionSnapshot,
    left_leaf_id: &str,
    right_leaf_id: &str,
    left_generation: &BranchGenerationInput,
    right_generation: &BranchGenerationInput,
    destructive_action_requested: bool,
) -> Result<BranchSiblingEquivalence, BranchExplorerError> {
    if destructive_action_requested {
        return Err(BranchExplorerError::new(BranchExplorerIssue::DestructiveAction, "equivalence"));
    }
    let left = projection
        .nodes
        .iter()
        .find(|node| node.entry_id == left_leaf_id)
        .ok_or_else(|| BranchExplorerError::new(BranchExplorerIssue::UnknownTarget, "left_leaf_id"))?;
    let right = projection
        .nodes
        .iter()
        .find(|node| node.entry_id == right_leaf_id)
        .ok_or_else(|| BranchExplorerError::new(BranchExplorerIssue::UnknownTarget, "right_leaf_id"))?;
    let siblings = left.is_leaf
        && right.is_leaf
        && left.entry_id != right.entry_id
        && left.parent_id.is_some()
        && left.parent_id == right.parent_id;
    if !siblings {
        return Err(BranchExplorerError::new(BranchExplorerIssue::NotSiblingLeaves, "equivalence"));
    }
    let left_generation_id = generation_identity(left_generation)?;
    let right_generation_id = generation_identity(right_generation)?;
    if left_generation_id != right_generation_id {
        return Err(BranchExplorerError::new(BranchExplorerIssue::GenerationMismatch, "generation_identity"));
    }
    let parent_entry_id = left
        .parent_id
        .clone()
        .ok_or_else(|| BranchExplorerError::new(BranchExplorerIssue::NotSiblingLeaves, "parent_entry_id"))?;
    Ok(BranchSiblingEquivalence {
        equivalence_id: hash_parts("animus.session-branch-explorer.equivalence.v1", [
            projection.projection_id.as_str(),
            parent_entry_id.as_str(),
            left_leaf_id,
            right_leaf_id,
            left_generation_id.as_str(),
        ]),
        parent_entry_id,
        left_leaf_id: String::from(left_leaf_id),
        right_leaf_id: String::from(right_leaf_id),
        generation_id: left_generation_id,
        branches_preserved: true,
    })
}

struct Indexes<'a> {
    by_id: BTreeMap<&'a str, &'a BranchLedgerNodeInput>,
    children: BTreeMap<&'a str, Vec<&'a str>>,
    roots: Vec<&'a str>,
}

struct ValidAnnotations<'a> {
    bookmarks: BTreeSet<&'a str>,
    labels: BTreeMap<&'a str, Vec<String>>,
    orders: BTreeMap<&'a str, Vec<&'a str>>,
}

fn build_indexes(source: &BranchExplorerSourceInput) -> Result<Indexes<'_>, BranchExplorerError> {
    let mut by_id = BTreeMap::new();
    let mut sequences = BTreeSet::new();
    for node in &source.nodes {
        safe_identity(node.entry_id.as_str(), "entry_id")?;
        if node.extra_parent_id.is_some() {
            return Err(BranchExplorerError::new(BranchExplorerIssue::MultipleParents, "extra_parent_id"));
        }
        if by_id.insert(node.entry_id.as_str(), node).is_some() {
            return Err(BranchExplorerError::new(BranchExplorerIssue::DuplicateEntry, "entry_id"));
        }
        if !sequences.insert(node.storage_sequence) {
            return Err(BranchExplorerError::new(BranchExplorerIssue::DuplicateSequence, "storage_sequence"));
        }
    }
    let mut children = BTreeMap::<&str, Vec<&str>>::new();
    let mut roots = Vec::new();
    for node in &source.nodes {
        if let Some(parent_id) = node.parent_id.as_deref() {
            if !by_id.contains_key(parent_id) {
                return Err(BranchExplorerError::new(BranchExplorerIssue::MissingParent, "parent_id"));
            }
            children.entry(parent_id).or_default().push(node.entry_id.as_str());
        } else {
            roots.push(node.entry_id.as_str());
        }
    }
    for child_ids in children.values_mut() {
        child_ids.sort_by_key(|child_id| by_id[*child_id].storage_sequence);
    }
    roots.sort_by_key(|root_id| by_id[*root_id].storage_sequence);
    Ok(Indexes { by_id, children, roots })
}

fn validate_topology(
    policy: &BranchExplorerPolicy,
    source: &BranchExplorerSourceInput,
    indexes: &Indexes<'_>,
) -> Result<(), BranchExplorerError> {
    if indexes.roots.len() != 1 {
        return Err(BranchExplorerError::new(BranchExplorerIssue::MissingParent, "root"));
    }
    if !indexes.by_id.contains_key(source.active_leaf_id.as_str()) {
        return Err(BranchExplorerError::new(BranchExplorerIssue::UnknownTarget, "active_leaf_id"));
    }
    let maximum_source_nodes = usize::try_from(HARD_MAX_NODES)
        .map_err(|_error| BranchExplorerError::new(BranchExplorerIssue::BoundExceeded, "nodes"))?;
    if source.nodes.len() > maximum_source_nodes {
        return Err(BranchExplorerError::new(BranchExplorerIssue::BoundExceeded, "nodes"));
    }
    for node in &source.nodes {
        let depth = depth_of(node.entry_id.as_str(), &indexes.by_id)?;
        if depth > policy.max_depth {
            return Err(BranchExplorerError::new(BranchExplorerIssue::BoundExceeded, "depth"));
        }
        let computed_leaf = indexes.children.get(node.entry_id.as_str()).is_none_or(Vec::is_empty);
        if computed_leaf != node.claimed_leaf {
            return Err(BranchExplorerError::new(BranchExplorerIssue::FalseLeaf, "claimed_leaf"));
        }
        validate_references(policy, node)?;
    }
    let active = indexes.by_id[source.active_leaf_id.as_str()];
    if !active.claimed_leaf {
        return Err(BranchExplorerError::new(BranchExplorerIssue::FalseLeaf, "active_leaf_id"));
    }
    Ok(())
}

fn validate_references(policy: &BranchExplorerPolicy, node: &BranchLedgerNodeInput) -> Result<(), BranchExplorerError> {
    let reference_count = node.artifact_reference_ids.len().saturating_add(node.effect_receipt_ids.len());
    let reference_limit = usize::try_from(policy.max_safe_references)
        .map_err(|_error| BranchExplorerError::new(BranchExplorerIssue::BoundExceeded, "safe_references"))?;
    if reference_count > reference_limit {
        return Err(BranchExplorerError::new(BranchExplorerIssue::BoundExceeded, "safe_references"));
    }
    for reference in node.artifact_reference_ids.iter().chain(node.effect_receipt_ids.iter()) {
        safe_identity(reference.as_str(), "safe_reference")?;
    }
    if let Some(model_profile_id) = node.model_profile_id.as_deref() {
        safe_identity(model_profile_id, "model_profile_id")?;
    }
    Ok(())
}

fn validate_annotations<'a>(
    policy: &BranchExplorerPolicy,
    source: &'a BranchExplorerSourceInput,
    indexes: &Indexes<'a>,
) -> Result<ValidAnnotations<'a>, BranchExplorerError> {
    if source.annotations.revision_id != source.revision_id {
        return Err(BranchExplorerError::new(BranchExplorerIssue::StaleRevision, "annotations.revision_id"));
    }
    let mut bookmarks = BTreeSet::new();
    let mut labels = BTreeMap::<&str, Vec<String>>::new();
    for bookmark in &source.annotations.bookmarks {
        let target = known_target(bookmark.target_entry_id.as_str(), indexes)?;
        if !bookmarks.insert(target) {
            return Err(BranchExplorerError::new(BranchExplorerIssue::InvalidSiblingOrder, "bookmarks"));
        }
        if let Some(label) = bookmark.label.as_deref() {
            validate_label(policy, label)?;
            labels.entry(target).or_default().push(String::from(label));
        }
    }
    for label in &source.annotations.labels {
        let target = known_target(label.target_entry_id.as_str(), indexes)?;
        validate_label(policy, label.label.as_str())?;
        labels.entry(target).or_default().push(label.label.clone());
    }
    let label_limit = usize::try_from(policy.max_labels_per_node)
        .map_err(|_error| BranchExplorerError::new(BranchExplorerIssue::BoundExceeded, "labels"))?;
    if labels.values().any(|node_labels| node_labels.len() > label_limit) {
        return Err(BranchExplorerError::new(BranchExplorerIssue::BoundExceeded, "labels"));
    }
    let mut orders = BTreeMap::new();
    for order in &source.annotations.sibling_orders {
        let parent = known_target(order.parent_entry_id.as_str(), indexes)?;
        if orders.contains_key(parent) {
            return Err(BranchExplorerError::new(BranchExplorerIssue::InvalidSiblingOrder, "sibling_orders"));
        }
        let order_limit = usize::try_from(policy.max_ordered_children)
            .map_err(|_error| BranchExplorerError::new(BranchExplorerIssue::BoundExceeded, "sibling_order"))?;
        if order.ordered_child_ids.len() > order_limit {
            return Err(BranchExplorerError::new(BranchExplorerIssue::BoundExceeded, "sibling_order"));
        }
        let canonical_children = indexes.children.get(parent).map(Vec::as_slice).unwrap_or_default();
        let mut seen = BTreeSet::new();
        let mut admitted = Vec::new();
        for child_id in &order.ordered_child_ids {
            let child = known_target(child_id.as_str(), indexes)?;
            if !canonical_children.contains(&child) || !seen.insert(child) {
                return Err(BranchExplorerError::new(BranchExplorerIssue::InvalidSiblingOrder, "sibling_order"));
            }
            admitted.push(child);
        }
        orders.insert(parent, admitted);
    }
    Ok(ValidAnnotations {
        bookmarks,
        labels,
        orders,
    })
}

fn validate_label(policy: &BranchExplorerPolicy, label: &str) -> Result<(), BranchExplorerError> {
    let label_limit = usize::try_from(policy.max_label_bytes)
        .map_err(|_error| BranchExplorerError::new(BranchExplorerIssue::BoundExceeded, "label"))?;
    let safe = !label.is_empty()
        && label.len() <= label_limit
        && !label.chars().any(char::is_control)
        && !UNSAFE_TEXT_MARKERS.iter().any(|marker| label.to_ascii_lowercase().contains(marker));
    if !safe {
        return Err(BranchExplorerError::new(BranchExplorerIssue::UnsafeLabel, "label"));
    }
    Ok(())
}

fn presentation_order(
    source: &BranchExplorerSourceInput,
    indexes: &Indexes<'_>,
    orders: &BTreeMap<&str, Vec<&str>>,
) -> Result<Vec<String>, BranchExplorerError> {
    let root_id = *indexes
        .roots
        .first()
        .ok_or_else(|| BranchExplorerError::new(BranchExplorerIssue::EmptyTree, "root"))?;
    let mut output = Vec::with_capacity(source.nodes.len());
    append_preorder(root_id, indexes, orders, &mut output)?;
    if output.len() != source.nodes.len() {
        return Err(BranchExplorerError::new(BranchExplorerIssue::Cycle, "topology"));
    }
    Ok(output)
}

fn append_preorder(
    entry_id: &str,
    indexes: &Indexes<'_>,
    orders: &BTreeMap<&str, Vec<&str>>,
    output: &mut Vec<String>,
) -> Result<(), BranchExplorerError> {
    if output.iter().any(|existing| existing == entry_id) {
        return Err(BranchExplorerError::new(BranchExplorerIssue::Cycle, "topology"));
    }
    output.push(String::from(entry_id));
    let canonical = indexes.children.get(entry_id).map(Vec::as_slice).unwrap_or_default();
    let explicit = orders.get(entry_id).map(Vec::as_slice).unwrap_or_default();
    for child_id in explicit {
        append_preorder(child_id, indexes, orders, output)?;
    }
    for child_id in canonical {
        if !explicit.contains(child_id) {
            append_preorder(child_id, indexes, orders, output)?;
        }
    }
    Ok(())
}

fn depth_of(entry_id: &str, by_id: &BTreeMap<&str, &BranchLedgerNodeInput>) -> Result<u32, BranchExplorerError> {
    let mut cursor = Some(entry_id);
    let mut seen = BTreeSet::new();
    let mut depth = 0_u32;
    while let Some(current_id) = cursor {
        if !seen.insert(current_id) {
            return Err(BranchExplorerError::new(BranchExplorerIssue::Cycle, "parent_id"));
        }
        let node = by_id
            .get(current_id)
            .ok_or_else(|| BranchExplorerError::new(BranchExplorerIssue::MissingParent, "parent_id"))?;
        cursor = node.parent_id.as_deref();
        if cursor.is_some() {
            depth = depth
                .checked_add(1)
                .ok_or_else(|| BranchExplorerError::new(BranchExplorerIssue::BoundExceeded, "depth"))?;
        }
    }
    Ok(depth)
}

fn collect_ancestor_set<'a>(
    leaf_id: &'a str,
    by_id: &BTreeMap<&'a str, &'a BranchLedgerNodeInput>,
) -> Result<BTreeSet<&'a str>, BranchExplorerError> {
    let mut output = BTreeSet::new();
    let mut cursor = Some(leaf_id);
    while let Some(current_id) = cursor {
        if !output.insert(current_id) {
            return Err(BranchExplorerError::new(BranchExplorerIssue::Cycle, "active_path"));
        }
        cursor = by_id
            .get(current_id)
            .ok_or_else(|| BranchExplorerError::new(BranchExplorerIssue::MissingParent, "active_path"))?
            .parent_id
            .as_deref();
    }
    Ok(output)
}

fn projection_path<'a>(
    target_id: &'a str,
    by_id: &BTreeMap<&'a str, &'a BranchProjectionNode>,
) -> Result<Vec<&'a str>, BranchExplorerError> {
    if !by_id.contains_key(target_id) {
        return Err(BranchExplorerError::new(BranchExplorerIssue::UnknownTarget, "comparison_target"));
    }
    let mut reversed = Vec::new();
    let mut seen = BTreeSet::new();
    let mut cursor = Some(target_id);
    while let Some(current_id) = cursor {
        if !seen.insert(current_id) {
            return Err(BranchExplorerError::new(BranchExplorerIssue::Cycle, "comparison_path"));
        }
        reversed.push(current_id);
        cursor = by_id[current_id].parent_id.as_deref();
        if cursor.is_some_and(|parent_id| !by_id.contains_key(parent_id)) {
            return Err(BranchExplorerError::new(BranchExplorerIssue::MissingCommonAncestor, "comparison_path"));
        }
    }
    reversed.reverse();
    Ok(reversed)
}

fn suffix_after<'a>(ancestor_id: &str, path: &'a [&'a str]) -> &'a [&'a str] {
    let position = path.iter().position(|entry_id| *entry_id == ancestor_id).unwrap_or(path.len());
    let start = position.saturating_add(1).min(path.len());
    &path[start..]
}

fn nodes_for_ids<'a>(
    ids: &[String],
    by_id: &BTreeMap<&'a str, &'a BranchProjectionNode>,
) -> Result<Vec<&'a BranchProjectionNode>, BranchExplorerError> {
    ids.iter()
        .map(|entry_id| {
            by_id
                .get(entry_id.as_str())
                .copied()
                .ok_or_else(|| BranchExplorerError::new(BranchExplorerIssue::UnknownTarget, "comparison"))
        })
        .collect()
}

fn class_counts(nodes: &[&BranchProjectionNode]) -> Vec<(BranchEntryClass, u32)> {
    let mut counts = BTreeMap::<BranchEntryClass, u32>::new();
    for node in nodes {
        let count = counts.entry(node.entry_class).or_default();
        *count = count.saturating_add(1);
    }
    counts.into_iter().collect()
}

fn usage_sum(nodes: &[&BranchProjectionNode]) -> Option<u64> {
    nodes
        .iter()
        .try_fold(0_u64, |total, node| node.usage_units.and_then(|usage| total.checked_add(usage)))
}

fn known_target<'a>(target_id: &'a str, indexes: &Indexes<'a>) -> Result<&'a str, BranchExplorerError> {
    indexes
        .by_id
        .contains_key(target_id)
        .then_some(target_id)
        .ok_or_else(|| BranchExplorerError::new(BranchExplorerIssue::UnknownTarget, "annotation_target"))
}

fn validate_isolation(
    source: &BranchExplorerSourceInput,
    external_state: ExternalStatePosture,
) -> Result<(), BranchExplorerError> {
    if external_state == ExternalStatePosture::Isolated && source.isolation_evidence_id.is_none() {
        return Err(BranchExplorerError::new(BranchExplorerIssue::MissingIsolationEvidence, "isolation_evidence_id"));
    }
    if let Some(evidence_id) = source.isolation_evidence_id.as_deref() {
        safe_identity(evidence_id, "isolation_evidence_id")?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct PlanBase<'a> {
    projection: &'a BranchProjectionSnapshot,
    operation: BranchExplorerOperation,
    expected_revision_id: &'a str,
    expected_active_leaf_id: &'a str,
    target_entry_id: &'a str,
}

fn admitted_plan(
    base: PlanBase<'_>,
    continuation_independent: bool,
    append_only_branch_operation: Option<SessionBranchOperation>,
) -> BranchOperationPlan {
    plan_from_base(base, BranchPlanStatus::Admitted, continuation_independent, append_only_branch_operation, None)
}

fn blocked_plan(base: PlanBase<'_>, issue: BranchExplorerIssue) -> BranchOperationPlan {
    plan_from_base(base, BranchPlanStatus::Blocked, false, None, Some(issue))
}

fn plan_from_base(
    base: PlanBase<'_>,
    status: BranchPlanStatus,
    continuation_independent: bool,
    append_only_branch_operation: Option<SessionBranchOperation>,
    diagnostic: Option<BranchExplorerIssue>,
) -> BranchOperationPlan {
    let operation_id = hash_parts(BRANCH_EXPLORER_OPERATION_DOMAIN, [
        base.projection.projection_id.as_str(),
        base.operation.as_str(),
        base.expected_revision_id,
        base.expected_active_leaf_id,
        base.target_entry_id,
        status.as_str(),
    ]);
    BranchOperationPlan {
        operation_id,
        operation: base.operation,
        status,
        session_id: base.projection.session_id.clone(),
        expected_revision_id: String::from(base.expected_revision_id),
        expected_active_leaf_id: String::from(base.expected_active_leaf_id),
        target_entry_id: String::from(base.target_entry_id),
        policy_id: base.projection.policy_id.clone(),
        external_state: base.projection.external_state,
        continuation_independent,
        isolation_evidence_id: base.projection.isolation_evidence_id.clone(),
        append_only_branch_operation,
        diagnostic,
    }
}

fn projection_identity(
    policy: &BranchExplorerPolicy,
    source: &BranchExplorerSourceInput,
    external_state: ExternalStatePosture,
    nodes: &[BranchProjectionNode],
    omitted_node_count: u32,
) -> String {
    let mut parts = vec![
        policy.policy_id.clone(),
        source.session_id.clone(),
        source.revision_id.clone(),
        source.ledger_generation.to_string(),
        source.active_leaf_id.clone(),
        String::from(external_state.as_str()),
        omitted_node_count.to_string(),
    ];
    for node in nodes {
        parts.extend([
            node.entry_id.clone(),
            node.parent_id.clone().unwrap_or_default(),
            String::from(node.entry_class.as_str()),
            node.storage_sequence.to_string(),
            node.depth.to_string(),
            node.child_count.to_string(),
            node.is_leaf.to_string(),
            node.is_active_path.to_string(),
            node.is_bookmarked.to_string(),
        ]);
        parts.extend(node.labels.iter().cloned());
        parts.extend(node.artifact_reference_ids.iter().cloned());
        parts.extend(node.effect_receipt_ids.iter().cloned());
    }
    hash_parts(BRANCH_EXPLORER_PROJECTION_DOMAIN, parts.iter().map(String::as_str))
}

fn safe_identity(value: &str, field: &'static str) -> Result<(), BranchExplorerError> {
    let safe = !value.is_empty()
        && !value.chars().any(char::is_control)
        && !UNSAFE_TEXT_MARKERS.iter().any(|marker| value.to_ascii_lowercase().contains(marker));
    if !safe {
        return Err(BranchExplorerError::new(BranchExplorerIssue::UnsafeLabel, field));
    }
    Ok(())
}

pub(crate) fn hash_parts<'a>(domain: &str, parts: impl IntoIterator<Item = &'a str>) -> String {
    let mut hasher = blake3::Hasher::new();
    update_hash_part(&mut hasher, domain);
    for part in parts {
        update_hash_part(&mut hasher, part);
    }
    hasher.finalize().to_hex().to_string()
}

fn update_hash_part(hasher: &mut blake3::Hasher, part: &str) {
    let byte_count = u64::try_from(part.len()).unwrap_or(u64::MAX);
    hasher.update(&byte_count.to_le_bytes());
    hasher.update(part.as_bytes());
}
