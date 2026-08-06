//! Ordered snapshot validation and atomic generation-checked updates.

use super::model::*;

impl<Id: Clone + Ord> CollectionSnapshot<Id> {
    /// Validates a complete immutable snapshot.
    pub fn new(generation: CollectionGeneration, items: Vec<Id>) -> Result<Self, CollectionDiagnostic> {
        validate_items(&items)?;
        Ok(Self { generation, items })
    }

    /// Applies one atomic update at exactly the next generation.
    pub fn apply(
        &self,
        expected_generation: CollectionGeneration,
        next_generation: CollectionGeneration,
        change: SnapshotChange<Id>,
    ) -> Result<Self, CollectionDiagnostic> {
        if expected_generation != self.generation || self.generation.next() != Some(next_generation) {
            return Err(CollectionDiagnostic::new(CollectionDiagnosticCode::InvalidGeneration, "snapshot-change"));
        }
        let mut candidate = self.items.clone();
        match change {
            SnapshotChange::Reset(items) => candidate = items,
            SnapshotChange::Insert { index, id } => {
                if index > candidate.len() {
                    return Err(CollectionDiagnostic::new(CollectionDiagnosticCode::InvalidIndex, index.to_string()));
                }
                candidate.insert(index, id);
            }
            SnapshotChange::Remove { id } => {
                let Some(index) = candidate.iter().position(|candidate| candidate == &id) else {
                    return Err(CollectionDiagnostic::new(CollectionDiagnosticCode::UnknownIdentity, "remove"));
                };
                candidate.remove(index);
            }
            SnapshotChange::Move { id, to } => {
                let Some(from) = candidate.iter().position(|candidate| candidate == &id) else {
                    return Err(CollectionDiagnostic::new(CollectionDiagnosticCode::UnknownIdentity, "move"));
                };
                if to >= candidate.len() {
                    return Err(CollectionDiagnostic::new(CollectionDiagnosticCode::InvalidIndex, to.to_string()));
                }
                let moved = candidate.remove(from);
                candidate.insert(to, moved);
            }
        }
        Self::new(next_generation, candidate)
    }
}

fn validate_items<Id: Ord>(items: &[Id]) -> Result<(), CollectionDiagnostic> {
    if items.len() > MAX_COLLECTION_ITEMS {
        return Err(CollectionDiagnostic::new(CollectionDiagnosticCode::LimitExceeded, "snapshot-items"));
    }
    let mut identities = std::collections::BTreeSet::new();
    if items.iter().any(|id| !identities.insert(id)) {
        return Err(CollectionDiagnostic::new(CollectionDiagnosticCode::DuplicateIdentity, "snapshot-items"));
    }
    Ok(())
}

pub(super) fn validate_request<Id: Ord>(
    snapshot: &CollectionSnapshot<Id>,
    request: &MaterializationRequest<Id>,
) -> Result<(), CollectionDiagnostic> {
    validate_generations(snapshot, request)?;
    validate_geometry(request)?;
    validate_measurement_shape(request)?;
    validate_anchor(request)
}

fn validate_generations<Id>(
    snapshot: &CollectionSnapshot<Id>,
    request: &MaterializationRequest<Id>,
) -> Result<(), CollectionDiagnostic> {
    if request.collection_generation == snapshot.generation && request.measurement_generation > 0 {
        return Ok(());
    }
    Err(CollectionDiagnostic::new(CollectionDiagnosticCode::InvalidGeneration, "materialization-request"))
}

fn validate_geometry<Id>(request: &MaterializationRequest<Id>) -> Result<(), CollectionDiagnostic> {
    if !request.viewport_start.is_finite() || request.viewport_start < 0.0 {
        return Err(CollectionDiagnostic::new(CollectionDiagnosticCode::InvalidExtent, "viewport-start"));
    }
    let is_overscan_valid = request.overscan.is_finite() && request.overscan >= 0.0 && request.overscan <= MAX_OVERSCAN;
    is_overscan_valid
        .then_some(())
        .ok_or_else(|| CollectionDiagnostic::new(CollectionDiagnosticCode::InvalidOverscan, "overscan"))
}

fn validate_measurement_shape<Id>(request: &MaterializationRequest<Id>) -> Result<(), CollectionDiagnostic> {
    if request.measurements.len() > MAX_MEASUREMENTS {
        return Err(CollectionDiagnostic::new(CollectionDiagnosticCode::LimitExceeded, "measurements"));
    }
    if request.uniform_extent.is_some() && !request.measurements.is_empty() {
        return Err(CollectionDiagnostic::new(CollectionDiagnosticCode::DuplicateMeasurement, "uniform-and-variable"));
    }
    Ok(())
}

fn validate_anchor<Id>(request: &MaterializationRequest<Id>) -> Result<(), CollectionDiagnostic> {
    let is_invalid = request.anchor.as_ref().is_some_and(|anchor| {
        !anchor.previous_item_offset.is_finite()
            || anchor.previous_item_offset < 0.0
            || !anchor.viewport_reference.is_finite()
    });
    (!is_invalid)
        .then_some(())
        .ok_or_else(|| CollectionDiagnostic::new(CollectionDiagnosticCode::InvalidExtent, "anchor"))
}
