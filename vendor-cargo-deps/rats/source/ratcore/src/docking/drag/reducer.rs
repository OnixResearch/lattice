/// Reduces a drag event into session state without mutating layout.
#[must_use]
pub fn reduce_drag_resize_session(
    previous: Option<&DragSession>,
    event: &DragEvent,
    targets: &[DragTarget],
) -> DragTransition {
    match &event.phase {
        DragPhase::Start => start_session(event, targets),
        DragPhase::Update | DragPhase::Preview => update_session(previous, event),
        DragPhase::Cancel => cancel_session(previous, event),
        DragPhase::Commit => commit_session_transition(previous, event),
        DragPhase::Unsupported(phase) => rejected(
            previous.cloned(),
            DragDiagnostic::new(
                DragDiagnosticCode::UnsupportedPhase,
                Some(event.target_id.clone()),
                format!("unsupported drag phase: {phase}"),
            ),
        ),
    }
}

/// Computes preview metadata from a session and explicit drop zones.
#[must_use]
pub fn compute_drag_resize_preview(
    session: &DragSession,
    zones: &[DropZoneCandidate],
) -> DragPreview {
    let mut candidates = zones.to_vec();
    candidates.sort_by_key(|candidate| (std::cmp::Reverse(candidate.priority), candidate.zone_id.clone()));
    let selected = candidates.iter().find(|candidate| candidate.allowed);
    let diagnostic = selected.is_none().then(|| {
        DragDiagnostic::new(
            DragDiagnosticCode::InvalidCommit,
            Some(session.target_id.clone()),
            "no allowed drop zone for drag preview",
        )
    });
    let preview = DragPreview {
        target_id: session.target_id.clone(),
        selected_zone_id: selected.map(|candidate| candidate.zone_id.clone()),
        ghost: selected.map_or(
            DragRect::new(session.current.x, session.current.y, 0, 0),
            |candidate| candidate.bounds,
        ),
        allowed: selected.is_some(),
        candidates,
        diagnostic,
    };
    debug_assert_eq!(preview.allowed, preview.selected_zone_id.is_some());
    debug_assert!(preview.allowed || preview.diagnostic.is_some());
    preview
}

/// Validates and converts an accepted session into a docking intent description.
#[must_use]
pub fn commit_drag_resize(request: DragCommitRequest) -> DragCommitOutcome {
    if request.session.status == DragSessionStatus::Canceled {
        return DragCommitOutcome {
            intent: None,
            diagnostic: Some(DragDiagnostic::new(
                DragDiagnosticCode::Canceled,
                Some(request.session.target_id),
                "cannot commit a canceled drag session",
            )),
        };
    }
    if request.session.status != DragSessionStatus::Committed {
        return DragCommitOutcome {
            intent: None,
            diagnostic: Some(DragDiagnostic::new(
                DragDiagnosticCode::InvalidCommit,
                Some(request.session.target_id),
                "drag session was not committed by the reducer",
            )),
        };
    }
    let diagnostic = validate_commit_operation(&request.operation);
    if let Some(diagnostic) = diagnostic {
        return DragCommitOutcome {
            intent: None,
            diagnostic: Some(diagnostic),
        };
    }
    debug_assert_eq!(request.session.status, DragSessionStatus::Committed);
    debug_assert!(validate_commit_operation(&request.operation).is_none());
    DragCommitOutcome {
        intent: Some(DragDockIntent {
            renderer: request.renderer,
            operation: request.operation,
        }),
        diagnostic: None,
    }
}

/// Creates a progress ratio from percent-like input for callers that expose sliders.
///
/// # Errors
///
/// Returns [`DragDiagnostic`] when the value is outside the accepted non-edge range.
pub fn resize_ratio_from_percent(percent: u16) -> Result<ResizeRatio, DragDiagnostic> {
    ResizeRatio::new(percent, DEFAULT_PROGRESS_DENOMINATOR).map_err(|_| {
        DragDiagnostic::new(
            DragDiagnosticCode::ImpossibleResize,
            None,
            "resize ratio would collapse a pane",
        )
    })
}

fn start_session(event: &DragEvent, targets: &[DragTarget]) -> DragTransition {
    debug_assert_eq!(event.phase, DragPhase::Start);
    let Some(target) = targets
        .iter()
        .find(|target| target.target_id == event.target_id)
    else {
        return rejected(
            None,
            DragDiagnostic::new(
                DragDiagnosticCode::StaleTarget,
                Some(event.target_id.clone()),
                "drag target is absent",
            ),
        );
    };
    debug_assert_eq!(target.target_id, event.target_id);
    if !target.enabled {
        return rejected(
            None,
            DragDiagnostic::new(
                DragDiagnosticCode::DisabledAffordance,
                Some(target.target_id.clone()),
                target
                    .diagnostic
                    .clone()
                    .unwrap_or_else(|| "drag target is disabled".to_string()),
            ),
        );
    }
    DragTransition {
        session: Some(DragSession {
            target_id: target.target_id.clone(),
            kind: target.kind,
            start: event.position,
            current: event.position,
            status: DragSessionStatus::Active,
        }),
        diagnostic: None,
    }
}

