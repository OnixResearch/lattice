use super::*;

const DIALOG_SURFACE: &str = "met-widgets.dialog-frame";
const OVERLAY_SURFACE: &str = "met-widgets.overlay-frame";
const POPOVER_SURFACE: &str = "met-overlay.popover";
const LEADER_SURFACE: &str = "met-leaderkey.overlay";
const OWNER_WIDGETS: &str = "met-widgets::chrome";
const OWNER_OVERLAY: &str = "met-overlay";
const OWNER_LEADER: &str = "met-leaderkey";
const EVIDENCE_CHROME: &str = "meteorite/crates/met-showcase/src/chrome.rs";
const EVIDENCE_OVERLAY: &str = "meteorite/crates/met-overlay/src/popover.rs";
const EVIDENCE_LEADER: &str = "meteorite/crates/met-leaderkey/src/overlay.rs";
const WEB_NATIVE_REASON: &str = "browser owns anchoring, portals, focus, and aria behavior";
const RENDERER_OWNED_REASON: &str = "adapter consumes shared state but owns DOM shell";
const KEEP_FRESH_ACTION: &str = "keep evidence fresh when overlay surface changes";

fn shared_surface(
    surface_id: &str,
    shared_chrome_kind: OverlayChromeKind,
) -> MeteoriteOverlaySurfaceEvidence {
    MeteoriteOverlaySurfaceEvidence {
        surface_id: surface_id.to_owned(),
        owner: OWNER_WIDGETS.to_owned(),
        role: MeteoriteOverlayRole::SharedChromeFrame,
        shared_chrome_kind: Some(shared_chrome_kind),
        evidence: EVIDENCE_CHROME.to_owned(),
        reason: String::new(),
        next_action: KEEP_FRESH_ACTION.to_owned(),
    }
}

fn web_native_surface(surface_id: &str) -> MeteoriteOverlaySurfaceEvidence {
    MeteoriteOverlaySurfaceEvidence {
        surface_id: surface_id.to_owned(),
        owner: OWNER_OVERLAY.to_owned(),
        role: MeteoriteOverlayRole::WebNativePrimitive,
        shared_chrome_kind: None,
        evidence: EVIDENCE_OVERLAY.to_owned(),
        reason: WEB_NATIVE_REASON.to_owned(),
        next_action: KEEP_FRESH_ACTION.to_owned(),
    }
}

fn renderer_owned_surface(surface_id: &str) -> MeteoriteOverlaySurfaceEvidence {
    MeteoriteOverlaySurfaceEvidence {
        surface_id: surface_id.to_owned(),
        owner: OWNER_LEADER.to_owned(),
        role: MeteoriteOverlayRole::RendererOwnedAdapter,
        shared_chrome_kind: None,
        evidence: EVIDENCE_LEADER.to_owned(),
        reason: RENDERER_OWNED_REASON.to_owned(),
        next_action: KEEP_FRESH_ACTION.to_owned(),
    }
}

fn has_code(
    report: &MeteoriteOverlaySemanticsReport,
    code: MeteoriteOverlayDiagnosticCode,
) -> bool {
    report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == code)
}

#[test]
fn valid_overlay_semantics_classify_shared_web_native_and_renderer_owned_rows() {
    let report = validate_meteorite_overlay_semantics(&[
        shared_surface(OVERLAY_SURFACE, OverlayChromeKind::OverlayFrame),
        shared_surface(DIALOG_SURFACE, OverlayChromeKind::DialogFrame),
        web_native_surface(POPOVER_SURFACE),
        renderer_owned_surface(LEADER_SURFACE),
    ]);

    assert!(report.is_valid(), "diagnostics: {:?}", report.diagnostics);
    assert_eq!(
        report.shared_surface_ids,
        vec![DIALOG_SURFACE.to_owned(), OVERLAY_SURFACE.to_owned()]
    );
    assert_eq!(
        report.web_native_surface_ids,
        vec![POPOVER_SURFACE.to_owned()]
    );
    assert_eq!(
        report.renderer_owned_surface_ids,
        vec![LEADER_SURFACE.to_owned()]
    );
}

#[test]
fn malformed_overlay_semantics_report_required_field_and_role_diagnostics() {
    let missing_required_fields = MeteoriteOverlaySurfaceEvidence {
        surface_id: String::new(),
        owner: String::new(),
        role: MeteoriteOverlayRole::SharedChromeFrame,
        shared_chrome_kind: None,
        evidence: String::new(),
        reason: String::new(),
        next_action: String::new(),
    };
    let duplicate = shared_surface(OVERLAY_SURFACE, OverlayChromeKind::OverlayFrame);
    let invalid_shared_kind = shared_surface(OVERLAY_SURFACE, OverlayChromeKind::TitleBar);
    let invalid_web_native = MeteoriteOverlaySurfaceEvidence {
        surface_id: POPOVER_SURFACE.to_owned(),
        owner: OWNER_OVERLAY.to_owned(),
        role: MeteoriteOverlayRole::WebNativePrimitive,
        shared_chrome_kind: Some(OverlayChromeKind::DialogFrame),
        evidence: EVIDENCE_OVERLAY.to_owned(),
        reason: String::new(),
        next_action: KEEP_FRESH_ACTION.to_owned(),
    };

    let report = validate_meteorite_overlay_semantics(&[
        missing_required_fields,
        duplicate,
        invalid_shared_kind,
        invalid_web_native,
    ]);

    assert!(!report.is_valid());
    assert!(has_code(
        &report,
        MeteoriteOverlayDiagnosticCode::EmptySurfaceId
    ));
    assert!(has_code(
        &report,
        MeteoriteOverlayDiagnosticCode::DuplicateSurfaceId
    ));
    assert!(has_code(
        &report,
        MeteoriteOverlayDiagnosticCode::EmptyOwner
    ));
    assert!(has_code(
        &report,
        MeteoriteOverlayDiagnosticCode::MissingEvidence
    ));
    assert!(has_code(
        &report,
        MeteoriteOverlayDiagnosticCode::MissingNextAction
    ));
    assert!(has_code(
        &report,
        MeteoriteOverlayDiagnosticCode::MissingSharedChromeKind
    ));
    assert!(has_code(
        &report,
        MeteoriteOverlayDiagnosticCode::UnsupportedSharedChromeKind
    ));
    assert!(has_code(
        &report,
        MeteoriteOverlayDiagnosticCode::MissingReason
    ));
    assert!(has_code(
        &report,
        MeteoriteOverlayDiagnosticCode::UnexpectedSharedChromeKind
    ));
}
