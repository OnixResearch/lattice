use super::super::*;

fn extent(value: f64) -> LogicalExtent {
    LogicalExtent::new(value).unwrap()
}

#[test]
fn large_sparse_collection_is_bounded_and_does_not_materialize_every_item() {
    let generation = CollectionGeneration::new(1).unwrap();
    let snapshot = CollectionSnapshot::new(generation, (0_u32..100_000).collect()).unwrap();
    let request = MaterializationRequest {
        collection_generation: generation,
        measurement_generation: 1,
        viewport_start: 99_990.0,
        viewport_extent: extent(5.0),
        uniform_extent: None,
        measurements: vec![ItemMeasurement {
            id: 99_992,
            extent: extent(2.0),
        }],
        estimated_extent: extent(1.0),
        overscan: 2.0,
        anchor: None,
        scroll_to: None,
    };
    let plan = materialize_collection(&snapshot, &request).unwrap();
    assert_eq!(plan.visible, vec![99_990, 99_991, 99_992, 99_993]);
    assert!(plan.materialized.len() <= 9);
    assert_eq!(plan.total_extent, 100_001.0);
}

#[test]
fn overflowing_coordinates_and_policy_limits_fail_before_projection() {
    let generation = CollectionGeneration::new(1).unwrap();
    let snapshot = CollectionSnapshot::new(generation, vec![1_u8]).unwrap();
    let mut request = MaterializationRequest {
        collection_generation: generation,
        measurement_generation: 1,
        viewport_start: f64::MAX,
        viewport_extent: extent(f64::MAX),
        uniform_extent: Some(extent(1.0)),
        measurements: Vec::new(),
        estimated_extent: extent(1.0),
        overscan: 0.0,
        anchor: None,
        scroll_to: None,
    };
    assert_eq!(
        materialize_collection(&snapshot, &request).unwrap_err().code,
        CollectionDiagnosticCode::InvalidExtent
    );
    request.viewport_start = 0.0;
    request.viewport_extent = extent(1.0);
    request.overscan = MAX_OVERSCAN + 1.0;
    assert_eq!(
        materialize_collection(&snapshot, &request).unwrap_err().code,
        CollectionDiagnosticCode::InvalidOverscan
    );
    request.overscan = 0.0;
    request.uniform_extent = None;
    request.measurements = vec![
        ItemMeasurement {
            id: 1,
            extent: extent(1.0),
        };
        MAX_MEASUREMENTS.saturating_add(1)
    ];
    assert_eq!(
        materialize_collection(&snapshot, &request).unwrap_err().code,
        CollectionDiagnosticCode::LimitExceeded
    );
    assert!(LogicalExtent::new(-1.0).is_err());
}
