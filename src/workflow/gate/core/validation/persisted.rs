fn validate_decision_integrity(
    request: &crate::workflow::gate::model::ApprovalRequest,
    config: &crate::workflow::gate::model::ApprovalNodeConfig,
) -> Result<(), AdmissionError> {
    let expected_class = expected_decision_class(request);
    if request.decision.as_ref().map(|decision| decision.decision) != expected_class {
        return Err(error(AdmissionCode::IdentityFailure, "approval decision does not match request status"));
    }
    let Some(decision) = &request.decision else {
        return Ok(());
    };
    validate_decision_authority(decision, config)?;
    let authority_input_id = authority_input_identity(AuthorityIdentityInput {
        request,
        actor: &decision.actor,
        ability: decision.ability,
        observed_epoch_millis: decision.decided_epoch_millis,
        profile: decision.authority_profile,
        verification_receipt_references: &decision.verification_receipt_references,
    })?;
    if authority_input_id != decision.authority_input_id {
        return Err(error(AdmissionCode::AuthorityDenied, "persisted approval authority-input identity changed"));
    }
    let material = DecisionIdentityMaterial {
        request_id: &request.id,
        workflow_revision: &request.workflow_revision,
        actor: &decision.actor,
        decision: decision.decision,
        ability: decision.ability,
        reason_identity: &decision.reason_identity,
        decided_epoch_millis: decision.decided_epoch_millis,
        authority_input_id: &decision.authority_input_id,
    };
    let expected_id =
        crate::workflow::gate::identity::ApprovalDecisionId::from_generated(hash(DECISION_HASH_DOMAIN, &material)?);
    if expected_id != decision.id {
        return Err(error(AdmissionCode::IdentityFailure, "persisted approval decision identity changed"));
    }
    debug_assert_eq!(decision.ability, crate::workflow::gate::model::DecisionAbility::for_decision(decision.decision));
    debug_assert_eq!(decision.authority_profile, config.authority.profile);
    Ok(())
}

fn expected_decision_class(
    request: &crate::workflow::gate::model::ApprovalRequest,
) -> Option<crate::workflow::gate::model::DecisionClass> {
    match request.status {
        crate::workflow::gate::model::ApprovalRequestStatus::Suspended => None,
        crate::workflow::gate::model::ApprovalRequestStatus::Rejected => {
            Some(crate::workflow::gate::model::DecisionClass::Reject)
        }
        crate::workflow::gate::model::ApprovalRequestStatus::Approved
        | crate::workflow::gate::model::ApprovalRequestStatus::Resuming
        | crate::workflow::gate::model::ApprovalRequestStatus::Resumed
        | crate::workflow::gate::model::ApprovalRequestStatus::ResumeUncertain => {
            Some(crate::workflow::gate::model::DecisionClass::Approve)
        }
        crate::workflow::gate::model::ApprovalRequestStatus::Expired => {
            request.decision.as_ref().map(|decision| decision.decision)
        }
    }
}

fn validate_decision_authority(
    decision: &crate::workflow::gate::model::ApprovalDecision,
    config: &crate::workflow::gate::model::ApprovalNodeConfig,
) -> Result<(), AdmissionError> {
    let has_changed_policy = [
        decision.ability != crate::workflow::gate::model::DecisionAbility::for_decision(decision.decision),
        decision.authority_profile != config.authority.profile,
        decision.delegated_policy_reference != config.authority.delegated_policy_reference,
    ]
    .into_iter()
    .any(std::convert::identity);
    if has_changed_policy {
        return Err(error(AdmissionCode::AuthorityDenied, "persisted approval decision authority changed"));
    }
    match decision.authority_profile {
        crate::workflow::gate::model::AuthorityProfile::LocalOperator
            if !decision.verification_receipt_references.is_empty() =>
        {
            return Err(error(
                AdmissionCode::AuthorityDenied,
                "local approval decision contains delegated verification references",
            ));
        }
        crate::workflow::gate::model::AuthorityProfile::BasaltUcanVerified
            if decision.verification_receipt_references.is_empty()
                || has_duplicates(&decision.verification_receipt_references)
                || decision.verification_receipt_references.iter().any(|reference| !is_valid_reference(reference)) =>
        {
            return Err(error(AdmissionCode::AuthorityDenied, "delegated approval decision references are invalid"));
        }
        crate::workflow::gate::model::AuthorityProfile::LocalOperator
        | crate::workflow::gate::model::AuthorityProfile::BasaltUcanVerified => {}
    }
    debug_assert_eq!(decision.authority_profile, config.authority.profile);
    debug_assert_eq!(decision.ability, crate::workflow::gate::model::DecisionAbility::for_decision(decision.decision));
    Ok(())
}
