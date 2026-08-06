pub(super) fn merge_validated_reports(
    task: &super::super::WorkerTask,
    reports: &[super::super::Report],
) -> super::super::ReportMergeOutcome {
    debug_assert!(!reports.is_empty());
    debug_assert!(!task.allowed_capabilities().is_empty());
    let evidence = collect_evidence(reports);
    let conflicts = detect_conflicts(reports);
    if !conflicts.is_empty() {
        return super::super::ReportMergeOutcome::Conflict(super::super::ConflictReport { conflicts, evidence });
    }
    super::super::ReportMergeOutcome::Merged(super::super::WorkerMergedReport {
        evidence,
        claims: collect_claims(reports),
        unresolved_mutation_intents: select_mutations(task, reports, MutationSelection::Allowed),
        denied_mutation_intents: select_mutations(task, reports, MutationSelection::Denied),
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum MutationSelection {
    Allowed,
    Denied,
}

#[allow(unbounded_collection_growth)]
fn collect_evidence(reports: &[super::super::Report]) -> Vec<super::super::EvidenceReference> {
    debug_assert!(!reports.is_empty());
    debug_assert!(reports.iter().all(|report| report.evidence.len() <= super::super::MAX_REPORT_FIELDS));
    let mut seen = std::collections::BTreeSet::new();
    let mut evidence = Vec::with_capacity(reports.len());
    for report in reports {
        for reference in &report.evidence {
            if seen.insert(reference.evidence_id.clone()) {
                evidence.push(reference.clone());
            }
        }
    }
    evidence
}

#[allow(unbounded_collection_growth)]
fn collect_claims(reports: &[super::super::Report]) -> Vec<super::super::Claim> {
    debug_assert!(!reports.is_empty());
    debug_assert!(reports.iter().all(|report| report.claims.len() <= super::super::MAX_REPORT_FIELDS));
    reports.iter().flat_map(|report| report.claims.iter().cloned()).collect()
}

fn detect_conflicts(reports: &[super::super::Report]) -> Vec<super::super::Conflict> {
    let mut conflicts = claims(reports);
    conflicts.extend(mutations(reports));
    conflicts
}
pub(super) fn select_mutations(
    task: &super::super::WorkerTask,
    reports: &[super::super::Report],
    selection: MutationSelection,
) -> Vec<super::super::MutationIntent> {
    reports
        .iter()
        .flat_map(|report| report.mutation_intents.iter())
        .filter(|intent| match selection {
            MutationSelection::Allowed => mutation_allowed(task, intent),
            MutationSelection::Denied => !mutation_allowed(task, intent),
        })
        .cloned()
        .collect()
}

#[allow(unbounded_collection_growth)]
pub(super) fn claims(reports: &[super::super::Report]) -> Vec<super::super::Conflict> {
    debug_assert!(!reports.is_empty());
    debug_assert!(reports.iter().all(|report| report.evidence.len() <= super::super::MAX_REPORT_FIELDS));
    let mut by_subject: std::collections::BTreeMap<&str, (&str, Vec<String>)> = std::collections::BTreeMap::new();
    let mut conflicts = Vec::with_capacity(reports.len());
    for claim in reports.iter().flat_map(|report| report.claims.iter()) {
        let evidence_ids = claim.evidence_ids.clone();
        match by_subject.get_mut(claim.subject.as_str()) {
            Some((value, existing_evidence)) if *value != claim.value.as_str() => {
                let mut combined = existing_evidence.clone();
                combined.extend(evidence_ids);
                conflicts.push(super::super::Conflict {
                    code: String::from(super::super::CONFLICT_CLAIM_CODE),
                    subject: claim.subject.clone(),
                    evidence_ids: combined,
                });
            }
            Some((_value, existing_evidence)) => existing_evidence.extend(evidence_ids),
            None => {
                by_subject.insert(claim.subject.as_str(), (claim.value.as_str(), evidence_ids));
            }
        }
    }
    conflicts
}

#[allow(unbounded_collection_growth)]
pub(super) fn mutations(reports: &[super::super::Report]) -> Vec<super::super::Conflict> {
    debug_assert!(!reports.is_empty());
    debug_assert!(reports.iter().all(|report| report.evidence.len() <= super::super::MAX_REPORT_FIELDS));
    let mut by_target: std::collections::BTreeMap<&str, (&str, Vec<String>)> = std::collections::BTreeMap::new();
    let mut conflicts = Vec::with_capacity(reports.len());
    for intent in reports.iter().flat_map(|report| report.mutation_intents.iter()) {
        match by_target.get_mut(intent.target.as_str()) {
            Some((kind, existing_evidence)) if *kind != intent.mutation_kind.as_str() => {
                let mut combined = existing_evidence.clone();
                combined.push(intent.evidence_id.clone());
                conflicts.push(super::super::Conflict {
                    code: String::from(super::super::CONFLICT_MUTATION_CODE),
                    subject: intent.target.clone(),
                    evidence_ids: combined,
                });
            }
            Some((_kind, existing_evidence)) => existing_evidence.push(intent.evidence_id.clone()),
            None => {
                by_target
                    .insert(intent.target.as_str(), (intent.mutation_kind.as_str(), vec![intent.evidence_id.clone()]));
            }
        }
    }
    conflicts
}

fn mutation_allowed(task: &super::super::WorkerTask, intent: &super::super::MutationIntent) -> bool {
    task.allowed_capabilities().iter().any(|capability| {
        matches!(
            capability,
            super::super::WorkerCapability::RequestMutation { mutation_kind }
                if mutation_kind == &intent.mutation_kind
        )
    })
}
