#[cfg(test)]
mod tests {
    use super::*;

    const START_X: i32 = 4;
    const START_Y: i32 = 6;
    const UPDATE_X: i32 = 10;
    const UPDATE_Y: i32 = 12;
    const RECT_X: i32 = 0;
    const RECT_Y: i32 = 0;
    const RECT_WIDTH: i32 = 20;
    const RECT_HEIGHT: i32 = 10;
    const HIGH_PRIORITY: i32 = 10;
    const LOW_PRIORITY: i32 = 1;
    const VALID_RESIZE_PERCENT: u16 = 40;

    fn point() -> DragPoint {
        DragPoint::new(START_X, START_Y)
    }

    fn updated_point() -> DragPoint {
        DragPoint::new(UPDATE_X, UPDATE_Y)
    }

    fn rect() -> DragRect {
        DragRect::new(RECT_X, RECT_Y, RECT_WIDTH, RECT_HEIGHT)
    }

    fn target() -> DragTarget {
        DragTarget::new("splitter", DragAffordanceKind::Splitter, rect())
    }

    #[test]
    fn drag_session_start_update_preview_and_commit_are_pure() {
        let start = DragEvent::new(DragPhase::Start, "splitter", point());
        let transition = reduce_drag_resize_session(None, &start, &[target()]);
        let Some(session) = transition.session else {
            panic!("start should create a session");
        };
        let update = DragEvent::new(DragPhase::Update, "splitter", updated_point());
        let updated = reduce_drag_resize_session(Some(&session), &update, &[target()]);
        let Some(updated_session) = updated.session else {
            panic!("update should keep a session");
        };
        let commit = DragEvent::new(DragPhase::Commit, "splitter", updated_point());
        let committed = reduce_drag_resize_session(Some(&updated_session), &commit, &[target()]);

        assert!(transition.diagnostic.is_none());
        assert_eq!(updated_session.current, updated_point());
        assert_eq!(
            committed.session.map(|session| session.status),
            Some(DragSessionStatus::Committed)
        );
    }

    #[test]
    fn stale_disabled_missing_and_unknown_phase_events_fail_closed() {
        let disabled_target = target().with_enabled(false);
        let start_disabled = DragEvent::new(DragPhase::Start, "splitter", point());
        let missing_update = DragEvent::new(DragPhase::Update, "splitter", point());
        let unknown_phase_event = DragEvent::new(
            DragPhase::Unsupported("hover-drag".to_string()),
            "splitter",
            point(),
        );
        let stale = DragEvent::new(DragPhase::Start, "missing", point());

        let disabled = reduce_drag_resize_session(None, &start_disabled, &[disabled_target]);
        let missing = reduce_drag_resize_session(None, &missing_update, &[target()]);
        let unknown_phase_result =
            reduce_drag_resize_session(None, &unknown_phase_event, &[target()]);
        let stale = reduce_drag_resize_session(None, &stale, &[target()]);

        assert!(matches!(
            disabled.diagnostic,
            Some(DragDiagnostic {
                code: DragDiagnosticCode::DisabledAffordance,
                ..
            })
        ));
        assert!(matches!(
            missing.diagnostic,
            Some(DragDiagnostic {
                code: DragDiagnosticCode::MissingStart,
                ..
            })
        ));
        assert!(matches!(
            unknown_phase_result.diagnostic,
            Some(DragDiagnostic {
                code: DragDiagnosticCode::UnsupportedPhase,
                ..
            })
        ));
        assert!(matches!(
            stale.diagnostic,
            Some(DragDiagnostic {
                code: DragDiagnosticCode::StaleTarget,
                ..
            })
        ));
    }

    #[test]
    fn preview_ranks_stable_candidates_and_rejects_empty_set() {
        let session = DragSession {
            target_id: "panel".to_string(),
            kind: DragAffordanceKind::Panel,
            start: point(),
            current: updated_point(),
            status: DragSessionStatus::Previewing,
        };
        let preview = compute_drag_resize_preview(
            &session,
            &[
                DropZoneCandidate::new("right", rect(), LOW_PRIORITY),
                DropZoneCandidate::new("left", rect(), HIGH_PRIORITY),
            ],
        );
        let rejected = compute_drag_resize_preview(
            &session,
            &[DropZoneCandidate::new("blocked", rect(), HIGH_PRIORITY).with_allowed(false)],
        );

        assert_eq!(
            preview.selected_zone_id,
            Some(crate::docking::DockNodeId::new("left"))
        );
        assert!(preview.allowed);
        assert!(!rejected.allowed);
        assert!(matches!(
            rejected.diagnostic,
            Some(DragDiagnostic {
                code: DragDiagnosticCode::InvalidCommit,
                ..
            })
        ));
    }

    #[test]
    fn commit_emits_intent_or_rejects_invalid_and_canceled_sessions() {
        let ratio = resize_ratio_from_percent(VALID_RESIZE_PERCENT)
            .expect("named test percent should be a valid resize ratio");
        let committed_session = DragSession {
            target_id: "splitter".to_string(),
            kind: DragAffordanceKind::Splitter,
            start: point(),
            current: updated_point(),
            status: DragSessionStatus::Committed,
        };
        let accepted = commit_drag_resize(DragCommitRequest {
            renderer: DragRenderer::Subwayrat,
            session: committed_session.clone(),
            operation: DragDockOperation::ResizeSplitter(ratio),
        });
        let canceled = commit_drag_resize(DragCommitRequest {
            renderer: DragRenderer::Meteorite,
            session: DragSession {
                status: DragSessionStatus::Canceled,
                ..committed_session
            },
            operation: DragDockOperation::DockPanel {
                panel_id: crate::docking::PanelId::new("panel"),
                zone_id: crate::docking::DockNodeId::new("left"),
            },
        });

        assert!(accepted.intent.is_some());
        assert!(accepted.diagnostic.is_none());
        assert!(matches!(
            canceled.diagnostic,
            Some(DragDiagnostic {
                code: DragDiagnosticCode::Canceled,
                ..
            })
        ));
    }

    #[test]
    fn impossible_resize_is_rejected_before_commit() {
        let rejected = resize_ratio_from_percent(DEFAULT_PROGRESS_DENOMINATOR)
            .expect_err("edge ratio must fail before a commit operation exists");

        assert_eq!(rejected.code, DragDiagnosticCode::ImpossibleResize);
        assert_eq!(rejected.target_id, None);
    }
}
