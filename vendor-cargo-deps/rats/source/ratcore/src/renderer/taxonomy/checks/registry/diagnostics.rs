pub(super) fn duplicate_family(identity: super::IssueIdentity<'_>) -> super::super::RendererPolicyIssue {
    super::super::RendererPolicyIssue::new(
        super::super::RendererPolicyIssueCode::DuplicateFamily,
        identity.domain,
        identity.subject,
        "renderer family identity is duplicated",
    )
}

pub(super) fn duplicate_adapter(identity: super::IssueIdentity<'_>) -> super::super::RendererPolicyIssue {
    super::super::RendererPolicyIssue::new(
        super::super::RendererPolicyIssueCode::DuplicateEvidenceAdapter,
        identity.domain,
        identity.subject,
        "evidence adapter identity is duplicated",
    )
}

pub(super) fn duplicate_mapping(identity: super::IssueIdentity<'_>) -> super::super::RendererPolicyIssue {
    super::super::RendererPolicyIssue::new(
        super::super::RendererPolicyIssueCode::DuplicateEvidenceMapping,
        identity.domain,
        identity.subject,
        "target and evidence role map to more than one adapter",
    )
}

pub(super) fn lifecycle_issue(identity: super::IssueIdentity<'_>) -> super::super::RendererPolicyIssue {
    super::super::RendererPolicyIssue::new(
        super::super::RendererPolicyIssueCode::LifecycleMismatch,
        identity.domain,
        identity.subject,
        "maintained renderer entry requires a maintained parent",
    )
}

pub(super) fn unknown_replacement(identity: super::IssueIdentity<'_>) -> super::super::RendererPolicyIssue {
    super::super::RendererPolicyIssue::new(
        super::super::RendererPolicyIssueCode::UnknownReplacement,
        identity.domain,
        identity.subject,
        "renderer replacement identity is not registered",
    )
}
