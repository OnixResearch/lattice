//! Pure evidence model for Meteorite overlay semantics parity.
//!
//! The model distinguishes shared chrome frames from browser-native or
//! renderer-owned overlays without forcing a DOM overlay migration.

#[path = "spec.rs"]
mod spec;

semantic_string_id! {
    /// Stable Meteorite overlay evidence surface identifier.
    pub struct MeteoriteOverlaySurfaceId;
}

/// Shared chrome kind used by overlay evidence rows.
pub type OverlayChromeKind = crate::meteorite_chrome_core::ChromeSurfaceKind;

type SurfaceIdSet = std::collections::BTreeSet<MeteoriteOverlaySurfaceId>;

/// Meteorite overlay ownership role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeteoriteOverlayRole {
    /// Surface uses shared chrome frame semantics.
    SharedChromeFrame,
    /// Surface relies on browser-native overlay behavior.
    WebNativePrimitive,
    /// Surface adapts shared state but owns renderer-specific overlay shell.
    RendererOwnedAdapter,
}

/// Meteorite overlay evidence row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeteoriteOverlaySurfaceEvidence {
    /// Stable surface identifier.
    pub surface_id: MeteoriteOverlaySurfaceId,
    /// Owning crate or module.
    pub owner: String,
    /// Overlay ownership role.
    pub role: MeteoriteOverlayRole,
    /// Shared chrome kind for shared frame rows.
    pub shared_chrome_kind: Option<OverlayChromeKind>,
    /// Evidence path, command, or module proving the row.
    pub evidence: String,
    /// Reason for web-native or renderer-owned ownership.
    pub reason: String,
    /// Next maintenance action for this surface.
    pub next_action: String,
}

/// Field associated with an overlay evidence diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeteoriteOverlayField {
    /// Surface identifier field.
    SurfaceId,
    /// Owner field.
    Owner,
    /// Shared chrome kind field.
    SharedChromeKind,
    /// Evidence field.
    Evidence,
    /// Reason field.
    Reason,
    /// Next action field.
    NextAction,
}

/// Meteorite overlay evidence diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeteoriteOverlayDiagnosticCode {
    /// Surface identifier is empty.
    EmptySurfaceId,
    /// Surface identifier appears more than once.
    DuplicateSurfaceId,
    /// Owner is empty.
    EmptyOwner,
    /// Evidence source is missing.
    MissingEvidence,
    /// Next action is missing.
    MissingNextAction,
    /// Shared chrome row omitted its chrome kind.
    MissingSharedChromeKind,
    /// Shared chrome row used a non-overlay chrome kind.
    UnsupportedSharedChromeKind,
    /// Non-shared row claimed a shared chrome kind.
    UnexpectedSharedChromeKind,
    /// Non-shared row omitted its ownership reason.
    MissingReason,
}

/// Meteorite overlay evidence diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeteoriteOverlayDiagnostic {
    /// Diagnostic code.
    pub code: MeteoriteOverlayDiagnosticCode,
    /// Surface associated with the diagnostic.
    pub surface_id: MeteoriteOverlaySurfaceId,
    /// Field associated with the diagnostic.
    pub field: MeteoriteOverlayField,
    /// Human-readable diagnostic message.
    pub message: String,
}

/// Validated Meteorite overlay semantics report.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MeteoriteOverlaySemanticsReport {
    /// Shared chrome frame surface identifiers.
    pub shared_surface_ids: Vec<MeteoriteOverlaySurfaceId>,
    /// Browser-native primitive surface identifiers.
    pub web_native_surface_ids: Vec<MeteoriteOverlaySurfaceId>,
    /// Renderer-owned adapter surface identifiers.
    pub renderer_owned_surface_ids: Vec<MeteoriteOverlaySurfaceId>,
    /// Validation diagnostics.
    pub diagnostics: Vec<MeteoriteOverlayDiagnostic>,
}

impl MeteoriteOverlaySemanticsReport {
    /// Returns true when no diagnostics were emitted.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

/// Validates Meteorite overlay semantics evidence.
#[must_use]
pub fn validate_meteorite_overlay_semantics(
    surfaces: &[MeteoriteOverlaySurfaceEvidence],
) -> MeteoriteOverlaySemanticsReport {
    let duplicate_ids = duplicate_surface_ids(surfaces);
    let mut outcome = MeteoriteOverlaySemanticsReport::default();

    for surface in surfaces {
        classify_surface(surface, &mut outcome);
        validate_required_fields(surface, &duplicate_ids, &mut outcome);
        validate_role_contract(surface, &mut outcome);
    }

    sort_report_ids(&mut outcome);
    outcome
}

fn sort_report_ids(outcome: &mut MeteoriteOverlaySemanticsReport) {
    outcome.shared_surface_ids.sort();
    outcome.web_native_surface_ids.sort();
    outcome.renderer_owned_surface_ids.sort();
}

fn classify_surface(surface: &MeteoriteOverlaySurfaceEvidence, outcome: &mut MeteoriteOverlaySemanticsReport) {
    if surface.surface_id.as_str().trim().is_empty() {
        return;
    }
    classification_bucket(surface, outcome).push(surface.surface_id.clone());
}

fn classification_bucket<'a>(
    surface: &MeteoriteOverlaySurfaceEvidence,
    outcome: &'a mut MeteoriteOverlaySemanticsReport,
) -> &'a mut Vec<MeteoriteOverlaySurfaceId> {
    match surface.role {
        MeteoriteOverlayRole::SharedChromeFrame => &mut outcome.shared_surface_ids,
        MeteoriteOverlayRole::WebNativePrimitive => &mut outcome.web_native_surface_ids,
        MeteoriteOverlayRole::RendererOwnedAdapter => &mut outcome.renderer_owned_surface_ids,
    }
}

