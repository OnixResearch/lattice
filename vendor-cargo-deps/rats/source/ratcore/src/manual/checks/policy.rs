pub(super) fn registry_scope_issues(
    registry: &crate::targeting_core::RendererRegistry,
    scope: &crate::targeting_core::RendererRequirementScope,
) -> Vec<super::SnapshotIssue> {
    crate::targeting_core::validate_renderer_scope(registry, scope)
        .issues
        .into_iter()
        .map(|issue| {
            super::SnapshotIssue::new(super::SnapshotIssueCode::InvalidRendererPolicy, issue.subject, issue.message)
        })
        .collect()
}

pub(super) fn manifest_adapter_issues(
    entry: &super::super::SnapshotManifestEntry,
    registry: &crate::targeting_core::RendererRegistry,
    scope: &crate::targeting_core::RendererRequirementScope,
) -> Vec<super::SnapshotIssue> {
    let descriptor = registry.evidence_adapter(&entry.evidence_adapter);
    let mut issues = label_issues(entry);
    issues.extend(
        [
            unknown_adapter_issue(entry, descriptor),
            wrong_role_issue(entry, descriptor),
            out_of_scope_issue(entry, scope),
        ]
        .into_iter()
        .flatten(),
    );
    issues
}

fn label_issues(entry: &super::super::SnapshotManifestEntry) -> Vec<super::SnapshotIssue> {
    if entry.renderer == entry.evidence_adapter.as_str() {
        return Vec::new();
    }
    vec![
        super::SnapshotIssue::new(
            super::SnapshotIssueCode::MismatchedEvidenceAdapter,
            entry.renderer.clone(),
            "legacy renderer label does not match nominal evidence-adapter identity",
        ),
        super::SnapshotIssue::new(
            super::SnapshotIssueCode::UnsupportedRenderer,
            entry.renderer.clone(),
            "snapshot renderer is unsupported by its nominal adapter mapping",
        ),
    ]
}

fn unknown_adapter_issue(
    entry: &super::super::SnapshotManifestEntry,
    descriptor: Option<&crate::targeting_core::EvidenceAdapterDescriptor>,
) -> Option<super::SnapshotIssue> {
    descriptor.is_none().then(|| {
        super::SnapshotIssue::new(
            super::SnapshotIssueCode::UnknownEvidenceAdapter,
            entry.evidence_adapter.as_str(),
            "snapshot evidence adapter is not registered",
        )
    })
}

fn wrong_role_issue(
    entry: &super::super::SnapshotManifestEntry,
    descriptor: Option<&crate::targeting_core::EvidenceAdapterDescriptor>,
) -> Option<super::SnapshotIssue> {
    descriptor
        .is_some_and(|adapter| adapter.role != crate::targeting_core::EvidenceRole::VisualSnapshot)
        .then(|| {
            super::SnapshotIssue::new(
                super::SnapshotIssueCode::WrongEvidenceRole,
                entry.evidence_adapter.as_str(),
                "snapshot manifest requires a visual snapshot evidence adapter",
            )
        })
}

fn out_of_scope_issue(
    entry: &super::super::SnapshotManifestEntry,
    scope: &crate::targeting_core::RendererRequirementScope,
) -> Option<super::SnapshotIssue> {
    let is_in_scope = scope.members.iter().any(|member| {
        matches!(
            member,
            crate::targeting_core::RendererSubject::EvidenceAdapter(id)
                if id == &entry.evidence_adapter
        )
    });
    (!is_in_scope).then(|| {
        super::SnapshotIssue::new(
            super::SnapshotIssueCode::UnknownEvidenceAdapter,
            entry.evidence_adapter.as_str(),
            "snapshot evidence adapter is outside the visual evidence scope",
        )
    })
}

pub(super) fn visual_adapter_for_family<'a>(
    registry: &'a crate::targeting_core::RendererRegistry,
    family: &crate::targeting_core::RendererFamilyId,
) -> Option<&'a crate::targeting_core::EvidenceAdapterDescriptor> {
    registry.evidence_adapter_for_family(family, crate::targeting_core::EvidenceRole::VisualSnapshot)
}

#[cfg(test)]
#[path = "policy/tests.rs"]
mod tests;
