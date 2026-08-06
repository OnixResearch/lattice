#[path = "requirements/content.rs"]
mod content;

/// Validates an ordered renderer requirement scope against a registry.
#[must_use]
pub fn validate_renderer_scope(
    registry: &super::super::RendererRegistry,
    scope: &super::super::RendererRequirementScope,
) -> super::RendererPolicyReport {
    let mut issues = super::validate_renderer_registry(registry).issues;
    issues.extend(scope_header_issue(scope));
    let mut seen = std::collections::BTreeSet::new();
    for member in &scope.members {
        issues.extend(member_record_issues(registry, scope, member, &mut seen));
    }
    super::finish(issues)
}

fn scope_header_issue(scope: &super::super::RendererRequirementScope) -> Option<super::RendererPolicyIssue> {
    (scope.id.as_str().trim().is_empty() || scope.members.is_empty()).then(|| {
        super::RendererPolicyIssue::new(
            super::RendererPolicyIssueCode::EmptyScope,
            "scope",
            scope.id.as_str(),
            "renderer scope identity and members are required",
        )
    })
}

fn member_record_issues(
    registry: &super::super::RendererRegistry,
    scope: &super::super::RendererRequirementScope,
    member: &super::super::RendererSubject,
    seen: &mut std::collections::BTreeSet<super::super::RendererSubject>,
) -> Vec<super::RendererPolicyIssue> {
    let mut issues = member_issues(registry, scope, member);
    if !seen.insert(member.clone()) {
        issues.push(super::RendererPolicyIssue::new(
            super::RendererPolicyIssueCode::DuplicateScopeMember,
            "scope",
            member.as_str(),
            "renderer scope member is duplicated",
        ));
    }
    issues
}

