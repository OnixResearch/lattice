//! Deterministic semantic virtualization evidence.

use super::model::*;

/// Coordinate-family classification for parity claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionCoordinateFamily {
    /// Uniform logical rows.
    LogicalRows,
    /// Browser CSS pixels.
    CssPixels,
    /// Native logical pixels/points.
    NativeLogical,
}

/// Normalized materialization evidence, distinct from visual output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionProjectionEvidence {
    /// Collection generation.
    pub collection_generation: u64,
    /// Measurement generation.
    pub measurement_generation: u64,
    /// Ordered requested identity labels.
    pub requested_ids: Vec<String>,
    /// Ordered visible identity labels.
    pub visible_ids: Vec<String>,
    /// Active identity label.
    pub active_id: Option<String>,
    /// Selected identity labels.
    pub selected_ids: Vec<String>,
    /// Whether anchor correction was requested.
    pub has_anchor_correction: bool,
    /// Whether a scroll-to-item request was produced.
    pub has_scroll_request: bool,
    /// Renderer-normalized coordinate family.
    pub coordinate_family: CollectionCoordinateFamily,
    /// BLAKE3 over canonical semantic fields.
    pub semantic_blake3: String,
}

/// Builds deterministic semantic evidence without pixel/cell claims.
#[must_use]
pub fn collection_projection_evidence<Id: Clone + Ord + ToString>(
    plan: &MaterializationPlan<Id>,
    state: &super::selection::CollectionIdentityState<Id>,
    coordinate_family: CollectionCoordinateFamily,
) -> CollectionProjectionEvidence {
    debug_assert!(plan.collection_generation.get() > 0);
    debug_assert!(plan.measurement_generation > 0);
    let requested_ids: Vec<String> = plan.materialized.iter().map(|item| item.id.to_string()).collect();
    let visible_ids: Vec<String> = plan.visible.iter().map(ToString::to_string).collect();
    let selected_ids: Vec<String> = state.selected.iter().map(ToString::to_string).collect();
    let active_id = state.active.as_ref().map(ToString::to_string);
    let canonical = format!(
        "collection={}\nmeasurement={}\nrequested={}\nvisible={}\nactive={}\nselected={}\nanchor={}\nscroll={}\n",
        plan.collection_generation.get(),
        plan.measurement_generation,
        requested_ids.join(","),
        visible_ids.join(","),
        active_id.as_deref().unwrap_or("-"),
        selected_ids.join(","),
        plan.anchor.is_some(),
        plan.scroll_request.is_some(),
    );
    CollectionProjectionEvidence {
        collection_generation: plan.collection_generation.get(),
        measurement_generation: plan.measurement_generation,
        requested_ids,
        visible_ids,
        active_id,
        selected_ids,
        has_anchor_correction: plan.anchor.is_some(),
        has_scroll_request: plan.scroll_request.is_some(),
        coordinate_family,
        semantic_blake3: crate::interaction_replay::blake3_hex(canonical.as_bytes()),
    }
}

/// Rejects stale normalized evidence for the current accepted plan and state.
pub fn validate_collection_projection_evidence<Id: Clone + Ord + ToString>(
    plan: &MaterializationPlan<Id>,
    state: &super::selection::CollectionIdentityState<Id>,
    coordinate_family: CollectionCoordinateFamily,
    actual: &CollectionProjectionEvidence,
) -> Result<(), CollectionDiagnostic> {
    let expected = collection_projection_evidence(plan, state, coordinate_family);
    (expected == *actual)
        .then_some(())
        .ok_or_else(|| CollectionDiagnostic::new(CollectionDiagnosticCode::StaleEvidence, "projection-evidence"))
}

/// Validates an exact-window parity claim between coordinate families.
pub fn validate_exact_window_parity(
    left: CollectionCoordinateFamily,
    right: CollectionCoordinateFamily,
) -> Result<(), CollectionDiagnostic> {
    equivalent_coordinate_families(left, right).then_some(()).ok_or_else(|| {
        CollectionDiagnostic::new(CollectionDiagnosticCode::IncompatibleCoordinateParity, "coordinate-family")
    })
}

/// Determines whether exact physical-window parity may be compared.
#[must_use]
pub fn equivalent_coordinate_families(left: CollectionCoordinateFamily, right: CollectionCoordinateFamily) -> bool {
    left == right
}
