//! Bounded pure collection materialization.

use super::model::*;

/// Validates explicit measurements and computes a materialization plan.
pub fn materialize_collection<Id: Clone + Ord>(
    snapshot: &CollectionSnapshot<Id>,
    request: &MaterializationRequest<Id>,
) -> Result<MaterializationPlan<Id>, CollectionDiagnostic> {
    super::snapshot::validate_request(snapshot, request)?;
    debug_assert_eq!(snapshot.generation, request.collection_generation);
    debug_assert!(snapshot.items.len() <= MAX_COLLECTION_ITEMS);
    let identity_index: std::collections::BTreeMap<&Id, usize> =
        snapshot.items.iter().enumerate().map(|(index, id)| (id, index)).collect();
    let extents = ExtentIndex::new(request, &identity_index)?;
    let total_extent = extents.offset(snapshot.items.len())?;
    let visible_boundary = checked_add(LogicalSum {
        left_operand: request.viewport_start,
        right_operand: request.viewport_extent.get(),
        subject: "viewport-end",
    })?;
    let materialization_lower = (request.viewport_start - request.overscan).max(0.0);
    let materialization_upper = checked_add(LogicalSum {
        left_operand: visible_boundary,
        right_operand: request.overscan,
        subject: "overscan-end",
    })?;
    let (visible, materialized) = project_materialized_items(snapshot, request, &extents, MaterializationBounds {
        lower: materialization_lower,
        item_boundary: materialization_upper,
        visible_boundary,
    })?;
    let before_extent = materialized.first().map_or(total_extent.min(materialization_lower), |item| item.offset);
    let projected_boundary = materialized.last().map_or(before_extent, |item| item.offset + item.extent);
    let anchor = project_anchor(request, &identity_index, &extents)?;
    let scroll_request = project_scroll(request, &identity_index, &extents)?;
    Ok(MaterializationPlan {
        collection_generation: snapshot.generation,
        measurement_generation: request.measurement_generation,
        visible,
        materialized,
        before_extent,
        after_extent: (total_extent - projected_boundary).max(0.0),
        total_extent,
        anchor,
        scroll_request,
    })
}

struct MaterializationBounds {
    lower: f64,
    item_boundary: f64,
    visible_boundary: f64,
}

fn project_materialized_items<Id: Clone + Ord>(
    snapshot: &CollectionSnapshot<Id>,
    request: &MaterializationRequest<Id>,
    extents: &ExtentIndex,
    bounds: MaterializationBounds,
) -> Result<(Vec<Id>, Vec<MaterializedItem<Id>>), CollectionDiagnostic> {
    debug_assert!(bounds.lower >= 0.0);
    debug_assert!(bounds.item_boundary >= bounds.lower);
    let start_index = extents.first_intersecting(bounds.lower, snapshot.items.len())?;
    let reservation = snapshot.items.len().min(4_096);
    let mut visible = Vec::with_capacity(reservation);
    let mut materialized = Vec::with_capacity(reservation);
    for index in start_index..snapshot.items.len() {
        let item_origin = extents.offset(index)?;
        if item_origin >= bounds.item_boundary {
            break;
        }
        let item_span = extents.extent(index);
        let item_boundary = checked_add(LogicalSum {
            left_operand: item_origin,
            right_operand: item_span,
            subject: "item-end",
        })?;
        let is_visible = item_boundary > request.viewport_start && item_origin < bounds.visible_boundary;
        if is_visible {
            visible.push(snapshot.items[index].clone());
        }
        if item_boundary > bounds.lower {
            materialized.push(MaterializedItem {
                id: snapshot.items[index].clone(),
                index,
                offset: item_origin,
                extent: item_span,
                measured: extents.measured(index),
                visible: is_visible,
            });
        }
    }
    Ok((visible, materialized))
}

struct ExtentIndex {
    default_extent: f64,
    uniform: bool,
    measured: std::collections::BTreeMap<usize, f64>,
    prefix_adjustments: Vec<(usize, f64)>,
}

impl ExtentIndex {
    fn new<Id: Ord>(
        request: &MaterializationRequest<Id>,
        identities: &std::collections::BTreeMap<&Id, usize>,
    ) -> Result<Self, CollectionDiagnostic> {
        let default_extent = request.uniform_extent.unwrap_or(request.estimated_extent).get();
        let mut measured = std::collections::BTreeMap::new();
        for measurement in &request.measurements {
            let Some(index) = identities.get(&measurement.id) else {
                return Err(CollectionDiagnostic::new(CollectionDiagnosticCode::UnknownIdentity, "measurement"));
            };
            if measured.insert(*index, measurement.extent.get()).is_some() {
                return Err(CollectionDiagnostic::new(CollectionDiagnosticCode::DuplicateMeasurement, "measurement"));
            }
        }
        let mut adjustment = 0.0_f64;
        let mut prefix_adjustments = Vec::with_capacity(measured.len());
        for (index, extent) in &measured {
            adjustment += extent - default_extent;
            if !adjustment.is_finite() {
                return Err(CollectionDiagnostic::new(
                    CollectionDiagnosticCode::InvalidExtent,
                    "measurement-adjustment",
                ));
            }
            prefix_adjustments.push((*index, adjustment));
        }
        Ok(Self {
            default_extent,
            uniform: request.uniform_extent.is_some(),
            measured,
            prefix_adjustments,
        })
    }

