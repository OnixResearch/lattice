pub(super) fn scope_issues(
    scope: &crate::targeting_core::RendererRequirementScope,
    report: &crate::targeting_core::RendererPolicyReport,
) -> Vec<super::Issue> {
    report
        .issues
        .iter()
        .map(|issue| {
            super::Issue::new(
                super::IssueCode::InvalidRendererScope,
                scope.id.as_str(),
                issue.subject.clone(),
                issue.message.clone(),
            )
        })
        .collect()
}

pub(super) fn row_cell_issues(
    row: &super::Row,
    registry: &crate::targeting_core::RendererRegistry,
    scope: &crate::targeting_core::RendererRequirementScope,
) -> Vec<super::Issue> {
    let cells = row.renderer_cells.iter().map(|cell| policy_cell(cell, &row.next_action)).collect::<Vec<_>>();
    crate::targeting_core::validate_scoped_renderer_cells(registry, scope, &cells)
        .issues
        .into_iter()
        .map(|issue| map_issue(row, registry, issue))
        .collect()
}

pub(super) fn specialized_target_issues(
    feature_id: &str,
    cells: &[super::Support],
    registry: &crate::targeting_core::RendererRegistry,
    scope: &crate::targeting_core::RendererRequirementScope,
) -> Vec<super::Issue> {
    let required = scope
        .members
        .iter()
        .map(crate::targeting_core::RendererSubject::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    let mut seen = std::collections::BTreeSet::new();
    let mut issues = Vec::with_capacity(cells.len().saturating_add(required.len()));
    for cell in cells {
        issues.extend(specialized_cell_issues(feature_id, cell, registry, &required, &mut seen));
    }
    issues.extend(missing_target_issues(feature_id, scope, &seen));
    issues
}

fn specialized_cell_issues(
    feature_id: &str,
    cell: &super::Support,
    registry: &crate::targeting_core::RendererRegistry,
    required: &std::collections::BTreeSet<&str>,
    seen: &mut std::collections::BTreeSet<crate::targeting_core::RendererTargetId>,
) -> Option<super::Issue> {
    target_issue_code(cell, registry, required, seen).map(|code| {
        super::Issue::new(
            code,
            feature_id,
            cell.renderer.clone(),
            "specialized renderer target is duplicated, unknown, or outside its scope",
        )
    })
}

fn target_issue_code(
    cell: &super::Support,
    registry: &crate::targeting_core::RendererRegistry,
    required: &std::collections::BTreeSet<&str>,
    seen: &mut std::collections::BTreeSet<crate::targeting_core::RendererTargetId>,
) -> Option<super::IssueCode> {
    let target = crate::targeting_core::RendererTargetId::new(&cell.renderer);
    if !seen.insert(target.clone()) {
        Some(super::IssueCode::DuplicateRenderer)
    } else if registry.target(&target).is_none() {
        Some(super::IssueCode::UnknownRenderer)
    } else if !required.contains(cell.renderer.as_str()) {
        Some(super::IssueCode::OutOfScopeRenderer)
    } else {
        None
    }
}

fn missing_target_issues(
    feature_id: &str,
    scope: &crate::targeting_core::RendererRequirementScope,
    seen: &std::collections::BTreeSet<crate::targeting_core::RendererTargetId>,
) -> Vec<super::Issue> {
    scope
        .members
        .iter()
        .filter(|member| !seen.iter().any(|target| target.as_str() == member.as_str()))
        .map(|member| {
            super::Issue::new(
                super::IssueCode::MissingRendererLabel,
                feature_id,
                member.as_str(),
                "specialized row is missing required renderer target metadata",
            )
        })
        .collect()
}

fn policy_cell(cell: &super::FamilySupport, next_action: &str) -> crate::targeting_core::ScopedRendererCell {
    let subject = crate::targeting_core::RendererSubject::Family(cell.family.clone());
    let mut scoped = crate::targeting_core::ScopedRendererCell::new(subject, policy_status(&cell.status));
    scoped.evidence.clone_from(&cell.evidence);
    scoped.reason.clone_from(&cell.reason);
    if !next_action.trim().is_empty() {
        scoped.next_action = Some(next_action.to_string());
    }
    scoped
}

fn policy_status(status: &str) -> crate::targeting_core::RendererCellStatus {
    match status {
        super::STATUS_COMPLETE => crate::targeting_core::RendererCellStatus::Complete,
        super::STATUS_PARTIAL => crate::targeting_core::RendererCellStatus::Partial,
        super::STATUS_UNSUPPORTED | super::STATUS_DIVERGENT => crate::targeting_core::RendererCellStatus::Unsupported,
        super::STATUS_MISSING => crate::targeting_core::RendererCellStatus::Missing,
        _ => crate::targeting_core::RendererCellStatus::Missing,
    }
}

fn map_issue(
    row: &super::Row,
    registry: &crate::targeting_core::RendererRegistry,
    issue: crate::targeting_core::RendererPolicyIssue,
) -> super::Issue {
    let code = if issue.code == crate::targeting_core::RendererPolicyIssueCode::OutOfScopeCell {
        let family = crate::targeting_core::RendererFamilyId::new(&issue.subject);
        if registry.family(&family).is_some() {
            super::IssueCode::OutOfScopeRenderer
        } else {
            super::IssueCode::UnknownRenderer
        }
    } else {
        mapped_issue_code(issue.code)
    };
    super::Issue::new(code, row.feature_id.clone(), issue.subject, issue.message)
}

const fn mapped_issue_code(code: crate::targeting_core::RendererPolicyIssueCode) -> super::IssueCode {
    match code {
        crate::targeting_core::RendererPolicyIssueCode::MissingRequiredCell => super::IssueCode::MissingRendererLabel,
        crate::targeting_core::RendererPolicyIssueCode::DuplicateCell => super::IssueCode::DuplicateRenderer,
        crate::targeting_core::RendererPolicyIssueCode::MissingUnsupportedReason => super::IssueCode::MissingReason,
        crate::targeting_core::RendererPolicyIssueCode::MissingEvidence => super::IssueCode::MissingEvidence,
        crate::targeting_core::RendererPolicyIssueCode::MissingNextAction => super::IssueCode::MissingNextAction,
        crate::targeting_core::RendererPolicyIssueCode::ExperimentalInMaintainedScope
        | crate::targeting_core::RendererPolicyIssueCode::CellOrderMismatch => {
            super::IssueCode::InvalidStatusRelationship
        }
        crate::targeting_core::RendererPolicyIssueCode::OutOfScopeCell
        | crate::targeting_core::RendererPolicyIssueCode::EmptyIdentity
        | crate::targeting_core::RendererPolicyIssueCode::DuplicateFamily
        | crate::targeting_core::RendererPolicyIssueCode::DuplicateTarget
        | crate::targeting_core::RendererPolicyIssueCode::DuplicateEvidenceAdapter
        | crate::targeting_core::RendererPolicyIssueCode::UnknownFamily
        | crate::targeting_core::RendererPolicyIssueCode::UnknownTarget
        | crate::targeting_core::RendererPolicyIssueCode::UnknownReplacement
        | crate::targeting_core::RendererPolicyIssueCode::LifecycleMismatch
        | crate::targeting_core::RendererPolicyIssueCode::RetiredWithoutReason
        | crate::targeting_core::RendererPolicyIssueCode::DuplicateEvidenceMapping
        | crate::targeting_core::RendererPolicyIssueCode::EmptyScope
        | crate::targeting_core::RendererPolicyIssueCode::DuplicateScopeMember
        | crate::targeting_core::RendererPolicyIssueCode::WrongScopeMemberKind
        | crate::targeting_core::RendererPolicyIssueCode::UnknownScopeMember
        | crate::targeting_core::RendererPolicyIssueCode::RetiredScopeMember => super::IssueCode::InvalidRendererScope,
    }
}

#[cfg(test)]
#[path = "cells/tests.rs"]
mod tests;
