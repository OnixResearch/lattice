//! Default-off intake of declared preference factors from an annotated Choregraph cohort.
//!
//! r[impl lattice.role_protocol.preference_factors]
//! r[impl lattice.role_protocol.preference_factors.identity]
//! r[impl lattice.role_protocol.preference_factors.boundary]
//!
//! The annotated bundle's unchanged v1 projection first passes the existing exact
//! intake. The producer receipt and annotated complete local set are then verified
//! against that bundle. Only an explicit `PreferenceProfile::RecordDeclaredFactors`
//! selection copies the declared factors and worlds into the cohort, as metadata.
//! Nothing here or in the session core reads them to select a label, weigh a branch,
//! convert a label set into a Boolean, or grant authority.

/// One annotated producer cohort plus the transported receipt and annotated local set to verify.
#[derive(Clone, Copy)]
pub struct AnnotatedProjection<'a> {
    pub bundle: &'a choregraph_choreography::preference::linkage::AnnotatedBundle,
    pub receipt: &'a choregraph_choreography::preference::linkage::ReceiptBody,
    pub receipt_bytes: &'a [u8],
    pub local_set: &'a choregraph_choreography::preference::projection::AnnotatedLocalSet,
}

/// Admitted cohort and the report of what happened to its declared factors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnnotatedIntake {
    pub cohort: super::model::Cohort,
    pub report: super::model::PreferenceReport,
}

/// Admits an annotated cohort under an explicit session profile.
///
/// Under the default `ExactLabelBaseline` the returned cohort equals the one
/// `admit_projection` returns for the unchanged v1 bundle, and the report states
/// that the factors were ignored because the profile is off.
pub fn admit_annotated_projection(
    input: AnnotatedProjection<'_>,
    profile: super::model::PreferenceProfile,
) -> Result<AnnotatedIntake, super::model::Error> {
    let base = super::intake::admit_projection(super::intake::FrozenProjection {
        projected: input.bundle.projected(),
        bundle: input.bundle.base(),
    })?;
    choregraph_choreography::preference::linkage::verify_receipt(input.bundle, input.receipt, input.receipt_bytes)
        .map_err(|_| super::model::Issue::IdentityMismatch)?;
    choregraph_choreography::preference::linkage::verify_local_set(input.bundle, input.local_set)
        .map_err(|_| super::model::Issue::IdentityMismatch)?;
    let record = declared_record(&base, input)?;
    debug_assert_eq!(record.roles.len(), base.roles.len());
    let disposition = match profile {
        super::model::PreferenceProfile::ExactLabelBaseline => {
            super::model::FactorDisposition::FactorsIgnoredProfileOff
        }
        super::model::PreferenceProfile::RecordDeclaredFactors => super::model::FactorDisposition::FactorsRecorded,
    };
    let report = super::model::PreferenceReport {
        schema: super::model::PREFERENCE_REPORT_SCHEMA.to_owned(),
        profile,
        disposition,
        game_mapping: super::model::GameMapping::NotFormalized,
        annotated_receipt: record.annotated_receipt.clone(),
        non_claims: record.non_claims.clone(),
    };
    let cohort = match profile {
        super::model::PreferenceProfile::ExactLabelBaseline => base,
        super::model::PreferenceProfile::RecordDeclaredFactors => {
            let cohort = super::model::Cohort {
                preference: Some(record),
                ..base
            };
            validate_record(&cohort)?;
            cohort
        }
    };
    debug_assert_eq!(cohort.preference.is_some(), disposition == super::model::FactorDisposition::FactorsRecorded);
    Ok(AnnotatedIntake { cohort, report })
}

/// Checks that a recorded factor set is bound to this exact cohort and addresses only
/// decider `InternalChoice` nodes with exactly their projected labels.
pub(super) fn validate_record(cohort: &super::model::Cohort) -> Result<(), super::model::Error> {
    let Some(record) = &cohort.preference else {
        return Ok(());
    };
    let is_profile = record.producer_profile == super::model::ANNOTATED_PRODUCER_PROFILE;
    let has_non_claims = record.non_claims.iter().map(String::as_str).eq(super::model::ANNOTATION_NON_CLAIMS);
    if !(is_profile && has_non_claims) {
        return Err(super::model::Issue::UnsupportedCohort.into());
    }
    let linkage = [
        record.base_receipt == cohort.projection_receipt,
        record.complete_local_set == cohort.complete_local_set,
        record.roles.len() == cohort.roles.len(),
        record.roles.iter().any(|entry| !entry.decisions.is_empty()),
    ];
    if linkage.contains(&false) {
        return Err(super::model::Issue::IdentityMismatch.into());
    }
    debug_assert_eq!(record.roles.len(), cohort.roles.len());
    debug_assert!(is_profile && has_non_claims);
    for (entry, definition) in record.roles.iter().zip(&cohort.roles) {
        validate_role(entry, definition)?;
    }
    Ok(())
}

