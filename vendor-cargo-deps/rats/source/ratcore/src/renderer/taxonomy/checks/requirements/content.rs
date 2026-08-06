pub(super) fn missing_evidence_issue(
    cell: &super::super::super::ScopedRendererCell,
) -> Option<super::super::RendererPolicyIssue> {
    (cell.status == super::super::super::RendererCellStatus::Complete
        && cell.evidence.iter().all(|entry| entry.trim().is_empty()))
    .then(|| {
        cell_issue(
            super::super::RendererPolicyIssueCode::MissingEvidence,
            cell,
            "complete renderer support requires evidence",
        )
    })
}

pub(super) fn missing_reason_issue(
    cell: &super::super::super::ScopedRendererCell,
) -> Option<super::super::RendererPolicyIssue> {
    (cell.status == super::super::super::RendererCellStatus::Unsupported
        && cell.reason.as_deref().is_none_or(|reason| reason.trim().is_empty()))
    .then(|| {
        cell_issue(
            super::super::RendererPolicyIssueCode::MissingUnsupportedReason,
            cell,
            "unsupported renderer support requires a reason",
        )
    })
}

pub(super) fn missing_next_action_issue(
    cell: &super::super::super::ScopedRendererCell,
) -> Option<super::super::RendererPolicyIssue> {
    let needs_action = matches!(
        cell.status,
        super::super::super::RendererCellStatus::Partial | super::super::super::RendererCellStatus::Missing
    );
    (needs_action && cell.next_action.as_deref().is_none_or(|action| action.trim().is_empty())).then(|| {
        cell_issue(
            super::super::RendererPolicyIssueCode::MissingNextAction,
            cell,
            "partial or missing renderer support requires a next action",
        )
    })
}

pub(super) fn cell_issue(
    code: super::super::RendererPolicyIssueCode,
    cell: &super::super::super::ScopedRendererCell,
    message: &'static str,
) -> super::super::RendererPolicyIssue {
    super::super::RendererPolicyIssue::new(code, "cell", cell.subject.as_str(), message)
}
