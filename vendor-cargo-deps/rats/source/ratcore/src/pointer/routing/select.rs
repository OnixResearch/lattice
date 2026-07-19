/// Normalizes a pointer event or reports unsupported input.
///
/// # Errors
///
/// Returns a deterministic diagnostic when source, phase, button, button state,
/// wheel unit, coordinate space, or click metadata is unsupported.
pub fn normalize_pointer_event(
    event: super::input::PointerEvent,
) -> Result<super::input::PointerEvent, super::result::PointerDiagnostic> {
    assert_normalize_shape(&event);
    if let Some(kind) = super::diagnose::unsupported_event_kind(&event) {
        return Err(super::diagnose::unsupported_diagnostic(kind));
    }
    Ok(event)
}

/// Selects a route from a pointer event and immutable hit map.
#[must_use]
pub fn route_pointer_event(
    event: &super::input::PointerEvent,
    hit_map: &super::area::HitMap,
) -> super::result::PointerRouteOutcome {
    if let Err(diagnostic) = normalize_pointer_event(event.clone()) {
        return super::result::PointerRouteOutcome::Diagnostic(diagnostic);
    }
    if let Some(target_id) = &event.captured_target {
        return route_captured_target(event, target_id, hit_map);
    }
    route_position(event, hit_map)
}

/// Projects a renderer-specific pointer event into the shared route outcome.
#[must_use]
pub fn project_renderer_pointer_route(
    renderer: super::result::PointerRenderer,
    event: &super::input::PointerEvent,
    hit_map: &super::area::HitMap,
) -> super::result::PointerAdapterProjection {
    super::result::PointerAdapterProjection {
        renderer,
        outcome: route_pointer_event(event, hit_map),
    }
}

/// Creates a stable unsupported-kind label for adapters without native labels.
#[must_use]
pub fn default_unsupported_pointer_kind() -> String {
    super::diagnose::default_kind_label()
}

fn route_captured_target(
    event: &super::input::PointerEvent,
    target_id: &str,
    hit_map: &super::area::HitMap,
) -> super::result::PointerRouteOutcome {
    let Some(region) = hit_map.target(target_id) else {
        return super::result::PointerRouteOutcome::Diagnostic(
            super::result::PointerDiagnostic::new(
                super::result::PointerDiagnosticCode::StaleTarget,
                Some(target_id.to_string()),
                "captured target is no longer present",
            ),
        );
    };
    route_region(event, region)
}

fn route_position(
    event: &super::input::PointerEvent,
    hit_map: &super::area::HitMap,
) -> super::result::PointerRouteOutcome {
    assert_hit_map_shape(hit_map);
    let mut candidates = matching_regions(event.position, hit_map);
    candidates.sort_by(|left, right| right.z_order.cmp(&left.z_order));
    let deferred = deferred_route_candidate(&candidates);
    for region in candidates {
        if route_candidate_is_skipped(region) {
            continue;
        }
        return route_region(event, region);
    }
    deferred_outcome(event, deferred)
}

fn route_region(
    event: &super::input::PointerEvent,
    region: &super::area::HitRegion,
) -> super::result::PointerRouteOutcome {
    if !region.visible {
        return super::diagnose::hidden_diagnostic(region);
    }
    if !region.enabled {
        return super::diagnose::disabled_diagnostic(region);
    }
    let interaction = super::intent::classify_interaction(event, region.capture);
    super::result::PointerRouteOutcome::Routed(super::result::PointerRoute {
        target_id: region.target_id.clone(),
        action_id: region.action_id.clone(),
        modal_capture: region.capture == super::area::HitCapturePolicy::ModalCapture,
        interaction,
        capture: super::intent::capture_transition(event, region.capture, interaction),
    })
}

fn pass_through_route(
    event: &super::input::PointerEvent,
    region: &super::area::HitRegion,
) -> super::result::PointerRouteOutcome {
    super::result::PointerRouteOutcome::Routed(super::result::PointerRoute {
        target_id: region.target_id.clone(),
        action_id: None,
        modal_capture: false,
        interaction: super::result::PointerInteractionClass::PassThrough,
        capture: super::intent::capture_transition(
            event,
            super::area::HitCapturePolicy::PassThrough,
            super::result::PointerInteractionClass::PassThrough,
        ),
    })
}

fn deferred_route_candidate<'a>(
    candidates: &[&'a super::area::HitRegion],
) -> Option<DeferredCandidate<'a>> {
    let mut deferred_disabled = None;
    let mut deferred_hidden = None;
    let mut deferred_pass_through = None;
    for region in candidates {
        if !region.visible {
            deferred_hidden = Some(*region);
        } else if region.capture == super::area::HitCapturePolicy::PassThrough {
            deferred_pass_through = Some(*region);
        } else if !region.enabled {
            deferred_disabled = Some(*region);
        }
    }
    deferred_disabled
        .map(DeferredCandidate::Disabled)
        .or_else(|| deferred_hidden.map(DeferredCandidate::Hidden))
        .or_else(|| deferred_pass_through.map(DeferredCandidate::PassThrough))
}

fn deferred_outcome(
    event: &super::input::PointerEvent,
    deferred: Option<DeferredCandidate<'_>>,
) -> super::result::PointerRouteOutcome {
    match deferred {
        Some(DeferredCandidate::Disabled(region)) => super::diagnose::disabled_diagnostic(region),
        Some(DeferredCandidate::Hidden(region)) => super::diagnose::hidden_diagnostic(region),
        Some(DeferredCandidate::PassThrough(region)) => pass_through_route(event, region),
        None => {
            super::result::PointerRouteOutcome::Diagnostic(super::result::PointerDiagnostic::new(
                super::result::PointerDiagnosticCode::NoTarget,
                None,
                "no hit region contains the pointer event",
            ))
        }
    }
}

fn route_candidate_is_skipped(region: &super::area::HitRegion) -> bool {
    !region.visible
        || region.capture == super::area::HitCapturePolicy::PassThrough
        || !region.enabled
}

fn matching_regions(
    position: super::shape::PointerPoint,
    hit_map: &super::area::HitMap,
) -> Vec<&super::area::HitRegion> {
    hit_map
        .regions
        .iter()
        .filter(|region| region.bounds.contains(position))
        .collect()
}

fn assert_normalize_shape(event: &super::input::PointerEvent) {
    debug_assert!(
        event
            .captured_target
            .as_ref()
            .is_none_or(|target| !target.contains('\n'))
    );
    debug_assert!(
        event
            .buttons
            .unsupported
            .iter()
            .all(|button| !button.contains('\n'))
    );
}

fn assert_hit_map_shape(hit_map: &super::area::HitMap) {
    debug_assert!(
        hit_map
            .regions
            .iter()
            .all(|region| !region.target_id.is_empty())
    );
    debug_assert!(
        hit_map
            .regions
            .iter()
            .all(|region| region.bounds.width > i32::MIN)
    );
}

#[derive(Clone, Copy)]
enum DeferredCandidate<'a> {
    Disabled(&'a super::area::HitRegion),
    Hidden(&'a super::area::HitRegion),
    PassThrough(&'a super::area::HitRegion),
}
