fn validate_event_chain(request: &crate::workflow::gate::model::ApprovalRequest) -> Result<(), AdmissionError> {
    if request.events.len() > MAX_REQUEST_EVENTS {
        return Err(error(AdmissionCode::EventHeadMismatch, "approval request event count exceeds its bound"));
    }
    let mut previous: Option<&crate::workflow::gate::identity::ApprovalEventId> = None;
    for (index, event) in request.events.iter().enumerate() {
        let sequence = u64::try_from(index)
            .map_err(|_| error(AdmissionCode::IdentityFailure, "approval event sequence exceeds its bound"))?;
        if event.sequence != sequence || event.previous.as_ref() != previous {
            return Err(error(AdmissionCode::EventHeadMismatch, "approval event chain is not contiguous"));
        }
        let material = EventIdentityMaterial {
            request_id: &request.id,
            previous,
            sequence,
            kind: event.kind,
            state_digest: &event.state_digest,
        };
        let expected_id =
            crate::workflow::gate::identity::ApprovalEventId::from_generated(hash(EVENT_HASH_DOMAIN, &material)?);
        if expected_id != event.id {
            return Err(error(AdmissionCode::EventHeadMismatch, "approval event identity is inconsistent"));
        }
        previous = Some(&event.id);
    }
    let last = request
        .events
        .last()
        .ok_or_else(|| error(AdmissionCode::EventHeadMismatch, "approval request has no event-chain head"))?;
    if last.state_digest != state_digest(request)? {
        return Err(error(
            AdmissionCode::EventHeadMismatch,
            "approval event head does not bind the current request state",
        ));
    }
    debug_assert_eq!(previous, Some(&last.id));
    debug_assert_eq!(
        last.sequence,
        u64::try_from(request.events.len().saturating_sub(EVENT_COUNT_INCREMENT)).unwrap_or(INITIAL_EVENT_SEQUENCE)
    );
    Ok(())
}

fn require_active_resume_owner(
    request: &crate::workflow::gate::model::ApprovalRequest,
    owner: &crate::workflow::gate::identity::ResumeOwnerId,
) -> Result<(), AdmissionError> {
    if request.status != crate::workflow::gate::model::ApprovalRequestStatus::Resuming
        || request.active_resume_owner.as_ref() != Some(owner)
    {
        return Err(error(AdmissionCode::ActiveOwner, "approval resume owner does not match the active claim"));
    }
    Ok(())
}

fn resume_plan(
    request: &crate::workflow::gate::model::ApprovalRequest,
    command: &crate::workflow::gate::model::ResumeCommand,
    config: &crate::workflow::gate::model::ApprovalNodeConfig,
) -> Result<crate::workflow::gate::model::ResumePlan, AdmissionError> {
    let values = resume_value_identities(command, config)?;
    let credentials = resume_credential_identities(command, config)?;
    let material = ResumeIdentityMaterial {
        request_id: &request.id,
        run_id: request.run_id,
        workflow_revision: &request.workflow_revision,
        node_id: request.node_id,
        owner: &command.owner,
        values: &values,
        credentials: &credentials,
    };
    let id = crate::workflow::gate::identity::ApprovalResumeId::from_generated(hash(RESUME_HASH_DOMAIN, &material)?);
    debug_assert_eq!(values.len(), config.resume.values.len());
    debug_assert_eq!(credentials.len(), config.resume.credential_bindings.len());
    Ok(crate::workflow::gate::model::ResumePlan {
        id,
        request_id: request.id.clone(),
        run_id: request.run_id,
        workflow_revision: request.workflow_revision.clone(),
        node_id: request.node_id,
        owner: command.owner.clone(),
        values,
        credentials,
    })
}

fn resume_value_identities(
    command: &crate::workflow::gate::model::ResumeCommand,
    config: &crate::workflow::gate::model::ApprovalNodeConfig,
) -> Result<Vec<crate::workflow::gate::model::ResumeValueIdentity>, AdmissionError> {
    let declared_values = config
        .resume
        .values
        .iter()
        .map(|field| (field.name.as_str(), field.type_tag.as_str()))
        .collect::<std::collections::BTreeMap<_, _>>();
    if command.values.len() != declared_values.len()
        || has_duplicates_by(command.values.iter().map(|value| value.name.as_str()))
    {
        return Err(error(AdmissionCode::ResumeSchemaMismatch, "resume values do not match the declared schema"));
    }
    let mut values = Vec::with_capacity(command.values.len());
    for value in &command.values {
        let expected_type = declared_values
            .get(value.name.as_str())
            .ok_or_else(|| error(AdmissionCode::ResumeSchemaMismatch, "resume value name is not declared"))?;
        if !value.value.sensitive_fragments.is_empty() {
            return Err(error(
                AdmissionCode::PlaintextSecretDenied,
                "resume values cannot contain resolved secret material",
            ));
        }
        let observed_type = value.value.type_tag().as_str();
        if *expected_type != "any" && *expected_type != observed_type {
            return Err(error(AdmissionCode::ResumeSchemaMismatch, "resume value type does not match the declaration"));
        }
        values.push(crate::workflow::gate::model::ResumeValueIdentity {
            name: value.name.clone(),
            type_tag: observed_type.to_owned(),
            value_hash: value.value.hash.to_hex(),
        });
    }
    values.sort_by(|left, right| left.name.cmp(&right.name));
    debug_assert_eq!(values.len(), declared_values.len());
    debug_assert!(values.windows(2).all(|pair| pair[0].name < pair[1].name));
    Ok(values)
}

