use super::super::*;

fn extent(value: f64) -> LogicalExtent {
    LogicalExtent::new(value).unwrap()
}

fn generation(value: u64) -> CollectionGeneration {
    CollectionGeneration::new(value).unwrap()
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
fn semantics_can_outlive_visual_materialization_with_explicit_support_policy() {
    let items = snapshot(vec![1, 2, 3]);
    let mut uniform = request();
    uniform.viewport_start = 0.0;
    uniform.viewport_extent = extent(1.0);
    uniform.uniform_extent = Some(extent(1.0));
    uniform.measurements.clear();
    uniform.anchor = None;
    uniform.overscan = 0.0;
    let plan = materialize_collection(&items, &uniform).unwrap();
    let inputs: Vec<_> = (1..=3)
        .map(|id| CollectionSemanticInput {
            id,
            level: 1,
            position: u64::from(id),
            set_size: 3,
            selected: id == 2,
            expanded: None,
            disabled: id == 3,
        })
        .collect();
    let full =
        project_collection_semantics(&inputs, &plan.materialized, &OffscreenSemanticPolicy::FullCollection).unwrap();
    assert_eq!(full.len(), 3);
    assert!(!full[1].materialized);
    let mounted =
        project_collection_semantics(&inputs, &plan.materialized, &OffscreenSemanticPolicy::MaterializedOnly {
            reason: "terminal has no offscreen semantic tree".into(),
        })
        .unwrap();
    assert_eq!(mounted.len(), 1);
}

#[test]
fn evidence_hashes_semantics_and_refuses_cross_unit_exact_window_equivalence() {
    let plan = materialize_collection(&snapshot(vec![0, 1, 2, 3, 4]), &request()).unwrap();
    let state = CollectionIdentityState {
        active: Some(1),
        selected: std::collections::BTreeSet::from([1, 2]),
        anchor: Some(2),
    };
    let evidence = collection_projection_evidence(&plan, &state, CollectionCoordinateFamily::LogicalRows);
    assert_eq!(evidence.semantic_blake3.len(), 64);
    assert_eq!(evidence, collection_projection_evidence(&plan, &state, CollectionCoordinateFamily::LogicalRows));
    validate_collection_projection_evidence(&plan, &state, CollectionCoordinateFamily::LogicalRows, &evidence).unwrap();
    let mut stale = evidence;
    stale.measurement_generation = 99;
    assert_eq!(
        validate_collection_projection_evidence(&plan, &state, CollectionCoordinateFamily::LogicalRows, &stale,)
            .unwrap_err()
            .code,
        CollectionDiagnosticCode::StaleEvidence
    );
    assert!(!equivalent_coordinate_families(
        CollectionCoordinateFamily::LogicalRows,
        CollectionCoordinateFamily::CssPixels
    ));
    assert_eq!(
        validate_exact_window_parity(CollectionCoordinateFamily::LogicalRows, CollectionCoordinateFamily::CssPixels,)
            .unwrap_err()
            .code,
        CollectionDiagnosticCode::IncompatibleCoordinateParity
    );
}
