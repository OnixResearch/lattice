/// Compare bounded proposed data; neither a match nor a difference grants effects.
// r[impl lattice.workflow_intent.proposals]
pub fn compare_proposal(
    base: &super::CompiledPlan,
    proposed: super::CompileRequest,
) -> Result<super::ProposalComparison, super::Diagnostic> {
    if proposed.facts != base.facts || proposed.intent.cohort != base.intent.cohort {
        return Err(super::Diagnostic::new(super::DiagnosticCode::BindingMismatch, "proposal_cohort"));
    }
    let proposed = super::compile(proposed)?;
    let before = base
        .intent
        .obligations
        .iter()
        .map(|obligation| (obligation.id.as_str(), obligation))
        .collect::<std::collections::BTreeMap<_, _>>();
    let after = proposed
        .intent
        .obligations
        .iter()
        .map(|obligation| (obligation.id.as_str(), obligation))
        .collect::<std::collections::BTreeMap<_, _>>();
    debug_assert_eq!(before.len(), base.intent.obligations.len());
    debug_assert_eq!(after.len(), proposed.intent.obligations.len());
    Ok(super::ProposalComparison {
        base_plan_identity: base.identity.clone(),
        proposed_plan_identity: proposed.identity.clone(),
        added_obligations: after.keys().filter(|id| !before.contains_key(**id)).map(|id| (*id).to_string()).collect(),
        removed_obligations: before.keys().filter(|id| !after.contains_key(**id)).map(|id| (*id).to_string()).collect(),
        changed_obligations: before
            .iter()
            .filter_map(|(id, obligation)| {
                after.get(id).filter(|current| *current != obligation).map(|_| (*id).to_string())
            })
            .collect(),
        disposition: if base.identity == proposed.identity {
            "identical_inert_plan"
        } else {
            "owner_readmission_required"
        },
    })
}