fn resume_credential_identities(
    command: &crate::workflow::gate::model::ResumeCommand,
    config: &crate::workflow::gate::model::ApprovalNodeConfig,
) -> Result<Vec<crate::workflow::gate::model::ResumeCredentialIdentity>, AdmissionError> {
    let declared_credentials = config.resume.credential_bindings.iter().collect::<std::collections::BTreeSet<_>>();
    if command.credentials.len() != declared_credentials.len()
        || has_duplicates_by(command.credentials.iter().map(|binding| binding.name.as_str()))
    {
        return Err(error(
            AdmissionCode::CredentialDenied,
            "resume credential bindings do not match the declared schema",
        ));
    }
    let mut credentials = Vec::with_capacity(command.credentials.len());
    for binding in &command.credentials {
        if !declared_credentials.contains(&binding.name)
            || crate::credentials::validate_secret_ref(&binding.reference).is_err()
        {
            return Err(error(
                AdmissionCode::CredentialDenied,
                "resume credential binding is not an admitted opaque reference",
            ));
        }
        credentials.push(crate::workflow::gate::model::ResumeCredentialIdentity {
            name: binding.name.clone(),
            reference_identity: binding.reference.identity(),
        });
    }
    credentials.sort_by(|left, right| left.name.cmp(&right.name));
    debug_assert_eq!(credentials.len(), declared_credentials.len());
    debug_assert!(credentials.windows(2).all(|pair| pair[0].name < pair[1].name));
    Ok(credentials)
}

fn validate_resume_expiry(
    request: &crate::workflow::gate::model::ApprovalRequest,
    config: &crate::workflow::gate::model::ApprovalNodeConfig,
    observed_epoch_millis: i64,
) -> Result<(), AdmissionError> {
    let Some(decision) = request.decision.as_ref() else {
        return Err(error(AdmissionCode::WrongStatus, "approved request is missing its decision"));
    };
    let base = if config.expiry.approval_must_be_fresh_at_resume {
        decision.decided_epoch_millis
    } else {
        request.requested_epoch_millis
    };
    let deadline = deadline(base, config.expiry.resume_ttl_millis)?;
    if observed_epoch_millis > deadline {
        return Err(error(AdmissionCode::Expired, "approval resume window is expired"));
    }
    Ok(())
}

fn persisted_authority_references(
    evidence: &crate::workflow::gate::model::DecisionAuthorityEvidence,
) -> (crate::workflow::gate::model::AuthorityProfile, Option<String>, Vec<String>) {
    match evidence {
        crate::workflow::gate::model::DecisionAuthorityEvidence::LocalOperator => {
            (crate::workflow::gate::model::AuthorityProfile::LocalOperator, None, Vec::new())
        }
        crate::workflow::gate::model::DecisionAuthorityEvidence::BasaltUcanVerified(facts) => {
            let mut references = facts.verification_receipt_references.clone();
            references.sort();
            (
                crate::workflow::gate::model::AuthorityProfile::BasaltUcanVerified,
                Some(facts.policy_reference.clone()),
                references,
            )
        }
    }
}

fn authority_input(
    request: &crate::workflow::gate::model::ApprovalRequest,
    config: &crate::workflow::gate::model::ApprovalNodeConfig,
    command: &crate::workflow::gate::model::DecisionCommand,
    ability: crate::workflow::gate::model::DecisionAbility,
) -> Result<crate::workflow::gate::identity::ApprovalAuthorityInputId, AdmissionError> {
    if config.authority.prohibit_self_approval && command.actor == request.requester {
        return Err(error(AdmissionCode::AuthorityDenied, "approval policy prohibits self-approval"));
    }
    let (profile, receipt_refs) = match (&config.authority.profile, &command.evidence) {
        (
            crate::workflow::gate::model::AuthorityProfile::LocalOperator,
            crate::workflow::gate::model::DecisionAuthorityEvidence::LocalOperator,
        ) => {
            if !config.authority.local_actors.contains(&command.actor) {
                return Err(error(AdmissionCode::AuthorityDenied, "local actor is not admitted by approval policy"));
            }
            (crate::workflow::gate::model::AuthorityProfile::LocalOperator, Vec::new())
        }
        (
            crate::workflow::gate::model::AuthorityProfile::BasaltUcanVerified,
            crate::workflow::gate::model::DecisionAuthorityEvidence::BasaltUcanVerified(facts),
        ) => {
            validate_delegated_facts(request, config, command, ability, facts)?;
            (
                crate::workflow::gate::model::AuthorityProfile::BasaltUcanVerified,
                facts.verification_receipt_references.clone(),
            )
        }
        (
            crate::workflow::gate::model::AuthorityProfile::LocalOperator,
            crate::workflow::gate::model::DecisionAuthorityEvidence::BasaltUcanVerified(_),
        )
        | (
            crate::workflow::gate::model::AuthorityProfile::BasaltUcanVerified,
            crate::workflow::gate::model::DecisionAuthorityEvidence::LocalOperator,
        ) => {
            return Err(error(
                AdmissionCode::AuthorityDenied,
                "approval evidence profile does not match current authority policy",
            ));
        }
    };
    let input_id = authority_input_identity(AuthorityIdentityInput {
        request,
        actor: &command.actor,
        ability,
        observed_epoch_millis: command.observed_epoch_millis,
        profile,
        verification_receipt_references: &receipt_refs,
    })?;
    if let crate::workflow::gate::model::DecisionAuthorityEvidence::BasaltUcanVerified(facts) = &command.evidence
        && facts.input_id != input_id
    {
        return Err(error(
            AdmissionCode::AuthorityDenied,
            "verified delegated authority input identity does not match",
        ));
    }
    debug_assert_eq!(ability, crate::workflow::gate::model::DecisionAbility::for_decision(command.decision));
    debug_assert!(!input_id.as_str().is_empty());
    Ok(input_id)
}