fn validate_required_fields(
    surface: &MeteoriteOverlaySurfaceEvidence,
    duplicate_ids: &SurfaceIdSet,
    outcome: &mut MeteoriteOverlaySemanticsReport,
) {
    validate_surface_id(surface, duplicate_ids, outcome);
    validate_text_field(surface, &surface.owner, spec::empty_owner(), outcome);
    validate_text_field(surface, &surface.evidence, spec::missing_evidence(), outcome);
    validate_text_field(surface, &surface.next_action, spec::missing_next_action(), outcome);
}

fn validate_surface_id(
    surface: &MeteoriteOverlaySurfaceEvidence,
    duplicate_ids: &SurfaceIdSet,
    outcome: &mut MeteoriteOverlaySemanticsReport,
) {
    if surface.surface_id.as_str().trim().is_empty() {
        push_diagnostic(outcome, surface, spec::empty_surface_id());
    } else if duplicate_ids.contains(&surface.surface_id) {
        push_diagnostic(outcome, surface, spec::duplicate_surface_id());
    }
}

fn validate_text_field(
    surface: &MeteoriteOverlaySurfaceEvidence,
    value: &str,
    spec: spec::DiagnosticPlan,
    outcome: &mut MeteoriteOverlaySemanticsReport,
) {
    if value.trim().is_empty() {
        push_diagnostic(outcome, surface, spec);
    }
}

fn validate_role_contract(surface: &MeteoriteOverlaySurfaceEvidence, outcome: &mut MeteoriteOverlaySemanticsReport) {
    match surface.role {
        MeteoriteOverlayRole::SharedChromeFrame => validate_shared_chrome_kind(surface, outcome),
        MeteoriteOverlayRole::WebNativePrimitive | MeteoriteOverlayRole::RendererOwnedAdapter => {
            validate_non_shared_surface(surface, outcome);
        }
    }
}

fn validate_shared_chrome_kind(
    surface: &MeteoriteOverlaySurfaceEvidence,
    outcome: &mut MeteoriteOverlaySemanticsReport,
) {
    let Some(kind) = surface.shared_chrome_kind else {
        push_diagnostic(outcome, surface, spec::missing_shared_kind());
        return;
    };
    if !is_shared_overlay_kind(kind) {
        push_diagnostic(outcome, surface, spec::unsupported_shared_kind());
    }
}

fn is_shared_overlay_kind(kind: OverlayChromeKind) -> bool {
    matches!(kind, OverlayChromeKind::OverlayFrame | OverlayChromeKind::DialogFrame)
}

fn validate_non_shared_surface(
    surface: &MeteoriteOverlaySurfaceEvidence,
    outcome: &mut MeteoriteOverlaySemanticsReport,
) {
    if surface.reason.trim().is_empty() {
        push_diagnostic(outcome, surface, spec::missing_reason());
    }
    if surface.shared_chrome_kind.is_some() {
        push_diagnostic(outcome, surface, spec::unexpected_shared_kind());
    }
}

fn duplicate_surface_ids(surfaces: &[MeteoriteOverlaySurfaceEvidence]) -> SurfaceIdSet {
    let mut seen = SurfaceIdSet::new();
    let mut duplicates = SurfaceIdSet::new();
    for surface in surfaces {
        record_seen_surface(&mut seen, &mut duplicates, surface);
    }
    duplicates
}

fn record_seen_surface(
    seen: &mut SurfaceIdSet,
    duplicates: &mut SurfaceIdSet,
    surface: &MeteoriteOverlaySurfaceEvidence,
) {
    if surface.surface_id.as_str().trim().is_empty() {
        return;
    }
    if !seen.insert(surface.surface_id.clone()) {
        duplicates.insert(surface.surface_id.clone());
    }
}

fn push_diagnostic(
    outcome: &mut MeteoriteOverlaySemanticsReport,
    surface: &MeteoriteOverlaySurfaceEvidence,
    spec: spec::DiagnosticPlan,
) {
    outcome.diagnostics.push(MeteoriteOverlayDiagnostic {
        code: spec.code,
        surface_id: surface.surface_id.clone(),
        field: spec.field,
        message: spec.message.to_owned(),
    });
}

#[cfg(test)]
#[path = "semantics/tests.rs"]
mod tests;
