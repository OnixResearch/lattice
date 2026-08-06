use super::super::*;

fn generation(value: u64) -> CollectionGeneration {
    CollectionGeneration::new(value).unwrap()
}

fn extent(value: f64) -> LogicalExtent {
    LogicalExtent::new(value).unwrap()
}

fn snapshot(items: Vec<u32>) -> CollectionSnapshot<u32> {
    CollectionSnapshot::new(generation(1), items).unwrap()
}

fn request() -> MaterializationRequest<u32> {
    MaterializationRequest {
        collection_generation: generation(1),
        measurement_generation: 1,
        viewport_start: 15.0,
        viewport_extent: extent(20.0),
        uniform_extent: None,
        measurements: vec![
            ItemMeasurement {
                id: 0,
                extent: extent(20.0),
            },
            ItemMeasurement {
                id: 2,
                extent: extent(5.0),
            },
        ],
        estimated_extent: extent(10.0),
        overscan: 5.0,
        anchor: Some(AnchorInput {
            id: 2,
            previous_item_offset: 20.0,
            viewport_reference: 5.0,
        }),
        scroll_to: None,
    }
}

#[test]
fn variable_measurements_compute_exact_visible_overscan_and_correction() {
    let plan = materialize_collection(&snapshot(vec![0, 1, 2, 3, 4]), &request()).unwrap();
    assert_eq!(plan.visible, vec![0, 1, 2]);
    assert_eq!(plan.materialized.iter().map(|item| item.id).collect::<Vec<_>>(), vec![0, 1, 2, 3]);
    assert_eq!(plan.materialized.iter().map(|item| item.measured).collect::<Vec<_>>(), vec![
        true, false, true, false
    ]);
    assert_eq!(plan.before_extent, 0.0);
    assert_eq!(plan.after_extent, 10.0);
    assert_eq!(plan.total_extent, 55.0);
    assert_eq!(plan.anchor.unwrap().scroll_correction, 10.0);
}

#[test]
fn exact_boundaries_empty_and_uniform_path_are_deterministic() {
    let empty = snapshot(Vec::new());
    let mut empty_request = request();
    empty_request.viewport_start = 0.0;
    empty_request.measurements.clear();
    empty_request.anchor = None;
    let plan = materialize_collection(&empty, &empty_request).unwrap();
    assert!(plan.visible.is_empty());
    assert_eq!(plan.total_extent, 0.0);

    let rows = snapshot(vec![10, 11, 12, 13]);
    let uniform = MaterializationRequest {
        collection_generation: generation(1),
        measurement_generation: 2,
        viewport_start: 1.0,
        viewport_extent: extent(2.0),
        uniform_extent: Some(extent(1.0)),
        measurements: Vec::new(),
        estimated_extent: extent(1.0),
        overscan: 0.0,
        anchor: None,
        scroll_to: Some((13, ScrollAlignment::End)),
    };
    let plan = materialize_collection(&rows, &uniform).unwrap();
    assert_eq!(plan.visible, vec![11, 12]);
    assert_eq!(plan.scroll_request.unwrap().viewport_start, 2.0);
}

#[test]
fn snapshot_updates_are_atomic_generation_checked_and_identity_unique() {
    let original = snapshot(vec![1, 2, 3]);
    let inserted = original.apply(generation(1), generation(2), SnapshotChange::Insert { index: 1, id: 4 }).unwrap();
    assert_eq!(inserted.items, vec![1, 4, 2, 3]);
    let moved = inserted.apply(generation(2), generation(3), SnapshotChange::Move { id: 3, to: 0 }).unwrap();
    assert_eq!(moved.items, vec![3, 1, 4, 2]);
    let removed = moved.apply(generation(3), generation(4), SnapshotChange::Remove { id: 4 }).unwrap();
    assert_eq!(removed.items, vec![3, 1, 2]);
    assert_eq!(
        removed.apply(generation(3), generation(5), SnapshotChange::Reset(vec![7])),
        Err(CollectionDiagnostic::new(CollectionDiagnosticCode::InvalidGeneration, "snapshot-change"))
    );
    let reset = removed.apply(generation(4), generation(5), SnapshotChange::Reset(vec![8, 9])).unwrap();
    assert_eq!(reset.items, vec![8, 9]);
    assert!(CollectionSnapshot::new(generation(1), vec![1, 1]).is_err());
    assert_eq!(original.items, vec![1, 2, 3]);
}