fn validate_role(
    entry: &super::model::RoleChoices,
    definition: &super::model::RoleDefinition,
) -> Result<(), super::model::Error> {
    let is_bound = entry.role == definition.role && entry.local_artifact == definition.local_artifact;
    if !is_bound {
        return Err(super::model::Issue::IdentityMismatch.into());
    }
    for decision in &entry.decisions {
        validate_choice(decision, entry.role, &definition.protocol)?;
    }
    Ok(())
}

fn validate_choice(
    decision: &super::model::DeclaredChoice,
    decider: super::model::RoleId,
    protocol: &super::model::RoleProgram,
) -> Result<(), super::model::Error> {
    let has_factors = !decision.factors.is_empty();
    let is_owned = decision.factors.iter().all(|factor| factor.role == decider);
    if !(has_factors && is_owned) {
        return Err(super::model::Issue::IdentityMismatch.into());
    }
    debug_assert!(has_factors && is_owned);
    let mut current = protocol;
    for step in &decision.local_path {
        current = continuation(current, *step).ok_or(super::model::Issue::IdentityMismatch)?;
    }
    let super::model::RoleProgram::InternalChoice { branches } = current else {
        return Err(super::model::Issue::IdentityMismatch.into());
    };
    if !branches.iter().map(|branch| branch.label).eq(decision.labels.iter().copied()) {
        return Err(super::model::Issue::IdentityMismatch.into());
    }
    debug_assert_eq!(branches.len(), decision.labels.len());
    Ok(())
}

fn continuation(
    node: &super::model::RoleProgram,
    step: super::model::ChoiceStep,
) -> Option<&super::model::RoleProgram> {
    match step {
        super::model::ChoiceStep::Next => next_node(node),
        super::model::ChoiceStep::Branch { label } => branch_node(node, label),
    }
}

fn next_node(node: &super::model::RoleProgram) -> Option<&super::model::RoleProgram> {
    match node {
        super::model::RoleProgram::Send { next, .. }
        | super::model::RoleProgram::Receive { next, .. }
        | super::model::RoleProgram::SendOwned { next, .. }
        | super::model::RoleProgram::ReceiveOwned { next, .. } => Some(next),
        super::model::RoleProgram::End
        | super::model::RoleProgram::InternalChoice { .. }
        | super::model::RoleProgram::Offer { .. } => None,
    }
}

fn branch_node(node: &super::model::RoleProgram, label: super::model::LabelId) -> Option<&super::model::RoleProgram> {
    let branches = match node {
        super::model::RoleProgram::InternalChoice { branches } | super::model::RoleProgram::Offer { branches, .. } => {
            branches
        }
        super::model::RoleProgram::End
        | super::model::RoleProgram::Send { .. }
        | super::model::RoleProgram::Receive { .. }
        | super::model::RoleProgram::SendOwned { .. }
        | super::model::RoleProgram::ReceiveOwned { .. } => return None,
    };
    branches.iter().find(|branch| branch.label == label).map(|branch| &branch.body)
}

