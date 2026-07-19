/// Private diagnostic template used by the overlay evidence validator.
#[derive(Debug, Clone, Copy)]
pub(super) struct DiagnosticPlan {
    /// Diagnostic code.
    pub(super) code: super::MeteoriteOverlayDiagnosticCode,
    /// Diagnostic field.
    pub(super) field: super::MeteoriteOverlayField,
    /// Diagnostic message.
    pub(super) message: &'static str,
}

/// Builds an empty-surface-id diagnostic specification.
#[must_use]
pub(super) fn empty_surface_id() -> DiagnosticPlan {
    DiagnosticPlan {
        code: super::MeteoriteOverlayDiagnosticCode::EmptySurfaceId,
        field: super::MeteoriteOverlayField::SurfaceId,
        message: "overlay surface id is empty",
    }
}

/// Builds a duplicate-surface-id diagnostic specification.
#[must_use]
pub(super) fn duplicate_surface_id() -> DiagnosticPlan {
    DiagnosticPlan {
        code: super::MeteoriteOverlayDiagnosticCode::DuplicateSurfaceId,
        field: super::MeteoriteOverlayField::SurfaceId,
        message: "overlay surface id is duplicated",
    }
}

/// Builds an empty-owner diagnostic specification.
#[must_use]
pub(super) fn empty_owner() -> DiagnosticPlan {
    DiagnosticPlan {
        code: super::MeteoriteOverlayDiagnosticCode::EmptyOwner,
        field: super::MeteoriteOverlayField::Owner,
        message: "overlay owner is empty",
    }
}

/// Builds a missing-evidence diagnostic specification.
#[must_use]
pub(super) fn missing_evidence() -> DiagnosticPlan {
    DiagnosticPlan {
        code: super::MeteoriteOverlayDiagnosticCode::MissingEvidence,
        field: super::MeteoriteOverlayField::Evidence,
        message: "overlay evidence source is missing",
    }
}

/// Builds a missing-next-action diagnostic specification.
#[must_use]
pub(super) fn missing_next_action() -> DiagnosticPlan {
    DiagnosticPlan {
        code: super::MeteoriteOverlayDiagnosticCode::MissingNextAction,
        field: super::MeteoriteOverlayField::NextAction,
        message: "overlay next action is missing",
    }
}

/// Builds a missing-shared-kind diagnostic specification.
#[must_use]
pub(super) fn missing_shared_kind() -> DiagnosticPlan {
    DiagnosticPlan {
        code: super::MeteoriteOverlayDiagnosticCode::MissingSharedChromeKind,
        field: super::MeteoriteOverlayField::SharedChromeKind,
        message: "shared overlay row is missing a chrome kind",
    }
}

/// Builds an unsupported-shared-kind diagnostic specification.
#[must_use]
pub(super) fn unsupported_shared_kind() -> DiagnosticPlan {
    DiagnosticPlan {
        code: super::MeteoriteOverlayDiagnosticCode::UnsupportedSharedChromeKind,
        field: super::MeteoriteOverlayField::SharedChromeKind,
        message: "shared overlay row must use OverlayFrame or DialogFrame",
    }
}

/// Builds a missing-reason diagnostic specification.
#[must_use]
pub(super) fn missing_reason() -> DiagnosticPlan {
    DiagnosticPlan {
        code: super::MeteoriteOverlayDiagnosticCode::MissingReason,
        field: super::MeteoriteOverlayField::Reason,
        message: "non-shared overlay row is missing an ownership reason",
    }
}

/// Builds an unexpected-shared-kind diagnostic specification.
#[must_use]
pub(super) fn unexpected_shared_kind() -> DiagnosticPlan {
    DiagnosticPlan {
        code: super::MeteoriteOverlayDiagnosticCode::UnexpectedSharedChromeKind,
        field: super::MeteoriteOverlayField::SharedChromeKind,
        message: "non-shared overlay row must not claim a shared chrome kind",
    }
}
