#[path = "registry/diagnostics.rs"]
mod diagnostics;

/// Validates renderer registry identities, relationships, and lifecycles.
#[must_use]
pub fn validate_renderer_registry(registry: &super::super::RendererRegistry) -> super::RendererPolicyReport {
    let mut issues = family_issues(registry);
    issues.extend(target_issues(registry));
    issues.extend(adapter_issues(registry));
    super::finish(issues)
}

fn family_issues(registry: &super::super::RendererRegistry) -> Vec<super::RendererPolicyIssue> {
    let mut seen = std::collections::BTreeSet::new();
    registry
        .families
        .iter()
        .flat_map(|family| family_record_issues(registry, family, &mut seen))
        .collect()
}

fn family_record_issues(
    registry: &super::super::RendererRegistry,
    family: &super::super::RendererFamilyDescriptor,
    seen: &mut std::collections::BTreeSet<super::super::RendererFamilyId>,
) -> Vec<super::RendererPolicyIssue> {
    let identity = IssueIdentity::new("family", family.id.as_str());
    let mut issues = family_descriptor_issues(family, identity);
    issues.extend(family_relation_issues(registry, family, identity, seen));
    issues
}

fn family_relation_issues(
    registry: &super::super::RendererRegistry,
    family: &super::super::RendererFamilyDescriptor,
    identity: IssueIdentity<'_>,
    seen: &mut std::collections::BTreeSet<super::super::RendererFamilyId>,
) -> Vec<super::RendererPolicyIssue> {
    [
        (!seen.insert(family.id.clone())).then(|| diagnostics::duplicate_family(identity)),
        family
            .replacement
            .as_ref()
            .is_some_and(|id| registry.family(id).is_none())
            .then(|| diagnostics::unknown_replacement(identity)),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn family_descriptor_issues(
    family: &super::super::RendererFamilyDescriptor,
    identity: IssueIdentity<'_>,
) -> Vec<super::RendererPolicyIssue> {
    descriptor_issues(DescriptorInput {
        identity,
        label: &family.label,
        lifecycle: family.lifecycle,
        reason: family.reason.as_deref(),
    })
}

fn target_issues(registry: &super::super::RendererRegistry) -> Vec<super::RendererPolicyIssue> {
    let mut seen = std::collections::BTreeSet::new();
    registry
        .targets
        .iter()
        .flat_map(|target| target_record_issues(registry, target, &mut seen))
        .collect()
}

fn target_record_issues(
    registry: &super::super::RendererRegistry,
    target: &super::super::RendererTargetDescriptor,
    seen: &mut std::collections::BTreeSet<super::super::RendererTargetId>,
) -> Vec<super::RendererPolicyIssue> {
    let identity = IssueIdentity::new("target", target.id.as_str());
    let mut issues = target_descriptor_issues(registry, target, identity);
    if !seen.insert(target.id.clone()) {
        issues.push(super::RendererPolicyIssue::new(
            super::RendererPolicyIssueCode::DuplicateTarget,
            identity.domain,
            identity.subject,
            "renderer target identity is duplicated",
        ));
    }
    issues
}

fn target_descriptor_issues(
    registry: &super::super::RendererRegistry,
    target: &super::super::RendererTargetDescriptor,
    identity: IssueIdentity<'_>,
) -> Vec<super::RendererPolicyIssue> {
    let mut issues = descriptor_issues(DescriptorInput {
        identity,
        label: &target.label,
        lifecycle: target.lifecycle,
        reason: target.reason.as_deref(),
    });
    issues.extend(target_relation_issues(registry, target, identity));
    issues
}

fn target_relation_issues(
    registry: &super::super::RendererRegistry,
    target: &super::super::RendererTargetDescriptor,
    identity: IssueIdentity<'_>,
) -> Vec<super::RendererPolicyIssue> {
    let parent = registry.family(&target.family);
    [
        parent.is_none().then(|| unknown_family(identity)),
        lifecycle_mismatch(target.lifecycle, parent.map(|family| family.lifecycle))
            .then(|| diagnostics::lifecycle_issue(identity)),
        target
            .replacement
            .as_ref()
            .is_some_and(|id| registry.target(id).is_none())
            .then(|| diagnostics::unknown_replacement(identity)),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn unknown_family(identity: IssueIdentity<'_>) -> super::RendererPolicyIssue {
    super::RendererPolicyIssue::new(
        super::RendererPolicyIssueCode::UnknownFamily,
        identity.domain,
        identity.subject,
        "renderer target references an unknown family",
    )
}

fn adapter_issues(registry: &super::super::RendererRegistry) -> Vec<super::RendererPolicyIssue> {
    let mut seen = std::collections::BTreeSet::new();
    let mut mappings = std::collections::BTreeSet::new();
    registry
        .evidence_adapters
        .iter()
        .flat_map(|adapter| adapter_record_issues(registry, adapter, &mut seen, &mut mappings))
        .collect()
}

fn adapter_record_issues(
    registry: &super::super::RendererRegistry,
    adapter: &super::super::EvidenceAdapterDescriptor,
    seen: &mut std::collections::BTreeSet<super::super::EvidenceAdapterId>,
    mappings: &mut std::collections::BTreeSet<(super::super::RendererTargetId, super::super::EvidenceRole)>,
) -> Vec<super::RendererPolicyIssue> {
    let identity = IssueIdentity::new("adapter", adapter.id.as_str());
    let mut issues = adapter_descriptor_issues(registry, adapter, identity);
    issues.extend(
        [
            (!seen.insert(adapter.id.clone())).then(|| diagnostics::duplicate_adapter(identity)),
            (!mappings.insert((adapter.target.clone(), adapter.role)))
                .then(|| diagnostics::duplicate_mapping(identity)),
        ]
        .into_iter()
        .flatten(),
    );
    issues
}

fn adapter_descriptor_issues(
    registry: &super::super::RendererRegistry,
    adapter: &super::super::EvidenceAdapterDescriptor,
    identity: IssueIdentity<'_>,
) -> Vec<super::RendererPolicyIssue> {
    let mut issues = descriptor_issues(DescriptorInput {
        identity,
        label: &adapter.label,
        lifecycle: adapter.lifecycle,
        reason: adapter.reason.as_deref(),
    });
    issues.extend(adapter_relation_issues(registry, adapter, identity));
    issues
}

fn adapter_relation_issues(
    registry: &super::super::RendererRegistry,
    adapter: &super::super::EvidenceAdapterDescriptor,
    identity: IssueIdentity<'_>,
) -> Vec<super::RendererPolicyIssue> {
    let target = registry.target(&adapter.target);
    [
        target.is_none().then(|| unknown_target(identity)),
        lifecycle_mismatch(adapter.lifecycle, target.map(|target| target.lifecycle))
            .then(|| diagnostics::lifecycle_issue(identity)),
        adapter
            .replacement
            .as_ref()
            .is_some_and(|id| registry.evidence_adapter(id).is_none())
            .then(|| diagnostics::unknown_replacement(identity)),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn unknown_target(identity: IssueIdentity<'_>) -> super::RendererPolicyIssue {
    super::RendererPolicyIssue::new(
        super::RendererPolicyIssueCode::UnknownTarget,
        identity.domain,
        identity.subject,
        "evidence adapter references an unknown target",
    )
}

#[derive(Clone, Copy)]
struct IssueIdentity<'a> {
    domain: &'static str,
    subject: &'a str,
}

impl<'a> IssueIdentity<'a> {
    const fn new(domain: &'static str, subject: &'a str) -> Self {
        Self { domain, subject }
    }
}

struct DescriptorInput<'a> {
    identity: IssueIdentity<'a>,
    label: &'a str,
    lifecycle: super::super::RendererLifecycle,
    reason: Option<&'a str>,
}

fn descriptor_issues(input: DescriptorInput<'_>) -> Vec<super::RendererPolicyIssue> {
    [empty_descriptor_issue(&input), retired_issue(&input)].into_iter().flatten().collect()
}

fn empty_descriptor_issue(input: &DescriptorInput<'_>) -> Option<super::RendererPolicyIssue> {
    (input.identity.subject.trim().is_empty() || input.label.trim().is_empty()).then(|| {
        super::RendererPolicyIssue::new(
            super::RendererPolicyIssueCode::EmptyIdentity,
            input.identity.domain,
            input.identity.subject,
            "renderer identity and label must not be empty",
        )
    })
}

fn retired_issue(input: &DescriptorInput<'_>) -> Option<super::RendererPolicyIssue> {
    let is_reason_missing = input.reason.is_none_or(|reason| reason.trim().is_empty());
    (input.lifecycle == super::super::RendererLifecycle::Retired && is_reason_missing).then(|| {
        super::RendererPolicyIssue::new(
            super::RendererPolicyIssueCode::RetiredWithoutReason,
            input.identity.domain,
            input.identity.subject,
            "retired renderer entries require a reason",
        )
    })
}

fn lifecycle_mismatch(child: super::super::RendererLifecycle, parent: Option<super::super::RendererLifecycle>) -> bool {
    child == super::super::RendererLifecycle::Maintained
        && parent.is_some_and(|parent| parent != super::super::RendererLifecycle::Maintained)
}