fn update_session(previous: Option<&DragSession>, event: &DragEvent) -> DragTransition {
    debug_assert!(matches!(event.phase, DragPhase::Update | DragPhase::Preview));
    let Some(session) = previous else {
        return rejected(
            None,
            DragDiagnostic::new(
                DragDiagnosticCode::MissingStart,
                Some(event.target_id.clone()),
                "drag update arrived without a start",
            ),
        );
    };
    if session.target_id != event.target_id {
        return rejected(
            Some(session.clone()),
            DragDiagnostic::new(
                DragDiagnosticCode::StaleTarget,
                Some(event.target_id.clone()),
                "drag update target does not match session target",
            ),
        );
    }
    let status = if event.phase == DragPhase::Preview {
        DragSessionStatus::Previewing
    } else {
        DragSessionStatus::Active
    };
    debug_assert!(matches!(
        status,
        DragSessionStatus::Active | DragSessionStatus::Previewing
    ));
    DragTransition {
        session: Some(DragSession {
            current: event.position,
            status,
            ..session.clone()
        }),
        diagnostic: None,
    }
}

fn cancel_session(previous: Option<&DragSession>, event: &DragEvent) -> DragTransition {
    debug_assert_eq!(event.phase, DragPhase::Cancel);
    let Some(session) = previous else {
        return rejected(
            None,
            DragDiagnostic::new(
                DragDiagnosticCode::MissingStart,
                Some(event.target_id.clone()),
                "drag cancel arrived without a start",
            ),
        );
    };
    let canceled_session = DragSession {
        status: DragSessionStatus::Canceled,
        ..session.clone()
    };
    debug_assert_eq!(canceled_session.status, DragSessionStatus::Canceled);
    DragTransition {
        session: Some(canceled_session),
        diagnostic: Some(DragDiagnostic::new(
            DragDiagnosticCode::Canceled,
            Some(session.target_id.clone()),
            "drag session canceled",
        )),
    }
}

fn commit_session_transition(previous: Option<&DragSession>, event: &DragEvent) -> DragTransition {
    debug_assert_eq!(event.phase, DragPhase::Commit);
    let Some(session) = previous else {
        return rejected(
            None,
            DragDiagnostic::new(
                DragDiagnosticCode::MissingStart,
                Some(event.target_id.clone()),
                "drag commit arrived without a start",
            ),
        );
    };
    if session.target_id != event.target_id {
        return rejected(
            Some(session.clone()),
            DragDiagnostic::new(
                DragDiagnosticCode::StaleTarget,
                Some(event.target_id.clone()),
                "drag commit target does not match session target",
            ),
        );
    }
    let committed_session = DragSession {
        current: event.position,
        status: DragSessionStatus::Committed,
        ..session.clone()
    };
    debug_assert_eq!(committed_session.status, DragSessionStatus::Committed);
    DragTransition {
        session: Some(committed_session),
        diagnostic: None,
    }
}

fn validate_commit_operation(operation: &DragDockOperation) -> Option<DragDiagnostic> {
    match operation {
        DragDockOperation::ResizeSplitter(_) => None,
        DragDockOperation::ReorderTab {
            tab_id,
            destination_id,
        } => validate_non_empty_pair((tab_id, destination_id)),
        DragDockOperation::DockPanel { panel_id, zone_id } => {
            validate_non_empty(panel_id.as_str()).or_else(|| validate_non_empty(zone_id.as_str()))
        }
        DragDockOperation::MoveFloatingPanel { panel_id, .. }
        | DragDockOperation::ResizeFrame { panel_id, .. } => validate_non_empty(panel_id.as_str()),
    }
}

fn validate_non_empty_pair(values: (&str, &str)) -> Option<DragDiagnostic> {
    let (left, right) = values;
    validate_non_empty(left).or_else(|| validate_non_empty(right))
}

fn validate_non_empty(value: &str) -> Option<DragDiagnostic> {
    value.trim().is_empty().then(|| {
        DragDiagnostic::new(
            DragDiagnosticCode::InvalidCommit,
            None,
            "commit operation contains an empty identifier",
        )
    })
}

fn rejected(session: Option<DragSession>, diagnostic: DragDiagnostic) -> DragTransition {
    DragTransition {
        session,
        diagnostic: Some(diagnostic),
    }
}