    fn extent(&self, index: usize) -> f64 {
        self.measured.get(&index).copied().unwrap_or(self.default_extent)
    }

    fn measured(&self, index: usize) -> bool {
        self.uniform || self.measured.contains_key(&index)
    }

    fn offset(&self, index: usize) -> Result<f64, CollectionDiagnostic> {
        let base = (index as f64) * self.default_extent;
        let prior_count = self.prefix_adjustments.partition_point(|(measured_index, _)| *measured_index < index);
        let adjustment = prior_count
            .checked_sub(1)
            .and_then(|prior| self.prefix_adjustments.get(prior))
            .map_or(0.0, |(_, adjustment)| *adjustment);
        checked_add(LogicalSum {
            left_operand: base,
            right_operand: adjustment,
            subject: "item-offset",
        })
    }

    fn first_intersecting(&self, logical_start: f64, item_count: usize) -> Result<usize, CollectionDiagnostic> {
        let mut low = 0;
        let mut high = item_count;
        while low < high {
            let middle = low.saturating_add(high.saturating_sub(low) / 2);
            let item_boundary = checked_add(LogicalSum {
                left_operand: self.offset(middle)?,
                right_operand: self.extent(middle),
                subject: "item-end",
            })?;
            if item_boundary <= logical_start {
                low = middle.saturating_add(1);
            } else {
                high = middle;
            }
        }
        Ok(low)
    }
}

fn project_anchor<Id: Clone + Ord>(
    request: &MaterializationRequest<Id>,
    identities: &std::collections::BTreeMap<&Id, usize>,
    extents: &ExtentIndex,
) -> Result<Option<AnchorProjection<Id>>, CollectionDiagnostic> {
    let Some(anchor) = &request.anchor else {
        return Ok(None);
    };
    let Some(index) = identities.get(&anchor.id) else {
        return Err(CollectionDiagnostic::new(CollectionDiagnosticCode::UnknownIdentity, "anchor"));
    };
    let item_origin = extents.offset(*index)?;
    Ok(Some(AnchorProjection {
        id: anchor.id.clone(),
        item_offset: item_origin,
        viewport_reference: anchor.viewport_reference,
        scroll_correction: item_origin - anchor.previous_item_offset,
    }))
}

fn project_scroll<Id: Clone + Ord>(
    request: &MaterializationRequest<Id>,
    identities: &std::collections::BTreeMap<&Id, usize>,
    extents: &ExtentIndex,
) -> Result<Option<ScrollRequest<Id>>, CollectionDiagnostic> {
    let Some((id, alignment)) = &request.scroll_to else {
        return Ok(None);
    };
    let Some(index) = identities.get(id) else {
        return Err(CollectionDiagnostic::new(CollectionDiagnosticCode::MissingScrollTarget, "scroll-to"));
    };
    debug_assert!(request.viewport_extent.get().is_finite());
    debug_assert!(request.viewport_extent.get() > 0.0);
    let item_origin = extents.offset(*index)?;
    let item_span = extents.extent(*index);
    let available_space = request.viewport_extent.get();
    let item_boundary = checked_add(LogicalSum {
        left_operand: item_origin,
        right_operand: item_span,
        subject: "scroll-item-end",
    })?;
    let current_boundary = checked_add(LogicalSum {
        left_operand: request.viewport_start,
        right_operand: available_space,
        subject: "scroll-viewport-end",
    })?;
    let target_origin = match alignment {
        ScrollAlignment::Start => item_origin,
        ScrollAlignment::Center => item_origin - (available_space - item_span) / 2.0,
        ScrollAlignment::End => item_boundary - available_space,
        ScrollAlignment::Nearest if item_origin < request.viewport_start => item_origin,
        ScrollAlignment::Nearest if item_boundary > current_boundary => item_boundary - available_space,
        ScrollAlignment::Nearest => request.viewport_start,
    };
    Ok(Some(ScrollRequest {
        id: id.clone(),
        viewport_start: target_origin.max(0.0),
        alignment: *alignment,
    }))
}

struct LogicalSum<'a> {
    left_operand: f64,
    right_operand: f64,
    subject: &'a str,
}

fn checked_add(input: LogicalSum<'_>) -> Result<f64, CollectionDiagnostic> {
    let result = input.left_operand + input.right_operand;
    result
        .is_finite()
        .then_some(result)
        .ok_or_else(|| CollectionDiagnostic::new(CollectionDiagnosticCode::InvalidExtent, input.subject))
}
