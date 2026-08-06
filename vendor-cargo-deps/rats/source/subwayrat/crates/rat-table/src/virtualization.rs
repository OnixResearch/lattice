//! Ratatui table normalization into shared collection materialization.

/// Stable row identity owned by a Subwayrat table domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TableRowId(u64);

impl TableRowId {
    /// Creates a stable table row identity.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns its domain value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for TableRowId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Builds an immutable table-row order without exposing Ratatui row objects.
pub fn table_row_snapshot(
    generation: ratcore::viewport::CollectionGeneration,
    row_ids: Vec<TableRowId>,
) -> Result<ratcore::viewport::CollectionSnapshot<TableRowId>, ratcore::viewport::CollectionDiagnostic> {
    ratcore::viewport::CollectionSnapshot::new(generation, row_ids)
}

/// Delegates checked row materialization to Ratcore.
pub fn project_table_rows(
    snapshot: &ratcore::viewport::CollectionSnapshot<TableRowId>,
    request: &ratcore::viewport::MaterializationRequest<TableRowId>,
) -> Result<ratcore::viewport::MaterializationPlan<TableRowId>, ratcore::viewport::CollectionDiagnostic> {
    ratcore::viewport::materialize_collection(snapshot, request)
}

/// Delegates active/selection/anchor preservation instead of repairing indices locally.
pub fn reconcile_table_state(
    previous: &ratcore::viewport::CollectionSnapshot<TableRowId>,
    candidate: &ratcore::viewport::CollectionSnapshot<TableRowId>,
    state: &ratcore::viewport::CollectionIdentityState<TableRowId>,
    disabled: &std::collections::BTreeSet<TableRowId>,
) -> Result<ratcore::viewport::IdentityReconciliation<TableRowId>, ratcore::viewport::CollectionDiagnostic> {
    ratcore::viewport::reconcile_collection_state(
        previous,
        candidate,
        state,
        disabled,
        ratcore::viewport::IdentityRepairPolicy::NearestSurvivor,
    )
}

/// Builds semantic table evidence explicitly classified as logical rows.
#[must_use]
pub fn table_projection_evidence(
    plan: &ratcore::viewport::MaterializationPlan<TableRowId>,
    state: &ratcore::viewport::CollectionIdentityState<TableRowId>,
) -> ratcore::viewport::CollectionProjectionEvidence {
    ratcore::viewport::collection_projection_evidence(
        plan,
        state,
        ratcore::viewport::CollectionCoordinateFamily::LogicalRows,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_table_fixture_receipt_is_stable() {
        let table = crate::DataTable::new(vec!["name".into(), "value".into()], vec![
            vec!["alpha".into(), "1".into()],
            vec!["beta".into(), "2".into()],
        ]);
        let canonical = format!(
            "rows={}\nselected={}\ncolumns={}\n",
            table.row_count(),
            table.selected_index().get(),
            table.column_count(),
        );
        assert_eq!(canonical.lines().count(), 3);
        assert_eq!(
            ratcore::interaction_replay::blake3_hex(canonical.as_bytes()),
            "dfce57ccd95ab6e0c4be4a0266a774e07c380791f2f6ab954d8c2288efe8cea3"
        );
    }

    #[test]
    fn variable_table_rows_preserve_requested_identity_under_reorder() {
        let first = ratcore::viewport::CollectionGeneration::new(1).unwrap();
        let second = ratcore::viewport::CollectionGeneration::new(2).unwrap();
        let snapshot = table_row_snapshot(first, vec![TableRowId::new(7), TableRowId::new(9)]).unwrap();
        let reordered = snapshot
            .apply(first, second, ratcore::viewport::SnapshotChange::Move {
                id: TableRowId::new(9),
                to: 0,
            })
            .unwrap();
        let request = ratcore::viewport::MaterializationRequest {
            collection_generation: second,
            measurement_generation: 3,
            viewport_start: 0.0,
            viewport_extent: ratcore::viewport::LogicalExtent::new(2.0).unwrap(),
            uniform_extent: None,
            measurements: vec![ratcore::viewport::ItemMeasurement {
                id: TableRowId::new(9),
                extent: ratcore::viewport::LogicalExtent::new(1.5).unwrap(),
            }],
            estimated_extent: ratcore::viewport::LogicalExtent::new(1.0).unwrap(),
            overscan: 1.0,
            anchor: None,
            scroll_to: Some((TableRowId::new(7), ratcore::viewport::ScrollAlignment::Nearest)),
        };
        let plan = project_table_rows(&reordered, &request).unwrap();
        assert_eq!(plan.visible, vec![TableRowId::new(9), TableRowId::new(7)]);
        assert_eq!(plan.scroll_request.as_ref().unwrap().id, TableRowId::new(7));
        let state = ratcore::viewport::CollectionIdentityState {
            active: Some(TableRowId::new(7)),
            selected: std::collections::BTreeSet::from([TableRowId::new(7)]),
            anchor: Some(TableRowId::new(9)),
        };
        let reconciled = reconcile_table_state(
            &snapshot,
            &reordered,
            &state,
            &std::collections::BTreeSet::from([TableRowId::new(7)]),
        )
        .unwrap();
        assert_eq!(reconciled.state.active, Some(TableRowId::new(9)));
        assert_eq!(reconciled.state.selected, state.selected);
        let evidence = table_projection_evidence(&plan, &reconciled.state);
        assert_eq!(evidence.requested_ids, vec!["9", "7"]);
    }
}