#[test]
fn malformed_measurements_generations_and_scroll_targets_fail_closed() {
    assert!(LogicalExtent::new(0.0).is_err());
    assert!(LogicalExtent::new(f64::NAN).is_err());
    assert!(LogicalExtent::new(f64::INFINITY).is_err());
    let items = snapshot(vec![0, 1]);
    let mut malformed = request();
    malformed.measurements = vec![ItemMeasurement {
        id: 9,
        extent: extent(1.0),
    }];
    assert_eq!(
        materialize_collection(&items, &malformed).unwrap_err().code,
        CollectionDiagnosticCode::UnknownIdentity
    );
    malformed.measurements = vec![
        ItemMeasurement {
            id: 0,
            extent: extent(1.0),
        },
        ItemMeasurement {
            id: 0,
            extent: extent(2.0),
        },
    ];
    assert_eq!(
        materialize_collection(&items, &malformed).unwrap_err().code,
        CollectionDiagnosticCode::DuplicateMeasurement
    );
    malformed.measurements.clear();
    malformed.anchor = None;
    malformed.overscan = -1.0;
    assert_eq!(
        materialize_collection(&items, &malformed).unwrap_err().code,
        CollectionDiagnosticCode::InvalidOverscan
    );
    malformed.overscan = 0.0;
    malformed.collection_generation = generation(2);
    assert_eq!(
        materialize_collection(&items, &malformed).unwrap_err().code,
        CollectionDiagnosticCode::InvalidGeneration
    );
    malformed.collection_generation = generation(1);
    malformed.measurement_generation = 0;
    assert_eq!(
        materialize_collection(&items, &malformed).unwrap_err().code,
        CollectionDiagnosticCode::InvalidGeneration
    );
    malformed.measurement_generation = 1;
    malformed.scroll_to = Some((99, ScrollAlignment::Start));
    assert_eq!(
        materialize_collection(&items, &malformed).unwrap_err().code,
        CollectionDiagnosticCode::MissingScrollTarget
    );
}

#[test]
fn identity_state_survives_reorder_and_repairs_removed_or_disabled_active_item() {
    let previous = snapshot(vec![1, 2, 3]);
    let candidate = CollectionSnapshot::new(generation(2), vec![3, 1]).unwrap();
    let state = CollectionIdentityState {
        active: Some(2),
        selected: std::collections::BTreeSet::from([1, 2]),
        anchor: Some(3),
    };
    let repaired = reconcile_collection_state(
        &previous,
        &candidate,
        &state,
        &std::collections::BTreeSet::from([1]),
        IdentityRepairPolicy::NearestSurvivor,
    )
    .unwrap();
    assert_eq!(repaired.state.active, Some(3));
    assert_eq!(repaired.state.selected, std::collections::BTreeSet::from([1]));
    assert_eq!(repaired.state.anchor, Some(3));
    assert_eq!(repaired.diagnostics.len(), 1);

    let reordered = CollectionSnapshot::new(generation(2), vec![3, 2, 1]).unwrap();
    let preserved = reconcile_collection_state(
        &previous,
        &reordered,
        &state,
        &std::collections::BTreeSet::new(),
        IdentityRepairPolicy::NearestSurvivor,
    )
    .unwrap();
    assert_eq!(preserved.state, state);

    let empty = CollectionSnapshot::new(generation(2), Vec::new()).unwrap();
    let cleared = reconcile_collection_state(
        &previous,
        &empty,
        &state,
        &std::collections::BTreeSet::new(),
        IdentityRepairPolicy::NearestSurvivor,
    )
    .unwrap();
    assert_eq!(cleared.state.active, None);
    assert_eq!(cleared.state.anchor, None);
    assert!(cleared.state.selected.is_empty());
}
