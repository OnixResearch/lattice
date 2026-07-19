use super::*;

pub(super) fn row_issues(row: &FullMouseSupportRow) -> Vec<FullMouseSupportIssue> {
    let mut issues =
        Vec::with_capacity(row.renderers.len().saturating_add(REQUIRED_RENDERERS.len()));
    debug_assert!(issues.capacity() >= row.renderers.len());
    if row_required_fields_missing(row) {
        issues.push(FullMouseSupportIssue::new(
            FullMouseSupportIssueCode::MissingRequiredField,
            row.capability_id.clone(),
            "row",
            "capability, event class, surface family, and shared owner are required",
        ));
    }
    issues.extend(required_renderer_issues(row));
    issues.extend(renderer_status_issues(row));
    if row.diagnostics_coverage.trim().is_empty() {
        issues.push(FullMouseSupportIssue::new(
            FullMouseSupportIssueCode::MissingDiagnosticsCoverage,
            row.capability_id.clone(),
            "diagnostics_coverage",
            "full mouse rows require diagnostic coverage evidence",
        ));
    }
    if has_gap(row) && row.next_action.trim().is_empty() {
        issues.push(FullMouseSupportIssue::new(
            FullMouseSupportIssueCode::MissingNextAction,
            row.capability_id.clone(),
            "next_action",
            "full mouse support gaps require a next action",
        ));
    }
    issues
}

fn required_renderer_issues(row: &FullMouseSupportRow) -> Vec<FullMouseSupportIssue> {
    debug_assert!(!REQUIRED_RENDERERS.is_empty());
    debug_assert!(row.renderers.len() <= REQUIRED_RENDERERS.len() || !row.renderers.is_empty());
    let labels = row
        .renderers
        .iter()
        .map(|renderer| renderer.renderer.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    REQUIRED_RENDERERS
        .iter()
        .copied()
        .filter(|required| !labels.contains(required))
        .map(|required| {
            FullMouseSupportIssue::new(
                FullMouseSupportIssueCode::MissingRenderer,
                row.capability_id.clone(),
                required,
                "full mouse row is missing required renderer status",
            )
        })
        .collect()
}

fn renderer_status_issues(row: &FullMouseSupportRow) -> Vec<FullMouseSupportIssue> {
    row.renderers
        .iter()
        .flat_map(|renderer| renderer_issues(row, renderer))
        .collect()
}

fn renderer_issues(
    row: &FullMouseSupportRow,
    renderer: &FullMouseRendererStatus,
) -> Vec<FullMouseSupportIssue> {
    let mut issues = Vec::with_capacity(KNOWN_STATUSES.len());
    let status = renderer.status.as_str();
    debug_assert!(issues.capacity() >= KNOWN_STATUSES.len());
    debug_assert!(!renderer.renderer.is_empty() || !status.is_empty());
    if !KNOWN_STATUSES.contains(&status) {
        issues.push(FullMouseSupportIssue::new(
            FullMouseSupportIssueCode::UnknownStatus,
            row.capability_id.clone(),
            renderer.renderer.clone(),
            format!("unknown full mouse support status `{status}`"),
        ));
        return issues;
    }
    if missing_evidence(status, renderer) {
        issues.push(FullMouseSupportIssue::new(
            FullMouseSupportIssueCode::MissingEvidence,
            row.capability_id.clone(),
            renderer.renderer.clone(),
            "complete or partial mouse support requires evidence",
        ));
    }
    if missing_reason(status, renderer) {
        issues.push(FullMouseSupportIssue::new(
            FullMouseSupportIssueCode::MissingReason,
            row.capability_id.clone(),
            renderer.renderer.clone(),
            "unsupported, blocked, or divergent mouse support requires a reason",
        ));
    }
    issues
}

fn row_required_fields_missing(row: &FullMouseSupportRow) -> bool {
    let is_missing_capability = row.capability_id.trim().is_empty();
    let is_missing_event_class = row.event_class.trim().is_empty();
    let is_missing_surface = row.surface_family.trim().is_empty();
    let is_missing_owner = row.shared_core_owner.trim().is_empty();
    is_missing_capability || is_missing_event_class || is_missing_surface || is_missing_owner
}

fn missing_evidence(status: &str, renderer: &FullMouseRendererStatus) -> bool {
    let is_status_needs_evidence = status == STATUS_COMPLETE || status == STATUS_PARTIAL;
    let is_evidence_missing = renderer
        .evidence
        .iter()
        .all(|entry| entry.trim().is_empty());
    is_status_needs_evidence && is_evidence_missing
}

fn missing_reason(status: &str, renderer: &FullMouseRendererStatus) -> bool {
    let is_status_needs_reason =
        [STATUS_UNSUPPORTED, STATUS_BLOCKED, STATUS_DIVERGENT].contains(&status);
    let is_reason_missing = renderer
        .reason
        .as_deref()
        .unwrap_or_default()
        .trim()
        .is_empty();
    is_status_needs_reason && is_reason_missing
}

fn has_gap(row: &FullMouseSupportRow) -> bool {
    row.renderers
        .iter()
        .any(|renderer| GAP_STATUSES.contains(&renderer.status.as_str()))
}
