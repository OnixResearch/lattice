//! Fail-closed evidence-grade aggregation.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// Complete matrix build with structural/receipt diagnostics.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PortabilityMatrixBuild {
    /// Ordered profile/target cells.
    pub cells: Vec<super::PortabilityMatrixCell>,
    /// Ordered receipt/profile diagnostics.
    pub issues: Vec<super::PortabilityMatrixIssue>,
}

impl PortabilityMatrixBuild {
    /// Whether every maintained required cell is supported and values are valid.
    #[must_use]
    pub fn all_required_supported(&self) -> bool {
        self.issues.is_empty()
            && self
                .cells
                .iter()
                .all(|cell| cell.required_grades.is_empty() || cell.status == super::PortabilityMatrixStatus::Supported)
    }
}

/// Builds every profile/exact-target cell and preserves non-passing states.
#[must_use]
pub fn build_portability_matrix(
    receipts: &[super::PortabilityReceipt],
    context: super::PortabilityReceiptContext<'_>,
) -> PortabilityMatrixBuild {
    let mut issues = super::validate_portability_profiles(context.profiles, context.requirements, context.targets);
    issues.extend(super::validate_portability_receipts(receipts, context));
    issues.sort();
    issues.dedup();
    let invalid_receipts = issues.iter().map(|issue| issue.subject.as_str()).collect::<BTreeSet<_>>();
    let mut requirement_map: BTreeMap<
        (super::PortabilityProfileId, super::super::RustTargetId),
        Vec<&super::PortabilityMatrixRequirement>,
    > = BTreeMap::new();
    for requirement in context.requirements {
        requirement_map
            .entry((requirement.profile.clone(), requirement.target.clone()))
            .or_default()
            .push(requirement);
    }

    let mut cells = Vec::new();
    for profile in context.profiles {
        for target in context.targets {
            let key = (profile.id.clone(), target.id.clone());
            let requirements = requirement_map.get(&key).cloned().unwrap_or_default();
            if requirements.is_empty() {
                let status = if profile.role == super::PortabilityProfileRole::Experimental
                    && target.id == super::super::TARGET_X86_64_LINUX_GNU
                {
                    super::PortabilityMatrixStatus::Experimental
                } else {
                    super::PortabilityMatrixStatus::Irrelevant
                };
                cells.push(super::PortabilityMatrixCell {
                    profile: profile.id.clone(),
                    target: target.id.clone(),
                    status,
                    required_grades: Vec::new(),
                    reason: status.as_str().to_string(),
                });
                continue;
            }
            let grade_statuses = requirements
                .iter()
                .map(|requirement| {
                    (requirement.grade, grade_status(requirement, profile, receipts, context, &invalid_receipts))
                })
                .collect::<Vec<_>>();
            let status = aggregate_grade_statuses(&grade_statuses);
            cells.push(super::PortabilityMatrixCell {
                profile: profile.id.clone(),
                target: target.id.clone(),
                status,
                required_grades: grade_statuses.iter().map(|(grade, _)| *grade).collect(),
                reason: grade_statuses
                    .iter()
                    .filter(|(_, grade_status)| *grade_status != super::PortabilityMatrixStatus::Supported)
                    .map(|(grade, grade_status)| format!("{}={}", grade.as_str(), grade_status.as_str()))
                    .collect::<Vec<_>>()
                    .join(",")
                    .then_nonempty()
                    .unwrap_or_else(|| "all-required-grades-passed".to_string()),
            });
        }
    }
    cells.sort_by(|left, right| left.profile.cmp(&right.profile).then_with(|| left.target.cmp(&right.target)));
    PortabilityMatrixBuild { cells, issues }
}

fn grade_status(
    requirement: &super::PortabilityMatrixRequirement,
    profile: &super::PortabilityPackageProfile,
    receipts: &[super::PortabilityReceipt],
    context: super::PortabilityReceiptContext<'_>,
    invalid_receipts: &BTreeSet<&str>,
) -> super::PortabilityMatrixStatus {
    let matching = receipts
        .iter()
        .filter(|receipt| {
            receipt.profile == requirement.profile
                && receipt.target == requirement.target
                && receipt.grade == requirement.grade
        })
        .collect::<Vec<_>>();
    if matching.is_empty() {
        return super::PortabilityMatrixStatus::Missing;
    }
    let valid = matching
        .iter()
        .copied()
        .filter(|receipt| !invalid_receipts.contains(receipt.id.as_str()))
        .collect::<Vec<_>>();
    if valid.iter().any(|receipt| {
        receipt.method == super::super::HostEvidenceMethod::Direct
            && receipt.state == super::super::HostEvidenceState::Passed
            && receipt.freshness == super::super::EvidenceFreshness::Fresh
            && receipt.source_hash == context.source_hash
            && receipt.package_set_hash == profile.package_set_hash
            && receipt.command == requirement.command
            && receipt.command_hash == requirement.command_hash
    }) {
        return super::PortabilityMatrixStatus::Supported;
    }
    if matching.iter().any(|receipt| {
        receipt.freshness == super::super::EvidenceFreshness::Stale
            || receipt.source_hash != context.source_hash
            || receipt.package_set_hash != profile.package_set_hash
            || receipt.command_hash != requirement.command_hash
    }) {
        return super::PortabilityMatrixStatus::Stale;
    }
    if valid.iter().any(|receipt| {
        receipt.method == super::super::HostEvidenceMethod::Direct
            && receipt.state == super::super::HostEvidenceState::Failed
    }) {
        return super::PortabilityMatrixStatus::Failed;
    }
    if valid.iter().any(|receipt| {
        receipt.method == super::super::HostEvidenceMethod::Direct
            && receipt.state == super::super::HostEvidenceState::Unsupported
    }) {
        return super::PortabilityMatrixStatus::Unsupported;
    }
    if valid.iter().any(|receipt| {
        receipt.method == super::super::HostEvidenceMethod::Direct
            && matches!(
                receipt.state,
                super::super::HostEvidenceState::Blocked | super::super::HostEvidenceState::Skipped
            )
    }) {
        return super::PortabilityMatrixStatus::Blocked;
    }
    if requirement.allows_cross_compile
        && valid.iter().any(|receipt| {
            receipt.method == super::super::HostEvidenceMethod::CrossCompile
                && receipt.state == super::super::HostEvidenceState::Passed
        })
    {
        return super::PortabilityMatrixStatus::CrossOnly;
    }
    super::PortabilityMatrixStatus::Partial
}

fn aggregate_grade_statuses(
    statuses: &[(super::super::HostEvidenceGrade, super::PortabilityMatrixStatus)],
) -> super::PortabilityMatrixStatus {
    if statuses.iter().all(|(_, status)| *status == super::PortabilityMatrixStatus::Supported) {
        return super::PortabilityMatrixStatus::Supported;
    }
    for status in [
        super::PortabilityMatrixStatus::Failed,
        super::PortabilityMatrixStatus::Stale,
        super::PortabilityMatrixStatus::Blocked,
        super::PortabilityMatrixStatus::Unsupported,
        super::PortabilityMatrixStatus::Missing,
        super::PortabilityMatrixStatus::CrossOnly,
        super::PortabilityMatrixStatus::Partial,
    ] {
        if statuses.iter().any(|(_, grade_status)| *grade_status == status) {
            return status;
        }
    }
    super::PortabilityMatrixStatus::Partial
}

trait NonEmptyString {
    fn then_nonempty(self) -> Option<String>;
}

impl NonEmptyString for String {
    fn then_nonempty(self) -> Option<String> {
        (!self.is_empty()).then_some(self)
    }
}