fn member_issues(
    registry: &super::super::RendererRegistry,
    scope: &super::super::RendererRequirementScope,
    member: &super::super::RendererSubject,
) -> Vec<super::RendererPolicyIssue> {
    if member.kind() != scope.subject_kind {
        return vec![super::RendererPolicyIssue::new(
            super::RendererPolicyIssueCode::WrongScopeMemberKind,
            "scope",
            member.as_str(),
            "renderer scope member has the wrong nominal identity kind",
        )];
    }
    let lifecycle = subject_lifecycle(registry, member);
    [
        unknown_member_issue(member, lifecycle),
        retired_member_issue(member, lifecycle),
        experimental_member_issue(scope, member, lifecycle),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn unknown_member_issue(
    member: &super::super::RendererSubject,
    lifecycle: Option<super::super::RendererLifecycle>,
) -> Option<super::RendererPolicyIssue> {
    lifecycle.is_none().then(|| {
        member_issue(
            super::RendererPolicyIssueCode::UnknownScopeMember,
            member,
            "renderer scope member is not registered",
        )
    })
}

fn retired_member_issue(
    member: &super::super::RendererSubject,
    lifecycle: Option<super::super::RendererLifecycle>,
) -> Option<super::RendererPolicyIssue> {
    (lifecycle == Some(super::super::RendererLifecycle::Retired)).then(|| {
        member_issue(
            super::RendererPolicyIssueCode::RetiredScopeMember,
            member,
            "retired renderer cannot be a required scope member",
        )
    })
}

fn experimental_member_issue(
    scope: &super::super::RendererRequirementScope,
    member: &super::super::RendererSubject,
    lifecycle: Option<super::super::RendererLifecycle>,
) -> Option<super::RendererPolicyIssue> {
    is_experimental_rejected(scope, lifecycle).then(|| {
        member_issue(
            super::RendererPolicyIssueCode::ExperimentalInMaintainedScope,
            member,
            "experimental renderer cannot broaden maintained completion",
        )
    })
}

fn is_experimental_rejected(
    scope: &super::super::RendererRequirementScope,
    lifecycle: Option<super::super::RendererLifecycle>,
) -> bool {
    scope.completion == super::super::RendererScopeCompletion::Maintained
        && lifecycle == Some(super::super::RendererLifecycle::Experimental)
}

fn member_issue(
    code: super::RendererPolicyIssueCode,
    member: &super::super::RendererSubject,
    message: &'static str,
) -> super::RendererPolicyIssue {
    super::RendererPolicyIssue::new(code, "scope", member.as_str(), message)
}

fn subject_lifecycle(
    registry: &super::super::RendererRegistry,
    subject: &super::super::RendererSubject,
) -> Option<super::super::RendererLifecycle> {
    match subject {
        super::super::RendererSubject::Family(id) => registry.family(id).map(|entry| entry.lifecycle),
        super::super::RendererSubject::Target(id) => registry.target(id).map(|entry| entry.lifecycle),
        super::super::RendererSubject::EvidenceAdapter(id) => {
            registry.evidence_adapter(id).map(|entry| entry.lifecycle)
        }
    }
}

/// Validates typed renderer cells against a registry and ordered scope.
#[must_use]
pub fn validate_scoped_renderer_cells(
    registry: &super::super::RendererRegistry,
    scope: &super::super::RendererRequirementScope,
    cells: &[super::super::ScopedRendererCell],
) -> super::RendererPolicyReport {
    let mut issues = validate_renderer_scope(registry, scope).issues;
    let required = scope.members.iter().cloned().collect::<std::collections::BTreeSet<_>>();
    let mut seen = std::collections::BTreeSet::new();
    for cell in cells {
        issues.extend(cell_record_issues(cell, &required, &mut seen));
    }
    issues.extend(missing_cell_issues(scope, &seen));
    issues.extend(cell_order_issue(scope, cells));
    super::finish(issues)
}

fn cell_record_issues(
    cell: &super::super::ScopedRendererCell,
    required: &std::collections::BTreeSet<super::super::RendererSubject>,
    seen: &mut std::collections::BTreeSet<super::super::RendererSubject>,
) -> Vec<super::RendererPolicyIssue> {
    let mut issues = cell_content_issues(cell);
    issues.extend(
        [
            duplicate_cell_issue(cell, seen),
            out_of_scope_cell_issue(cell, required),
        ]
        .into_iter()
        .flatten(),
    );
    issues
}

fn duplicate_cell_issue(
    cell: &super::super::ScopedRendererCell,
    seen: &mut std::collections::BTreeSet<super::super::RendererSubject>,
) -> Option<super::RendererPolicyIssue> {
    (!seen.insert(cell.subject.clone())).then(|| {
        content::cell_issue(super::RendererPolicyIssueCode::DuplicateCell, cell, "renderer cell is duplicated")
    })
}

fn out_of_scope_cell_issue(
    cell: &super::super::ScopedRendererCell,
    required: &std::collections::BTreeSet<super::super::RendererSubject>,
) -> Option<super::RendererPolicyIssue> {
    (!required.contains(&cell.subject)).then(|| {
        content::cell_issue(
            super::RendererPolicyIssueCode::OutOfScopeCell,
            cell,
            "renderer cell is outside the named scope",
        )
    })
}

fn missing_cell_issues(
    scope: &super::super::RendererRequirementScope,
    seen: &std::collections::BTreeSet<super::super::RendererSubject>,
) -> Vec<super::RendererPolicyIssue> {
    scope
        .members
        .iter()
        .filter(|member| !seen.contains(member))
        .map(|member| {
            super::RendererPolicyIssue::new(
                super::RendererPolicyIssueCode::MissingRequiredCell,
                "cell",
                member.as_str(),
                "required renderer cell is missing",
            )
        })
        .collect()
}

fn cell_order_issue(
    scope: &super::super::RendererRequirementScope,
    cells: &[super::super::ScopedRendererCell],
) -> Option<super::RendererPolicyIssue> {
    let actual = cells.iter().map(|cell| &cell.subject).collect::<Vec<_>>();
    let expected = scope.members.iter().collect::<Vec<_>>();
    (actual != expected).then(|| {
        super::RendererPolicyIssue::new(
            super::RendererPolicyIssueCode::CellOrderMismatch,
            "cell",
            scope.id.as_str(),
            "renderer cells do not follow the ordered requirement scope",
        )
    })
}

fn cell_content_issues(cell: &super::super::ScopedRendererCell) -> Vec<super::RendererPolicyIssue> {
    [
        content::missing_evidence_issue(cell),
        content::missing_reason_issue(cell),
        content::missing_next_action_issue(cell),
    ]
    .into_iter()
    .flatten()
    .collect()
}