fn declared_record(
    base: &super::model::Cohort,
    input: AnnotatedProjection<'_>,
) -> Result<super::model::PreferenceRecord, super::model::Error> {
    linked_receipt(base, input.receipt)?;
    let annotated_local_set = super::identity::from_producer(&input.receipt.annotated_local_set)?;
    let linkage = [
        super::identity::from_producer(&input.local_set.complete_local_set)? == base.complete_local_set,
        annotated_local_set == super::identity::from_producer(&input.bundle.local_set().identity)?,
        input.local_set.locals.len() == base.roles.len(),
    ];
    if linkage.contains(&false) {
        return Err(super::model::Issue::IdentityMismatch.into());
    }
    debug_assert_eq!(input.local_set.locals.len(), base.roles.len());
    let roles = input.local_set.locals.iter().map(role_choices).collect::<Result<Vec<_>, _>>()?;
    debug_assert_eq!(roles.len(), base.roles.len());
    Ok(super::model::PreferenceRecord {
        producer_profile: input.receipt.profile.clone(),
        annotated_receipt: super::identity::from_producer(&input.bundle.receipt().identity)?,
        base_receipt: base.projection_receipt.clone(),
        complete_local_set: base.complete_local_set.clone(),
        annotations: super::identity::from_producer(&input.receipt.annotations)?,
        annotated_local_set,
        game_mapping: super::model::GameMapping::NotFormalized,
        non_claims: super::model::ANNOTATION_NON_CLAIMS.map(str::to_owned).to_vec(),
        roles,
    })
}

fn linked_receipt(
    base: &super::model::Cohort,
    receipt: &choregraph_choreography::preference::linkage::ReceiptBody,
) -> Result<(), super::model::Error> {
    let producer_non_claims = choregraph_choreography::preference::model::REQUIRED_NON_CLAIMS;
    let contract = [
        receipt.profile == super::model::ANNOTATED_PRODUCER_PROFILE,
        receipt.claim == super::model::ANNOTATED_PRODUCER_CLAIM,
        producer_non_claims == super::model::ANNOTATION_NON_CLAIMS,
        matches!(receipt.game_mapping, choregraph_choreography::preference::linkage::GameMapping::NotFormalized),
    ];
    if contract.contains(&false) {
        return Err(super::model::Issue::UnsupportedCohort.into());
    }
    let local_artifacts =
        receipt.local_artifacts.iter().map(super::identity::from_producer).collect::<Result<Vec<_>, _>>()?;
    let linkage = [
        super::identity::from_producer(&receipt.base_receipt)? == base.projection_receipt,
        super::identity::from_producer(&receipt.global_artifact)? == base.global_artifact,
        super::identity::from_producer(&receipt.complete_local_set)? == base.complete_local_set,
        local_artifacts.iter().eq(base.roles.iter().map(|role| &role.local_artifact)),
    ];
    if linkage.contains(&false) {
        return Err(super::model::Issue::IdentityMismatch.into());
    }
    debug_assert_eq!(local_artifacts.len(), base.roles.len());
    debug_assert!(contract.iter().all(|holds| *holds));
    Ok(())
}

fn role_choices(
    local: &choregraph_choreography::preference::projection::AnnotatedLocal,
) -> Result<super::model::RoleChoices, super::model::Error> {
    Ok(super::model::RoleChoices {
        role: super::model::RoleId(local.role.0),
        local_artifact: super::identity::from_producer(&local.local_artifact)?,
        decisions: local.decisions.iter().map(declared_choice).collect(),
    })
}

fn declared_choice(
    decision: &choregraph_choreography::preference::projection::DeciderAnnotation,
) -> super::model::DeclaredChoice {
    let choice = super::model::DeclaredChoice {
        global_path: decision.global_path.iter().map(choice_step).collect(),
        local_path: decision.local_path.iter().map(choice_step).collect(),
        labels: decision.labels.iter().map(|label| super::model::LabelId(label.0)).collect(),
        factors: decision
            .factors
            .iter()
            .map(|factor| super::model::DeclaredFactor {
                id: factor.id.0,
                role: super::model::RoleId(factor.role.0),
                name: factor.name.clone(),
            })
            .collect(),
        world: decision.world.as_ref().map(|world| super::model::DeclaredWorld {
            id: world.id.0,
            name: world.name.clone(),
        }),
    };
    debug_assert_eq!(choice.labels.len(), decision.labels.len());
    debug_assert_eq!(choice.factors.len(), decision.factors.len());
    choice
}

fn choice_step(step: &choregraph_choreography::preference::model::PathStep) -> super::model::ChoiceStep {
    match step {
        choregraph_choreography::preference::model::PathStep::Next => super::model::ChoiceStep::Next,
        choregraph_choreography::preference::model::PathStep::Branch { label } => super::model::ChoiceStep::Branch {
            label: super::model::LabelId(label.0),
        },
    }
}
