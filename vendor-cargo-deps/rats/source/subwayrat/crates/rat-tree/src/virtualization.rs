//! Flattened terminal tree adapter for shared virtualization and semantics.

use std::collections::BTreeSet;

use crate::model::NodeId;
use crate::model::VisibleRow;

/// Shared visual and semantic projection for a terminal tree.
#[derive(Debug, Clone, PartialEq)]
pub struct TreeVirtualizationProjection {
    /// Pure visual materialization plan.
    pub plan: ratcore::viewport::MaterializationPlan<NodeId>,
    /// Semantic metadata available for mounted rows.
    pub semantics: Vec<ratcore::viewport::CollectionSemanticProjection<NodeId>>,
    /// Explicit terminal offscreen-semantics limitation.
    pub offscreen_policy: ratcore::viewport::OffscreenSemanticPolicy,
}

/// Repairs a tree active identity through shared collection policy.
pub fn repair_tree_active(
    previous_rows: &[VisibleRow],
    candidate_rows: &[VisibleRow],
    active: Option<NodeId>,
    disabled: &BTreeSet<NodeId>,
) -> Result<Option<NodeId>, ratcore::viewport::CollectionDiagnostic> {
    let first = ratcore::viewport::CollectionGeneration::new(1)?;
    let second = ratcore::viewport::CollectionGeneration::new(2)?;
    let previous =
        ratcore::viewport::CollectionSnapshot::new(first, previous_rows.iter().map(|row| row.node_id).collect())?;
    let candidate =
        ratcore::viewport::CollectionSnapshot::new(second, candidate_rows.iter().map(|row| row.node_id).collect())?;
    let state = ratcore::viewport::CollectionIdentityState {
        active,
        selected: BTreeSet::new(),
        anchor: None,
    };
    ratcore::viewport::reconcile_collection_state(
        &previous,
        &candidate,
        &state,
        disabled,
        ratcore::viewport::IdentityRepairPolicy::NearestSurvivor,
    )
    .map(|projection| projection.state.active)
}

/// Projects an existing flattened tree order without owning flattening or rows.
pub fn project_tree_rows(
    generation: ratcore::viewport::CollectionGeneration,
    measurement_generation: u64,
    rows: &[VisibleRow],
    viewport_start: usize,
    viewport_height: usize,
    selected: &BTreeSet<NodeId>,
    disabled: &BTreeSet<NodeId>,
) -> Result<TreeVirtualizationProjection, ratcore::viewport::CollectionDiagnostic> {
    let snapshot =
        ratcore::viewport::CollectionSnapshot::new(generation, rows.iter().map(|row| row.node_id).collect())?;
    let request = ratcore::viewport::MaterializationRequest {
        collection_generation: generation,
        measurement_generation,
        viewport_start: viewport_start as f64,
        viewport_extent: ratcore::viewport::LogicalExtent::new(viewport_height as f64)?,
        uniform_extent: Some(ratcore::viewport::LogicalExtent::new(1.0)?),
        measurements: Vec::new(),
        estimated_extent: ratcore::viewport::LogicalExtent::new(1.0)?,
        overscan: 0.0,
        anchor: None,
        scroll_to: None,
    };
    let plan = ratcore::viewport::materialize_collection(&snapshot, &request)?;
    let semantic_inputs = rows
        .iter()
        .map(|row| ratcore::viewport::CollectionSemanticInput {
            id: row.node_id,
            level: u64::try_from(row.depth.saturating_add(1)).unwrap_or(u64::MAX),
            position: u64::try_from(row.position).unwrap_or(u64::MAX),
            set_size: u64::try_from(row.set_size).unwrap_or(u64::MAX),
            selected: selected.contains(&row.node_id),
            expanded: row.has_children.then_some(row.is_expanded),
            disabled: disabled.contains(&row.node_id),
        })
        .collect::<Vec<_>>();
    let offscreen_policy = ratcore::viewport::OffscreenSemanticPolicy::MaterializedOnly {
        reason: "terminal output has no native offscreen semantic collection".to_string(),
    };
    let semantics =
        ratcore::viewport::project_collection_semantics(&semantic_inputs, &plan.materialized, &offscreen_policy)?;
    Ok(TreeVirtualizationProjection {
        plan,
        semantics,
        offscreen_policy,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::SimpleTree;

    #[test]
    fn collapsed_flat_order_and_semantic_levels_share_materialization() {
        let tree = SimpleTree::new(vec![
            (NodeId::new(1), None, "root".into()),
            (NodeId::new(2), Some(NodeId::new(1)), "child".into()),
        ]);
        let rows = crate::model::compute_visible_rows(&tree, &BTreeSet::from([NodeId::new(1)]));
        let projection = project_tree_rows(
            ratcore::viewport::CollectionGeneration::new(1).unwrap(),
            1,
            &rows,
            0,
            1,
            &BTreeSet::from([NodeId::new(2)]),
            &BTreeSet::new(),
        )
        .unwrap();
        assert_eq!(projection.plan.visible, vec![NodeId::new(1)]);
        assert_eq!(projection.semantics.len(), 1);
        assert_eq!(projection.semantics[0].level, 1);
        assert!(matches!(
            projection.offscreen_policy,
            ratcore::viewport::OffscreenSemanticPolicy::MaterializedOnly { .. }
        ));
    }
}
